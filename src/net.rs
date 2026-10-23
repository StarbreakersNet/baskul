//! Connexions réseau.
//!
//! - La machine principale écoute et accepte plusieurs pilotées à la fois ;
//!   chaque pilotée se connecte à la principale (adresse donnée, ou trouvée
//!   par la découverte automatique) et s'y reconnecte d'elle-même.
//! - TCP (`TCP_NODELAY`), éventuellement par une carte réseau imposée.
//! - Poignée de main Noise (voir `crypto.rs`), puis échange de `Hello`.
//! - Trames : longueur sur 2 octets (big-endian) + message chiffré.
//! - Par connexion, un thread lit, un thread écrit. Les messages d'entrée
//!   (souris, clavier) passent avant le presse-papiers ; les déplacements en
//!   attente sont fusionnés.
//! - Maintien de connexion : `Ping` toutes les secondes en l'absence de
//!   trafic ; connexion considérée perdue après 5 s de silence.

use crate::crypto::{self, Sealer};
use crate::discovery;
use crate::i18n;
use crate::protocol::{Message, PROTOCOL_VERSION, Role};
use crate::status::{self, Conn};
use crossbeam_channel::{Receiver, Sender, bounded, select, unbounded};
use std::io::{self, BufReader, Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const PING_INTERVAL: Duration = Duration::from_millis(1000);
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Identifiant d'une connexion (une par pilotée, pour la durée de la
/// connexion).
pub type PeerId = u32;

/// Événements remontés par les connexions.
#[derive(Debug)]
pub enum LinkEvent {
    Connected { peer: PeerId, name: String, os: String },
    Disconnected { peer: PeerId },
    Message { peer: PeerId, msg: Message },
}

pub struct NetSettings {
    pub role: Role,
    pub name: String,
    /// Identifiant propre à la machine (deux pilotées ne peuvent pas porter
    /// le même nom).
    pub machine_id: String,
    pub psk: [u8; 32],
    /// Machine principale : adresse d'écoute.
    pub listen: String,
    /// Machine pilotée : adresse de la principale (`None` : découverte).
    pub connect: Option<String>,
    /// Découverte automatique active.
    pub discovery: bool,
    /// Adresse locale imposée (carte réseau choisie).
    pub bind: Option<IpAddr>,
}

/// Autre machine connectée.
#[derive(Debug, Clone, PartialEq)]
pub struct PeerInfo {
    pub id: PeerId,
    pub name: String,
    pub os: String,
}

struct Peer {
    info: PeerInfo,
    machine_id: String,
    input_tx: Sender<Message>,
    bulk_tx: Sender<Message>,
    /// Pour couper la connexion (remplacée par une plus récente).
    stream: Option<TcpStream>,
}

/// Poignée pour envoyer des messages aux autres machines (machine pilotée :
/// une seule, la principale).
#[derive(Clone, Default)]
pub struct LinkHandle {
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl LinkHandle {
    fn with_peer<T>(&self, peer: PeerId, f: impl FnOnce(&Peer) -> T) -> Option<T> {
        let peers = self.peers.lock().unwrap_or_else(|e| e.into_inner());
        peers.iter().find(|p| p.info.id == peer).map(f)
    }

    /// Envoie un message d'entrée (non bloquant). Ignoré hors connexion.
    pub fn send_input(&self, peer: PeerId, m: Message) {
        self.with_peer(peer, |p| {
            let _ = p.input_tx.send(m);
        });
    }

    /// Envoie un message d'entrée à toutes les machines, sauf `except`.
    pub fn broadcast_input(&self, m: Message, except: Option<PeerId>) {
        let peers = self.peers.lock().unwrap_or_else(|e| e.into_inner());
        for p in peers.iter().filter(|p| Some(p.info.id) != except) {
            let _ = p.input_tx.send(m.clone());
        }
    }

    /// Envoie un message volumineux (presse-papiers). Bloque si le réseau est
    /// saturé ; renvoie `false` si la connexion est perdue.
    pub fn send_bulk(&self, peer: PeerId, m: Message) -> bool {
        let Some(tx) = self.with_peer(peer, |p| p.bulk_tx.clone()) else { return false };
        tx.send_timeout(m, Duration::from_secs(10)).is_ok()
    }

    /// Autres machines connectées, dans l'ordre de connexion.
    pub fn peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|p| p.info.clone()).collect()
    }

    pub fn is_connected(&self, peer: PeerId) -> bool {
        self.with_peer(peer, |_| ()).is_some()
    }

    /// Machine pilotée : la connexion à la machine principale.
    pub fn primary(&self) -> Option<PeerId> {
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).first().map(|p| p.info.id)
    }

    /// Enregistre une connexion établie. Une connexion plus ancienne de la
    /// même machine (reconnexion avant la fin du délai de détection) est
    /// coupée ; une autre machine du même nom est refusée.
    fn register(&self, peer: Peer) -> Result<(), String> {
        let mut peers = self.peers.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(old) = peers.iter().find(|p| p.info.name == peer.info.name) {
            if old.machine_id != peer.machine_id {
                return Err(i18n::tf("status.nameTaken", &[("name", &peer.info.name)]));
            }
            log::info!("« {} » s'est reconnectée : l'ancienne connexion est fermée", peer.info.name);
            if let Some(s) = &old.stream {
                let _ = s.shutdown(Shutdown::Both);
            }
        }
        peers.push(peer);
        Ok(())
    }

    fn unregister(&self, peer: PeerId) {
        self.peers.lock().unwrap_or_else(|e| e.into_inner()).retain(|p| p.info.id != peer);
    }

    /// Poignée avec une machine fictive (`id` 1) : renvoie le récepteur de ses
    /// messages volumineux (tests).
    #[cfg(test)]
    pub fn detached_bulk() -> (LinkHandle, Receiver<Message>) {
        let (input_tx, _input_rx) = unbounded();
        let (bulk_tx, bulk_rx) = unbounded();
        let h = LinkHandle::default();
        let info = PeerInfo { id: 1, name: "test".into(), os: "test".into() };
        h.peers.lock().unwrap().push(Peer { info, machine_id: String::new(), input_tx, bulk_tx, stream: None });
        (h, bulk_rx)
    }
}

