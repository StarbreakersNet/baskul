//! Connexion réseau entre les deux machines.
//!
//! - TCP (`TCP_NODELAY`), une seule connexion à la fois.
//! - Poignée de main Noise (voir `crypto.rs`), puis échange de `Hello`.
//! - Trames : longueur sur 2 octets (big-endian) + message chiffré.
//! - Un thread lit, un thread écrit. Les messages d'entrée (souris, clavier)
//!   sont prioritaires sur le presse-papiers ; les déplacements en attente
//!   sont fusionnés.
//! - Maintien de connexion : `Ping` toutes les secondes en l'absence de
//!   trafic ; connexion considérée perdue après 5 s de silence.
//! - Reconnexion automatique.

use crate::config::Endpoint;
use crate::crypto::{self, Sealer};
use crate::i18n;
use crate::protocol::{Message, PROTOCOL_VERSION, Role};
use crate::status::{self, Conn};
use crossbeam_channel::{Receiver, Sender, bounded, select, unbounded};
use std::io::{self, BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const PING_INTERVAL: Duration = Duration::from_millis(1000);
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Événements remontés par la connexion.
#[derive(Debug)]
pub enum LinkEvent {
    Connected { name: String, os: String },
    Disconnected,
    Message(Message),
}

pub struct NetSettings {
    pub endpoint: Endpoint,
    pub psk: [u8; 32],
    pub role: Role,
    pub name: String,
}

/// Poignée pour envoyer des messages à l'autre machine.
#[derive(Clone)]
pub struct LinkHandle {
    input_tx: Sender<Message>,
    bulk_tx: Sender<Message>,
    connected: Arc<AtomicBool>,
}

impl LinkHandle {
    /// Envoie un message d'entrée (non bloquant). Ignoré hors connexion.
    pub fn send_input(&self, m: Message) {
        if self.connected.load(Ordering::Acquire) {
            let _ = self.input_tx.send(m);
        }
    }

    /// Envoie un message volumineux (presse-papiers). Bloque si le réseau est
    /// saturé ; renvoie `false` si la connexion est perdue.
    pub fn send_bulk(&self, m: Message) -> bool {
        if !self.connected.load(Ordering::Acquire) {
            return false;
        }
        self.bulk_tx.send_timeout(m, Duration::from_secs(10)).is_ok()
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    /// Poignée « connectée » sans réseau : renvoie le récepteur des messages
    /// volumineux (tests).
    #[cfg(test)]
    pub fn detached_bulk() -> (LinkHandle, Receiver<Message>) {
        let (input_tx, _input_rx) = unbounded();
        let (bulk_tx, bulk_rx) = unbounded();
        let h = LinkHandle { input_tx, bulk_tx, connected: Arc::new(AtomicBool::new(true)) };
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

/// Démarre la gestion de connexion en arrière-plan.
pub fn start(settings: NetSettings, on_event: impl Fn(LinkEvent) + Send + Sync + 'static) -> LinkHandle {
    let (input_tx, input_rx) = unbounded::<Message>();
    let (bulk_tx, bulk_rx) = bounded::<Message>(8);
    let connected = Arc::new(AtomicBool::new(false));
    let handle = LinkHandle { input_tx, bulk_tx, connected: connected.clone() };
    let on_event: EventFn = Arc::new(on_event);
    std::thread::Builder::new()
        .name("baskul-net".into())
        .spawn(move || manager(settings, input_rx, bulk_rx, connected, on_event))
        .expect("thread réseau");
    handle
}

fn manager(
    s: NetSettings,
    input_rx: Receiver<Message>,
    bulk_rx: Receiver<Message>,
    connected: Arc<AtomicBool>,
    on_event: EventFn,
) {
    let mut listener: Option<TcpListener> = None;
    let mut backoff = Duration::from_millis(500);
    // Dernière erreur, gardée visible dans l'état tant qu'aucune connexion ne réussit.
    let mut last_error: Option<String> = None;
    let waiting = |base: String, last_error: &Option<String>| match last_error {
        Some(e) => Conn::Error(format!("{e} — {base}")),
        None => Conn::Waiting(base),
    };
    loop {
        let (stream, initiator) = match &s.endpoint {
            Endpoint::Listen(addr) => {
                if listener.is_none() {
                    match TcpListener::bind(addr.as_str()) {
                        Ok(l) => {
                            log::info!("en écoute sur {addr}");
                            listener = Some(l);
                        }
                        Err(e) => {
                            log::error!("impossible d'écouter sur {addr} : {e}");
                            status::set_conn(Conn::Error(i18n::tf(
                                "status.listenFailed",
                                &[("address", addr), ("error", &e.to_string())],
                            )));
                            std::thread::sleep(Duration::from_secs(5));
                            continue;
                        }
                    }
                }
                status::set_conn(waiting(i18n::tf("status.waiting", &[("address", addr)]), &last_error));
                match listener.as_ref().unwrap().accept() {
                    Ok((stream, peer)) => {
                        log::info!("connexion entrante de {peer}");
                        (stream, false)
                    }
                    Err(e) => {
                        log::warn!("accept : {e}");
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }
            Endpoint::Connect(addr) => {
                status::set_conn(waiting(i18n::tf("status.connecting", &[("address", addr)]), &last_error));
                match connect(addr) {
                    Ok(stream) => (stream, true),
                    Err(e) => {
                        log::debug!("connexion à {addr} impossible : {e}");
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(4));
                        continue;
                    }
                }
            }
        };
        backoff = Duration::from_millis(500);
        let result = session(&s, stream, initiator, &input_rx, &bulk_rx, &connected, &on_event);
        let was_connected = connected.swap(false, Ordering::AcqRel);
        if was_connected {
            on_event(LinkEvent::Disconnected);
        }
        if was_connected {
            last_error = None;
        }
        // Côté « écoute », on ne patiente jamais : une connexion parasite ne
        // doit pas retarder la vraie. Côté « connexion », on espace les essais.
        let mut pause = Duration::from_millis(500);
        match result {
            Ok(()) => log::info!("connexion terminée"),
            Err(SessionError::Io(e)) => {
                if was_connected {
                    log::warn!("connexion perdue : {e}");
                } else {
                    log::warn!("échec de la poignée de main : {e}");
                }
            }
            Err(SessionError::Auth) => {
                let msg = i18n::t("status.wrongSecret");
                log::error!("{msg}");
                status::set_conn(Conn::Error(msg.clone()));
                last_error = Some(msg);
                pause = Duration::from_secs(5);
            }
            Err(SessionError::Refused(msg)) => {
                log::error!("{msg}");
                status::set_conn(Conn::Error(msg.clone()));
                last_error = Some(msg);
                pause = Duration::from_secs(5);
            }
        }
        if initiator {
            std::thread::sleep(pause);
        }
    }
}

fn connect(addr: &str) -> io::Result<TcpStream> {
    let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();
    let mut last = io::Error::new(io::ErrorKind::NotFound, "adresse introuvable");
    for a in addrs {
        match TcpStream::connect_timeout(&a, CONNECT_TIMEOUT) {
            Ok(s) => {
                log::info!("connecté à {a}");
                return Ok(s);
            }
            Err(e) => last = e,
        }
    }
    Err(last)
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
    input_rx: &Receiver<Message>,
    bulk_rx: &Receiver<Message>,
    connected: &Arc<AtomicBool>,
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
    let hello = Message::Hello {
        version: PROTOCOL_VERSION,
        role: s.role,
        name: s.name.clone(),
        os: std::env::consts::OS.to_string(),
    };
    let c = sealer.seal(&hello.encode()).map_err(|_| SessionError::Auth)?;
    write_frame(&mut write_half, &mut frame, c)?;
    let n = read_frame(&mut reader, &mut buf)?;
    let plain = opener.open(&buf[..n]).map_err(|_| SessionError::Auth)?;
    let (peer_name, peer_os) = match Message::decode(plain) {
        Ok(Message::Hello { version, role, name, os }) => {
            if version != PROTOCOL_VERSION {
                return Err(SessionError::Refused(i18n::tf(
                    "status.versionMismatch",
                    &[("here", &PROTOCOL_VERSION.to_string()), ("name", &name), ("there", &version.to_string())],
                )));
            }
            if role == s.role {
                let role = i18n::t(if role == Role::Primary { "status.rolePrimary" } else { "status.roleSecondary" });
                return Err(SessionError::Refused(i18n::tf("status.sameRole", &[("role", &role)])));
            }
            (name, os)
        }
        _ => return Err(SessionError::Refused(i18n::t("status.unexpectedReply"))),
    };

    // Messages restés en file d'une connexion précédente : obsolètes.
    while input_rx.try_recv().is_ok() {}
    while bulk_rx.try_recv().is_ok() {}

    log::info!("connecté à « {peer_name} » ({peer_os})");
    connected.store(true, Ordering::Release);
    status::set_connected(&peer_name, &peer_os);
    on_event(LinkEvent::Connected { name: peer_name, os: peer_os });

    // --- Échanges ---
    let (stop_tx, stop_rx) = bounded::<()>(0);
    let in_rx = input_rx.clone();
    let b_rx = bulk_rx.clone();
    let writer = std::thread::Builder::new()
        .name("baskul-net-tx".into())
        .spawn(move || writer_loop(write_half, frame, sealer, in_rx, b_rx, stop_rx))
        .map_err(SessionError::Io)?;

    let result = (|| -> Result<(), SessionError> {
        loop {
            let n = read_frame(&mut reader, &mut buf)?;
            let plain = opener.open(&buf[..n]).map_err(|_| SessionError::Auth)?;
            match Message::decode(plain) {
                Ok(Message::Ping) => {}
                Ok(Message::Hello { .. }) => log::warn!("Hello inattendu ignoré"),
                Ok(m) => on_event(LinkEvent::Message(m)),
                Err(e) => log::warn!("message illisible ignoré : {e}"),
            }
        }
    })();

    connected.store(false, Ordering::Release);
    let _ = stream.shutdown(Shutdown::Both);
    drop(stop_tx);
    let _ = writer.join();
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

    fn start_pair(
        secret_a: &str,
        secret_b: &str,
        role_b: Role,
    ) -> (LinkHandle, Receiver<LinkEvent>, LinkHandle, Receiver<LinkEvent>) {
        let port = free_port();
        let (a_tx, a_rx) = unbounded();
        let (b_tx, b_rx) = unbounded();
        let a = start(
            NetSettings {
                endpoint: Endpoint::Listen(format!("127.0.0.1:{port}")),
                psk: crypto::derive_psk(secret_a),
                role: Role::Primary,
                name: "A".into(),
            },
            move |e| {
                let _ = a_tx.send(e);
            },
        );
        let b = start(
            NetSettings {
                endpoint: Endpoint::Connect(format!("127.0.0.1:{port}")),
                psk: crypto::derive_psk(secret_b),
                role: role_b,
                name: "B".into(),
            },
            move |e| {
                let _ = b_tx.send(e);
            },
        );
        (a, a_rx, b, b_rx)
    }

    fn wait_connected(rx: &Receiver<LinkEvent>) -> String {
        match rx.recv_timeout(Duration::from_secs(10)).expect("connexion") {
            LinkEvent::Connected { name, .. } => name,
            e => panic!("événement inattendu {e:?}"),
        }
    }

    #[test]
    fn end_to_end_exchange() {
        let (a, a_rx, b, b_rx) = start_pair("le-meme-secret", "le-meme-secret", Role::Secondary);
        assert_eq!(wait_connected(&a_rx), "B");
        assert_eq!(wait_connected(&b_rx), "A");

        a.send_input(Message::Enter { edge: Edge::Left, ratio: 0.5 });
        a.send_input(Message::Button { button: MouseButton::Left, down: true });
        a.send_input(Message::Key { code: 0x070004, down: true, repeat: false });
        let mut got = vec![];
        while got.len() < 3 {
            match b_rx.recv_timeout(Duration::from_secs(5)).unwrap() {
                LinkEvent::Message(m) => got.push(m),
                e => panic!("{e:?}"),
            }
        }
        assert_eq!(got[0], Message::Enter { edge: Edge::Left, ratio: 0.5 });
        assert_eq!(got[2], Message::Key { code: 0x070004, down: true, repeat: false });

        // Sens inverse + gros message.
        b.send_input(Message::Leave { ratio: 0.25 });
        assert!(b.send_bulk(Message::ClipChunk { id: 1, data: vec![7; crate::protocol::CLIP_CHUNK] }));
        let mut n = 0;
        while n < 2 {
            match a_rx.recv_timeout(Duration::from_secs(5)).unwrap() {
                LinkEvent::Message(Message::Leave { ratio }) => {
                    assert_eq!(ratio, 0.25);
                    n += 1;
                }
                LinkEvent::Message(Message::ClipChunk { data, .. }) => {
                    assert_eq!(data.len(), crate::protocol::CLIP_CHUNK);
                    n += 1;
                }
                e => panic!("{e:?}"),
            }
        }
    }

    #[test]
    fn motions_are_preserved_in_total() {
        let (a, a_rx, _b, b_rx) = start_pair("secret-de-test", "secret-de-test", Role::Secondary);
        wait_connected(&a_rx);
        wait_connected(&b_rx);
        for _ in 0..1000 {
            a.send_input(Message::Motion { dx: 1, dy: -2 });
        }
        a.send_input(Message::ReleaseAll);
        let (mut sx, mut sy) = (0, 0);
        loop {
            match b_rx.recv_timeout(Duration::from_secs(5)).unwrap() {
                LinkEvent::Message(Message::Motion { dx, dy }) => {
                    sx += dx;
                    sy += dy;
                }
                LinkEvent::Message(Message::ReleaseAll) => break,
                e => panic!("{e:?}"),
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
    fn reconnects_after_peer_drop() {
        // Le serveur écoute ; on simule un client brut qui se connecte puis coupe :
        // le serveur doit revenir en écoute et accepter un vrai client ensuite.
        let port = free_port();
        let (a_tx, a_rx) = unbounded();
        let _a = start(
            NetSettings {
                endpoint: Endpoint::Listen(format!("127.0.0.1:{port}")),
                psk: crypto::derive_psk("secret-de-test"),
                role: Role::Primary,
                name: "A".into(),
            },
            move |e| {
                let _ = a_tx.send(e);
            },
        );
        std::thread::sleep(Duration::from_millis(200));
        {
            let mut raw = TcpStream::connect(("127.0.0.1", port)).unwrap();
            raw.write_all(&[0, 3, 1, 2, 3]).unwrap();
        }
        let (b_tx, b_rx) = unbounded();
        let _b = start(
            NetSettings {
                endpoint: Endpoint::Connect(format!("127.0.0.1:{port}")),
                psk: crypto::derive_psk("secret-de-test"),
                role: Role::Secondary,
                name: "B".into(),
            },
            move |e| {
                let _ = b_tx.send(e);
            },
        );
        let t = Instant::now();
        assert_eq!(wait_connected(&b_rx), "A");
        assert_eq!(wait_connected(&a_rx), "B");
        assert!(t.elapsed() < Duration::from_secs(10));
    }
}