type EventFn = Arc<dyn Fn(LinkEvent) + Send + Sync>;

#[derive(Debug)]
enum SessionError {
    Io(io::Error),
    Auth,
    Refused(String),
}

impl From<io::Error> for SessionError {
    fn from(e: io::Error) -> Self {
        SessionError::Io(e)
    }
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

/// Démarre les connexions en arrière-plan : écoute (machine principale) ou
/// connexion à la principale (machine pilotée).
pub fn start(settings: NetSettings, on_event: impl Fn(LinkEvent) + Send + Sync + 'static) -> LinkHandle {
    let handle = LinkHandle::default();
    let links = handle.clone();
    let on_event: EventFn = Arc::new(on_event);
    let s = Arc::new(settings);
    std::thread::Builder::new()
        .name("baskul-net".into())
        .spawn(move || match s.role {
            Role::Primary => server(s, links, on_event),
            Role::Secondary => client(s, links, on_event),
        })
        .expect("thread réseau");
    handle
}

/// Adresse d'écoute, sur la carte réseau choisie s'il y en a une.
fn listen_addr(s: &NetSettings) -> String {
    match s.bind {
        Some(ip) => {
            let port = s
                .listen
                .rsplit_once(':')
                .and_then(|(_, p)| p.parse::<u16>().ok())
                .unwrap_or(crate::protocol::DEFAULT_PORT);
            SocketAddr::new(ip, port).to_string()
        }
        None => s.listen.clone(),
    }
}

/// Machine principale : accepte les pilotées, chacune dans son thread.
fn server(s: Arc<NetSettings>, links: LinkHandle, on_event: EventFn) {
    let addr = listen_addr(&s);
    let listener = loop {
        match TcpListener::bind(addr.as_str()) {
            Ok(l) => break l,
            Err(e) => {
                log::error!("impossible d'écouter sur {addr} : {e}");
                status::set_conn(Conn::Error(i18n::tf(
                    "status.listenFailed",
                    &[("address", &addr), ("error", &e.to_string())],
                )));
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    };
    log::info!("en écoute sur {addr}");
    status::set_conn(Conn::Waiting(i18n::tf("status.waitingClients", &[("address", &addr)])));
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(st) => st,
            Err(e) => {
                log::warn!("accept : {e}");
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
        };
        let peer_addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
        log::info!("connexion entrante de {peer_addr}");
        let (s, links, on_event) = (s.clone(), links.clone(), on_event.clone());
        let _ = std::thread::Builder::new().name("baskul-net-peer".into()).spawn(move || {
            match session(&s, stream, false, &links, &on_event) {
                Ok(()) => log::info!("connexion de {peer_addr} terminée"),
                Err(SessionError::Io(e)) => log::warn!("connexion de {peer_addr} : {e}"),
                // Mauvais secret, version ou nom : seule la pilotée l'affiche.
                Err(SessionError::Auth) => log::warn!("{peer_addr} : {}", i18n::t("status.wrongSecret")),
                Err(SessionError::Refused(msg)) => log::warn!("{peer_addr} : {msg}"),
            }
        });
    }
}

/// Machine pilotée : se connecte à la principale et s'y reconnecte.
fn client(s: Arc<NetSettings>, links: LinkHandle, on_event: EventFn) {
    let mut backoff = Duration::from_millis(500);
    // Dernière erreur, gardée visible tant qu'aucune connexion ne réussit.
    let mut last_error: Option<String> = None;
    let waiting = |base: String, last_error: &Option<String>| match last_error {
        Some(e) => Conn::Error(format!("{e} — {base}")),
        None => Conn::Waiting(base),
    };
    loop {
        // Adresses à essayer : celle de la configuration, ou les machines
        // principales trouvées sur le réseau.
        let targets: Vec<(String, Vec<SocketAddr>)> = match &s.connect {
            Some(addr) => {
                status::set_conn(waiting(i18n::tf("status.connecting", &[("address", addr)]), &last_error));
                vec![(addr.clone(), addr.to_socket_addrs().map(|a| a.collect()).unwrap_or_default())]
            }
            None if !s.discovery => {
                status::set_conn(Conn::Error(i18n::t("status.noAddress")));
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            None => {
                let found: Vec<_> =
                    discovery::primaries().into_iter().filter(|f| f.protocol == PROTOCOL_VERSION).collect();
                if found.is_empty() {
                    status::set_conn(waiting(i18n::t("status.searching"), &last_error));
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                found.into_iter().map(|f| (f.name, f.addresses)).collect()
            }
        };
        let mut session_ran = false;
        'targets: for (label, addrs) in targets {
            for a in addrs {
                let stream = match connect(a, s.bind) {
                    Ok(st) => st,
                    Err(e) => {
                        log::debug!("connexion à {label} ({a}) impossible : {e}");
                        continue;
                    }
                };
                log::info!("connecté à {label} ({a})");
                session_ran = true;
                let result = session(&s, stream, true, &links, &on_event);
                match result {
                    Ok(()) => log::info!("connexion terminée"),
                    Err(SessionError::Io(e)) => log::warn!("connexion perdue : {e}"),
                    Err(SessionError::Auth) => {
                        let msg = i18n::t("status.wrongSecret");
                        log::error!("{label} : {msg}");
                        last_error = Some(msg);
                        continue 'targets;
                    }
                    Err(SessionError::Refused(msg)) => {
                        log::error!("{label} : {msg}");
                        last_error = Some(msg);
                        continue 'targets;
                    }
                }
                last_error = None;
                backoff = Duration::from_millis(500);
                break 'targets;
            }
        }
        if let Some(e) = &last_error {
            status::set_conn(Conn::Error(e.clone()));
        }
        // Espacement des essais (plus long après un refus).
        let pause = if last_error.is_some() { Duration::from_secs(5) } else { backoff };
        std::thread::sleep(pause);
        if !session_ran {
            backoff = (backoff * 2).min(Duration::from_secs(4));
        }
    }
}

/// Connexion TCP, depuis l'adresse locale `bind` si une carte est imposée.
fn connect(addr: SocketAddr, bind: Option<IpAddr>) -> io::Result<TcpStream> {
    use socket2::{Domain, Protocol, Socket, Type};
    let sock = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    if let Some(ip) = bind {
        if ip.is_ipv4() != addr.is_ipv4() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "famille d'adresses différente de la carte choisie",
            ));
        }
        sock.bind(&SocketAddr::new(ip, 0).into())?;
    }
    sock.connect_timeout(&addr.into(), CONNECT_TIMEOUT)?;
    Ok(sock.into())
}

fn write_frame(stream: &mut impl Write, frame: &mut Vec<u8>, data: &[u8]) -> io::Result<()> {
    frame.clear();
    frame.extend_from_slice(&(data.len() as u16).to_be_bytes());
    frame.extend_from_slice(data);
    stream.write_all(frame)
}

fn read_frame(r: &mut impl Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut len = [0u8; 2];
    r.read_exact(&mut len)?;
    let n = u16::from_be_bytes(len) as usize;
    r.read_exact(&mut buf[..n])?;
    Ok(n)
}

fn session(
    s: &NetSettings,
    stream: TcpStream,
    initiator: bool,
    links: &LinkHandle,
    on_event: &EventFn,
) -> Result<(), SessionError> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
    let mut write_half = stream.try_clone()?;
    let mut reader = BufReader::with_capacity(128 * 1024, stream.try_clone()?);
    let mut frame = Vec::with_capacity(crypto::MAX_MSG + 2);
    let mut buf = vec![0u8; crypto::MAX_MSG];

    // --- Poignée de main Noise ---
    let mut hs = crypto::new_handshake(&s.psk, initiator).map_err(|_| SessionError::Auth)?;
    let mut tmp = vec![0u8; crypto::MAX_MSG];
    if initiator {
        let n = hs.write_message(&[], &mut tmp).map_err(|_| SessionError::Auth)?;
        write_frame(&mut write_half, &mut frame, &tmp[..n])?;
        let n = read_frame(&mut reader, &mut buf)?;
        hs.read_message(&buf[..n], &mut tmp).map_err(|_| SessionError::Auth)?;
    } else {
        let n = read_frame(&mut reader, &mut buf)?;
        hs.read_message(&buf[..n], &mut tmp).map_err(|_| SessionError::Auth)?;
        let n = hs.write_message(&[], &mut tmp).map_err(|_| SessionError::Auth)?;
        write_frame(&mut write_half, &mut frame, &tmp[..n])?;
    }
    let transport = hs.into_stateless_transport_mode().map_err(|_| SessionError::Auth)?;
    let (mut sealer, mut opener) = crypto::split(transport);

    // --- Présentation ---
    // La pilotée se présente ; la principale vérifie (version, rôle, nom) puis
    // répond par sa propre présentation, ou par un refus motivé.
    let hello = Message::Hello {
        version: PROTOCOL_VERSION,
        role: s.role,
        name: s.name.clone(),
        os: std::env::consts::OS.to_string(),
        id: s.machine_id.clone(),
    };
    let mut send = |m: &Message, sealer: &mut Sealer| -> Result<(), SessionError> {
        let c = sealer.seal(&m.encode()).map_err(|_| SessionError::Auth)?;
        write_frame(&mut write_half, &mut frame, c)?;
        Ok(())
    };
    if initiator {
        send(&hello, &mut sealer)?;
    }
    let n = read_frame(&mut reader, &mut buf)?;
    let plain = opener.open(&buf[..n]).map_err(|_| SessionError::Auth)?;
    let checked = match Message::decode(plain) {
        Ok(Message::Hello { version, role, name, os, id }) => {
            if version != PROTOCOL_VERSION {
                Err(i18n::tf(
                    "status.versionMismatch",
                    &[("here", &PROTOCOL_VERSION.to_string()), ("name", &name), ("there", &version.to_string())],
                ))
            } else if role == s.role {
                let role = i18n::t(if role == Role::Primary { "status.rolePrimary" } else { "status.roleSecondary" });
                Err(i18n::tf("status.sameRole", &[("role", &role), ("name", &name)]))
            } else {
                Ok((name, os, id))
            }
        }
        Ok(Message::Refused { reason }) => return Err(SessionError::Refused(reason)),
        // Ancienne version : le message ne se décode pas.
        _ => Err(i18n::t("status.unexpectedReply")),
    };
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (input_tx, input_rx) = unbounded::<Message>();
    let (bulk_tx, bulk_rx) = bounded::<Message>(8);
    let registered = checked.and_then(|(name, os, machine)| {
        let info = PeerInfo { id, name: name.clone(), os: os.clone() };
        let peer = Peer { info, machine_id: machine, input_tx, bulk_tx, stream: stream.try_clone().ok() };
        links.register(peer).map(|()| (name, os))
    });
    let (peer_name, peer_os) = match registered {
        Ok(p) => p,
        Err(reason) => {
            if !initiator {
                let _ = send(&Message::Refused { reason: reason.clone() }, &mut sealer);
            }
            return Err(SessionError::Refused(reason));
        }
    };
    if !initiator {
        if let Err(e) = send(&hello, &mut sealer) {
            links.unregister(id);
            return Err(e);
        }
    }
    log::info!("connecté à « {peer_name} » ({peer_os})");
    status::add_peer(&peer_name, &peer_os);
    on_event(LinkEvent::Connected { peer: id, name: peer_name.clone(), os: peer_os });

    // --- Échanges ---
    let (stop_tx, stop_rx) = bounded::<()>(0);
    let writer = std::thread::Builder::new()
        .name("baskul-net-tx".into())
        .spawn(move || writer_loop(write_half, frame, sealer, input_rx, bulk_rx, stop_rx))
        .map_err(SessionError::Io);

    let result = writer.and_then(|writer| {
        let r = (|| -> Result<(), SessionError> {
            loop {
                let n = read_frame(&mut reader, &mut buf)?;
                let plain = opener.open(&buf[..n]).map_err(|_| SessionError::Auth)?;
                match Message::decode(plain) {
                    Ok(Message::Ping) => {}
                    Ok(Message::Hello { .. }) => log::warn!("Hello inattendu ignoré"),
                    Ok(Message::Refused { reason }) => return Err(SessionError::Refused(reason)),
                    Ok(msg) => on_event(LinkEvent::Message { peer: id, msg }),
                    Err(e) => log::warn!("message illisible ignoré : {e}"),
                }
            }
        })();
        let _ = stream.shutdown(Shutdown::Both);
        drop(stop_tx);
        let _ = writer.join();
        r
    });

    links.unregister(id);
    // Pas de retrait si une connexion plus récente de la même machine a pris
    // la place.
    if !links.peers().iter().any(|p| p.name == peer_name) {
        status::remove_peer(&peer_name);
    }
    on_event(LinkEvent::Disconnected { peer: id });
    result
}

fn writer_loop(
    mut stream: TcpStream,
    mut frame: Vec<u8>,
    mut sealer: Sealer,
    input_rx: Receiver<Message>,
    bulk_rx: Receiver<Message>,
    stop_rx: Receiver<()>,
) {
    let mut pending: Option<Message> = None;
    loop {
        let msg = match pending.take() {
            Some(m) => m,
            None => match input_rx.try_recv() {
                Ok(m) => m,
                Err(_) => select! {
                    recv(input_rx) -> m => match m { Ok(m) => m, Err(_) => return },
                    recv(bulk_rx) -> m => match m { Ok(m) => m, Err(_) => return },
                    recv(stop_rx) -> _ => return,
                    default(PING_INTERVAL) => Message::Ping,
                },
            },
        };
        // Fusion des déplacements en attente : si le réseau a pris du retard,
        // on envoie un seul déplacement cumulé plutôt qu'une file qui s'allonge.
        let msg = match msg {
            Message::Motion { mut dx, mut dy } => {
                loop {
                    match input_rx.try_recv() {
                        Ok(Message::Motion { dx: a, dy: b }) => {
                            dx = dx.saturating_add(a);
                            dy = dy.saturating_add(b);
                        }
                        Ok(other) => {
                            pending = Some(other);
                            break;
                        }
                        Err(_) => break,
                    }
                }
                Message::Motion { dx, dy }
            }
            m => m,
        };
        let plain = msg.encode();
        let cipher = match sealer.seal(&plain) {
            Ok(c) => c,
            Err(e) => {
                log::error!("chiffrement impossible : {e}");
                let _ = stream.shutdown(Shutdown::Both);
                return;
            }
        };
        if let Err(e) = write_frame(&mut stream, &mut frame, cipher) {
            log::debug!("écriture réseau : {e}");
            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Edge, MouseButton};
    use std::time::Instant;

    fn free_port() -> u16 {
        TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    fn settings(role: Role, name: &str, id: &str, secret: &str, port: u16) -> NetSettings {
        NetSettings {
            role,
            name: name.into(),
            machine_id: id.into(),
            psk: crypto::derive_psk(secret),
            listen: format!("127.0.0.1:{port}"),
            connect: Some(format!("127.0.0.1:{port}")),
            discovery: false,
            bind: None,
        }
    }

    fn start_with(s: NetSettings) -> (LinkHandle, Receiver<LinkEvent>) {
        let (tx, rx) = unbounded();
        let h = start(s, move |e| {
            let _ = tx.send(e);
        });
        (h, rx)
    }

    /// Principale « A » en écoute, pilotée « B » qui s'y connecte.
    fn start_pair(
        secret_a: &str,
        secret_b: &str,
        role_b: Role,
    ) -> (LinkHandle, Receiver<LinkEvent>, LinkHandle, Receiver<LinkEvent>) {
        let port = free_port();
        let (a, a_rx) = start_with(settings(Role::Primary, "A", "id-a", secret_a, port));
        let (b, b_rx) = start_with(settings(role_b, "B", "id-b", secret_b, port));
        (a, a_rx, b, b_rx)
    }

    fn wait_connected(rx: &Receiver<LinkEvent>) -> (PeerId, String) {
        match rx.recv_timeout(Duration::from_secs(10)).expect("connexion") {
            LinkEvent::Connected { peer, name, .. } => (peer, name),
            e => panic!("événement inattendu {e:?}"),
        }
    }

    fn next_message(rx: &Receiver<LinkEvent>) -> (PeerId, Message) {
        match rx.recv_timeout(Duration::from_secs(5)).unwrap() {
            LinkEvent::Message { peer, msg } => (peer, msg),
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn end_to_end_exchange() {
        let (a, a_rx, b, b_rx) = start_pair("le-meme-secret", "le-meme-secret", Role::Secondary);
        let (to_b, name) = wait_connected(&a_rx);
        assert_eq!(name, "B");
        let (to_a, name) = wait_connected(&b_rx);
        assert_eq!(name, "A");
        assert_eq!(b.primary(), Some(to_a));

        a.send_input(to_b, Message::Enter { edge: Edge::Left, ratio: 0.5, exits: vec![Edge::Left] });
        a.send_input(to_b, Message::Button { button: MouseButton::Left, down: true });
        a.send_input(to_b, Message::Key { code: 0x070004, down: true, repeat: false });
        let got: Vec<Message> = (0..3).map(|_| next_message(&b_rx).1).collect();
        assert_eq!(got[0], Message::Enter { edge: Edge::Left, ratio: 0.5, exits: vec![Edge::Left] });
        assert_eq!(got[2], Message::Key { code: 0x070004, down: true, repeat: false });

        // Sens inverse + gros message.
        b.send_input(to_a, Message::Leave { edge: Edge::Left, ratio: 0.25 });
        assert!(b.send_bulk(to_a, Message::ClipChunk { id: 1, data: vec![7; crate::protocol::CLIP_CHUNK] }));
        for _ in 0..2 {
            match next_message(&a_rx) {
                (p, Message::Leave { ratio, .. }) => assert_eq!((p, ratio), (to_b, 0.25)),
                (_, Message::ClipChunk { data, .. }) => assert_eq!(data.len(), crate::protocol::CLIP_CHUNK),
                m => panic!("{m:?}"),
            }
        }
    }

    #[test]
    fn motions_are_preserved_in_total() {
        let (a, a_rx, _b, b_rx) = start_pair("secret-de-test", "secret-de-test", Role::Secondary);
        let (to_b, _) = wait_connected(&a_rx);
        wait_connected(&b_rx);
        for _ in 0..1000 {
            a.send_input(to_b, Message::Motion { dx: 1, dy: -2 });
        }
        a.send_input(to_b, Message::ReleaseAll);
        let (mut sx, mut sy) = (0, 0);
        loop {
            match next_message(&b_rx).1 {
                Message::Motion { dx, dy } => {
                    sx += dx;
                    sy += dy;
                }
                Message::ReleaseAll => break,
                m => panic!("{m:?}"),
            }
        }
        assert_eq!((sx, sy), (1000, -2000));
    }

    #[test]
    fn wrong_secret_never_connects() {
        let (_a, a_rx, _b, b_rx) = start_pair("secret-numero-un", "secret-numero-deux", Role::Secondary);
        assert!(a_rx.recv_timeout(Duration::from_secs(2)).is_err());
        assert!(b_rx.recv_timeout(Duration::from_millis(100)).is_err());
    }

    #[test]
    fn same_roles_are_refused() {
        let (_a, a_rx, _b, b_rx) = start_pair("secret-de-test", "secret-de-test", Role::Primary);
        assert!(a_rx.recv_timeout(Duration::from_secs(2)).is_err());
        assert!(b_rx.recv_timeout(Duration::from_millis(100)).is_err());
    }

    #[test]
    fn several_piloted_machines_at_once() {
        let port = free_port();
        let (a, a_rx) = start_with(settings(Role::Primary, "A", "id-a", "secret-de-test", port));
        let (_b, b_rx) = start_with(settings(Role::Secondary, "B", "id-b", "secret-de-test", port));
        let (_c, c_rx) = start_with(settings(Role::Secondary, "C", "id-c", "secret-de-test", port));
        let mut names: Vec<String> = (0..2).map(|_| wait_connected(&a_rx).1).collect();
        names.sort();
        assert_eq!(names, ["B", "C"]);
        wait_connected(&b_rx);
        wait_connected(&c_rx);
        a.broadcast_input(Message::ReleaseAll, None);
        assert_eq!(next_message(&b_rx).1, Message::ReleaseAll);
        assert_eq!(next_message(&c_rx).1, Message::ReleaseAll);
    }

    #[test]
    fn a_second_machine_with_the_same_name_is_refused() {
        let port = free_port();
        let (a, a_rx) = start_with(settings(Role::Primary, "A", "id-a", "secret-de-test", port));
        let (_b, b_rx) = start_with(settings(Role::Secondary, "B", "id-b", "secret-de-test", port));
        wait_connected(&a_rx);
        wait_connected(&b_rx);
        // Autre machine, même nom : refusée.
        let (_other, other_rx) = start_with(settings(Role::Secondary, "B", "id-autre", "secret-de-test", port));
        assert!(other_rx.recv_timeout(Duration::from_secs(2)).is_err());
        assert_eq!(a.peers().len(), 1);
    }

    #[test]
    fn the_same_machine_reconnecting_replaces_its_connection() {
        let port = free_port();
        let (a, a_rx) = start_with(settings(Role::Primary, "A", "id-a", "secret-de-test", port));
        let (_b1, b1_rx) = start_with(settings(Role::Secondary, "B", "id-b", "secret-de-test", port));
        let (first, _) = wait_connected(&a_rx);
        wait_connected(&b1_rx);
        // Même machine (redémarrage de Baskul) : l'ancienne connexion est fermée.
        // La fin de l'ancienne et l'arrivée de la nouvelle se signalent dans un
        // ordre quelconque ; la première pilotée, toujours lancée ici, se
        // reconnecte ensuite à son tour (événements ignorés).
        let (_b2, b2_rx) = start_with(settings(Role::Secondary, "B", "id-b", "secret-de-test", port));
        wait_connected(&b2_rx);
        let (mut second, mut first_closed) = (None, false);
        while second.is_none() || !first_closed {
            match a_rx.recv_timeout(Duration::from_secs(10)).expect("événement") {
                LinkEvent::Connected { peer, .. } => second = second.or(Some(peer)),
                LinkEvent::Disconnected { peer } => first_closed |= peer == first,
                LinkEvent::Message { .. } => {}
            }
        }
        assert_ne!(second, Some(first));
        assert!(a.peers().iter().all(|p| p.id != first));
    }

    #[test]
    fn accepts_a_real_client_after_garbage() {
        // Un client brut se connecte puis coupe : le serveur continue
        // d'accepter les vraies pilotées.
        let port = free_port();
        let (_a, a_rx) = start_with(settings(Role::Primary, "A", "id-a", "secret-de-test", port));
        std::thread::sleep(Duration::from_millis(200));
        {
            let mut raw = TcpStream::connect(("127.0.0.1", port)).unwrap();
            raw.write_all(&[0, 3, 1, 2, 3]).unwrap();
        }
        let (_b, b_rx) = start_with(settings(Role::Secondary, "B", "id-b", "secret-de-test", port));
        let t = Instant::now();
        assert_eq!(wait_connected(&b_rx).1, "A");
        assert_eq!(wait_connected(&a_rx).1, "B");
        assert!(t.elapsed() < Duration::from_secs(10));
    }
}
