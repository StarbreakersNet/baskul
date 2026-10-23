//! Assemblage commun : réseau + presse-papiers + routage des messages.

use crate::api;
use crate::clipboard::{self, ClipEvent, ClipboardHandle, SystemBackend};
use crate::config::Settings;
use crate::crypto;
use crate::discovery;
use crate::i18n;
use crate::iface;
use crate::net::{self, LinkEvent, LinkHandle, NetSettings};
use crate::primary::{ACTIVITY_INTERVAL, PeerEvent, RECONNECT_GRACE};
use crate::protocol::{ClipHeader, ClipImage};
use crate::protocol::{DEFAULT_PORT, Message, Role};
use crate::secondary::{self, Emulator, SecondaryCore};
use crossbeam_channel::{Receiver, unbounded};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Fonctions du presse-papiers propres à la plateforme.
#[derive(Clone, Copy)]
pub struct ClipboardHooks {
    pub sequence: fn() -> Option<u64>,
    pub sensitive: fn() -> bool,
    /// Image du presse-papiers dans son format d'origine (la première), et
    /// éventuellement des formats de secours.
    pub read_images: fn() -> Vec<ClipImage>,
    /// Écriture par le système (texte, HTML, images, fichiers) ; `None` :
    /// écriture par arboard.
    pub write: fn(&ClipHeader, &[PathBuf]) -> Option<bool>,
    /// Formats présents dans le presse-papiers, pour le journal.
    pub formats: fn() -> Vec<String>,
}

impl ClipboardHooks {
    /// Aucune fonction propre à la plateforme : tout passe par arboard.
    pub const NONE: ClipboardHooks = ClipboardHooks {
        sequence: || None,
        sensitive: || false,
        read_images: Vec::new,
        write: |_, _| None,
        formats: Vec::new,
    };
}

pub struct Services {
    pub link: LinkHandle,
    pub clipboard: ClipboardHandle,
}

/// Démarre le réseau (avec la découverte automatique) et le presse-papiers.
/// Les événements de connexion et les messages d'entrée sont transmis à
/// `input`.
pub fn start_services(
    settings: &Settings,
    hooks: ClipboardHooks,
    input: impl Fn(LinkEvent) + Send + Sync + 'static,
) -> Services {
    let bind = settings.interface.as_deref().and_then(|name| {
        let ip = iface::address_of(name);
        match ip {
            Some(ip) => log::info!("carte réseau « {name} » ({ip})"),
            None => log::warn!("carte réseau « {name} » introuvable ou sans adresse : choix automatique"),
        }
        ip
    });
    if settings.discovery {
        match settings.role {
            Role::Primary => {
                let port = settings.listen.rsplit_once(':').and_then(|(_, p)| p.parse().ok()).unwrap_or(DEFAULT_PORT);
                discovery::advertise(&settings.name, port, settings.interface.as_deref());
            }
            Role::Secondary => discovery::browse(settings.interface.as_deref()),
        }
    }
    let clip_cell: Arc<OnceLock<ClipboardHandle>> = Arc::new(OnceLock::new());
    let clip_for_net = clip_cell.clone();
    let link = net::start(
        NetSettings {
            role: settings.role,
            name: settings.name.clone(),
            machine_id: crate::config::machine_id(),
            psk: crypto::derive_psk(&settings.secret),
            listen: settings.listen.clone(),
            connect: settings.connect.clone(),
            discovery: settings.discovery,
            bind,
        },
        move |ev| {
            let clip = |e: ClipEvent| {
                if let Some(c) = clip_for_net.get() {
                    c.notify(e);
                }
            };
            match ev {
                LinkEvent::Message { peer, msg: Message::ClipStart { id, header, total } } => {
                    clip(ClipEvent::Start { peer, id, header, total })
                }
                LinkEvent::Message { peer, msg: Message::ClipChunk { id, data } } => {
                    clip(ClipEvent::Chunk { peer, id, data })
                }
                LinkEvent::Message { peer, msg: Message::ClipCancel { id } } => clip(ClipEvent::Cancel { peer, id }),
                LinkEvent::Connected { peer, name, os } => {
                    clip(ClipEvent::Connected { peer });
                    input(LinkEvent::Connected { peer, name, os });
                }
                LinkEvent::Disconnected { peer } => {
                    clip(ClipEvent::Disconnected { peer });
                    input(LinkEvent::Disconnected { peer });
                }
                other => input(other),
            }
        },
    );
    let clipboard = clipboard::start(move || SystemBackend::new(hooks), settings.clipboard.clone(), link.clone());
    let _ = clip_cell.set(clipboard.clone());
    // Connexions établies avant que le presse-papiers soit prêt.
    for p in link.peers() {
        clipboard.notify(ClipEvent::Connected { peer: p.id });
    }
    Services { link, clipboard }
}

/// Machine principale : les événements des pilotées arrivent dans le canal
/// renvoyé ; `wake` réveille le thread de capture pour qu'il les traite.
pub fn start_primary(
    settings: &Settings,
    hooks: ClipboardHooks,
    wake: impl Fn() + Send + Sync + 'static,
) -> (Services, Receiver<PeerEvent>) {
    let (tx, rx) = unbounded::<PeerEvent>();
    let wake = Arc::new(wake);
    // Commande `baskul switch`, traitée comme un événement des pilotées.
    let (switch_tx, switch_wake) = (tx.clone(), wake.clone());
    api::set_switch_handler(Role::Primary, move |to| {
        let _ = switch_tx.send(PeerEvent::Switch { from: None, to });
        switch_wake();
        Ok(())
    });
    let services = start_services(settings, hooks, move |ev| {
        let pe = match ev {
            LinkEvent::Connected { peer, name, .. } => PeerEvent::Connected { id: peer, name },
            LinkEvent::Disconnected { peer } => {
                // Vérification à la fin du délai de grâce de cette coupure.
                let (tx, wake) = (tx.clone(), wake.clone());
                let _ = std::thread::Builder::new().name("baskul-grace".into()).spawn(move || {
                    std::thread::sleep(RECONNECT_GRACE + Duration::from_millis(50));
                    let _ = tx.send(PeerEvent::GraceOver);
                    wake();
                });
                PeerEvent::Disconnected { id: peer }
            }
            LinkEvent::Message { peer, msg: Message::Leave { edge, ratio } } => {
                PeerEvent::Leave { id: peer, edge, ratio }
            }
            LinkEvent::Message { peer, msg: Message::Switch { to } } => PeerEvent::Switch { from: Some(peer), to },
            LinkEvent::Message { peer, msg: Message::UserActive } => PeerEvent::UserActive { from: peer },
            LinkEvent::Message { msg, .. } => {
                log::debug!("message ignoré par la machine principale : {msg:?}");
                return;
            }
        };
        let _ = tx.send(pe);
        wake();
    });
    (services, rx)
}

/// Machine pilotée : les messages sont appliqués directement par
/// l'émulateur, depuis le thread réseau. `idle` : temps écoulé depuis la
/// dernière entrée reçue par le système (activité partagée).
pub fn start_secondary<E: Emulator + Send + 'static>(
    settings: &Settings,
    hooks: ClipboardHooks,
    emulator: E,
    idle: fn() -> Option<Duration>,
) -> Services {
    let state = Arc::new(Mutex::new((SecondaryCore::new(settings.mouse_speed, settings.remap.clone()), emulator)));
    let link_cell: Arc<OnceLock<LinkHandle>> = Arc::new(OnceLock::new());
    let link_for_cb = link_cell.clone();
    let mirror = settings.mirror_activity;
    let services = start_services(settings, hooks, move |ev| {
        let mut guard = state.lock().unwrap_or_else(|e| e.into_inner());
        let (core, emu) = &mut *guard;
        match ev {
            LinkEvent::Connected { .. } => {}
            LinkEvent::Disconnected { .. } => core.disconnected(emu),
            // L'utilisateur est actif sur une autre machine.
            LinkEvent::Message { msg: Message::UserActive, .. } => {
                if mirror {
                    secondary::note_injected();
                    emu.keep_active();
                }
            }
            LinkEvent::Message { peer, msg } => {
                secondary::note_injected();
                if let Some(reply) = core.handle(emu, msg) {
                    if let Some(link) = link_for_cb.get() {
                        link.send_input(peer, reply);
                    }
                }
            }
        }
    });
    let _ = link_cell.set(services.link.clone());
    // Commande `baskul switch` : transmise à la machine principale.
    let link = services.link.clone();
    api::set_switch_handler(Role::Secondary, move |to| {
        let Some(primary) = link.primary() else {
            return Err(i18n::t("status.noPrimary"));
        };
        link.send_input(primary, Message::Switch { to });
        Ok(())
    });
    watch_local_activity(services.link.clone(), idle);
    services
}

/// Machine pilotée : son propre clavier ou trackpad (et non les événements
/// injectés par Baskul) compte aussi comme activité, signalée à la machine
/// principale.
fn watch_local_activity(link: LinkHandle, idle: fn() -> Option<Duration>) {
    const POLL: Duration = Duration::from_secs(2);
    let _ = std::thread::Builder::new().name("baskul-activity".into()).spawn(move || {
        let mut sent: Option<Instant> = None;
        loop {
            std::thread::sleep(POLL);
            let Some(idle) = idle() else { return };
            let Some(primary) = link.primary() else { continue };
            if idle >= POLL || sent.is_some_and(|t| t.elapsed() < ACTIVITY_INTERVAL) {
                continue;
            }
            if is_local_input(Instant::now(), idle, secondary::injected_at()) {
                sent = Some(Instant::now());
                link.send_input(primary, Message::UserActive);
            }
        }
    });
}

/// La dernière entrée du système (il y a `idle`) n'est pas un événement
/// injecté par Baskul (le dernier datant de `injected`).
fn is_local_input(now: Instant, idle: Duration, injected: Option<Instant>) -> bool {
    const MARGIN: Duration = Duration::from_millis(500);
    let Some(input) = now.checked_sub(idle) else { return false };
    injected.is_none_or(|t| input > t + MARGIN)
}

/// Machine principale : enregistre la grille (pilotée placée d'elle-même),
/// en arrière-plan.
pub fn save_layout(config_path: &Path, machines: Vec<crate::config::MachineCfg>) {
    let path = config_path.to_path_buf();
    let _ = std::thread::Builder::new().name("baskul-layout".into()).spawn(move || {
        let result = crate::config::load_or_create(&path).and_then(|(mut cfg, _)| {
            cfg.layout.machines = machines;
            crate::config::write_config(&path, &cfg)
        });
        match result {
            Ok(()) => log::info!("grille des machines enregistrée"),
            Err(e) => log::warn!("enregistrement de la grille impossible : {e}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, default_config_text};
    use crate::geometry::{Layout, Point, Rect};
    use crate::net::PeerId;
    use crate::protocol::{Edge, MouseButton, SwitchTo};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    enum Ev {
        Move(Point),
        Button(MouseButton, bool),
        Wheel(i32, i32),
        Key(u32, bool, bool),
        ReleaseAll,
        KeepActive,
    }

    #[derive(Clone, Default)]
    struct RecEmu(Arc<Mutex<Vec<Ev>>>);

    impl Emulator for RecEmu {
        fn layout(&mut self) -> Layout {
            Layout::new(vec![Rect::new(0, 0, 1512, 982)])
        }
        fn move_to(&mut self, p: Point, _dx: i32, _dy: i32) {
            self.0.lock().unwrap().push(Ev::Move(p));
        }
        fn button(&mut self, b: MouseButton, down: bool) {
            self.0.lock().unwrap().push(Ev::Button(b, down));
        }
        fn wheel(&mut self, dx: i32, dy: i32) {
            self.0.lock().unwrap().push(Ev::Wheel(dx, dy));
        }
        fn key(&mut self, code: u32, down: bool, repeat: bool) {
            self.0.lock().unwrap().push(Ev::Key(code, down, repeat));
        }
        fn release_all(&mut self) {
            self.0.lock().unwrap().push(Ev::ReleaseAll);
        }
        fn keep_active(&mut self) {
            self.0.lock().unwrap().push(Ev::KeepActive);
        }
    }

    /// Réglages de test : `name`, sans presse-papiers ni découverte ; la
    /// principale écoute sur `port`, les pilotées s'y connectent.
    fn settings(role: &str, name: &str, port: u16) -> Settings {
        let text = default_config_text(role, "secret-de-test-integration").replace("enabled = true", "enabled = false");
        let mut s = Config::parse(&text).unwrap().validate().unwrap();
        s.name = name.into();
        s.discovery = false;
        s.listen = format!("127.0.0.1:{port}");
        s.connect = Some(format!("127.0.0.1:{port}"));
        s
    }

    fn wait_for(f: impl Fn() -> bool) -> bool {
        for _ in 0..200 {
            if f() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        false
    }

    /// Prochain événement de connexion : identifiant et nom.
    fn connected(rx: &Receiver<PeerEvent>) -> (PeerId, String) {
        match rx.recv_timeout(Duration::from_secs(10)).unwrap() {
            PeerEvent::Connected { id, name } => (id, name),
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn primary_and_two_piloted_machines_end_to_end() {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let hooks = ClipboardHooks::NONE;

        let prim = settings("primary", "Bureau", port);
        let (primary, peer_rx) = start_primary(&prim, hooks, || {});

        let emu = RecEmu::default();
        let atelier = start_secondary(&settings("secondary", "Atelier", port), hooks, emu.clone(), || None);
        let (id_a, name) = connected(&peer_rx);
        assert_eq!(name, "Atelier");
        let salon_emu = RecEmu::default();
        let salon = start_secondary(&settings("secondary", "Salon", port), hooks, salon_emu.clone(), || None);
        let (id_s, name) = connected(&peer_rx);
        assert_eq!(name, "Salon");
        assert_eq!(primary.link.peers().len(), 2);

        // Le curseur arrive par la gauche d'Atelier, bouge, clique, tape une touche.
        let link = &primary.link;
        link.send_input(id_a, Message::Enter { edge: Edge::Left, ratio: 0.5, exits: vec![Edge::Left] });
        link.send_input(id_a, Message::Motion { dx: 30, dy: -10 });
        link.send_input(id_a, Message::Button { button: MouseButton::Back, down: true });
        link.send_input(id_a, Message::Button { button: MouseButton::Back, down: false });
        link.send_input(id_a, Message::Wheel { dx: 0, dy: -120 });
        link.send_input(id_a, Message::Key { code: 0x070004, down: true, repeat: false });
        link.send_input(id_a, Message::Key { code: 0x070004, down: false, repeat: false });
        assert!(wait_for(|| emu.0.lock().unwrap().len() >= 7));
        let evs = emu.0.lock().unwrap().clone();
        assert_eq!(evs[0], Ev::Move(Point::new(0, 491)));
        assert_eq!(evs[1], Ev::Move(Point::new(30, 481)));
        assert_eq!(evs[2], Ev::Button(MouseButton::Back, true));
        assert_eq!(evs[4], Ev::Wheel(0, -120));
        assert_eq!(evs[5], Ev::Key(0x070004, true, false));
        // Rien n'est arrivé sur Salon.
        assert!(salon_emu.0.lock().unwrap().is_empty());

        // Retour vers la gauche : la machine principale reçoit « Leave ».
        link.send_input(id_a, Message::Motion { dx: -100, dy: 0 });
        match peer_rx.recv_timeout(Duration::from_secs(5)).unwrap() {
            PeerEvent::Leave { id, edge: Edge::Left, ratio } => {
                assert_eq!(id, id_a);
                assert!((ratio - 0.49).abs() < 0.02, "ratio {ratio}");
            }
            e => panic!("{e:?}"),
        }

        // Activité partagée : l'utilisateur est actif sur la machine principale.
        link.broadcast_input(Message::UserActive, None);
        assert!(wait_for(|| emu.0.lock().unwrap().last() == Some(&Ev::KeepActive)));
        assert!(wait_for(|| salon_emu.0.lock().unwrap().last() == Some(&Ev::KeepActive)));

        // Dans l'autre sens : commande `baskul switch` lancée sur Salon,
        // activité de son propre clavier.
        let to_primary = salon.link.primary().unwrap();
        salon.link.send_input(to_primary, Message::Switch { to: SwitchTo::Sender });
        assert_eq!(
            peer_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            PeerEvent::Switch { from: Some(id_s), to: SwitchTo::Sender }
        );
        salon.link.send_input(to_primary, Message::UserActive);
        assert_eq!(peer_rx.recv_timeout(Duration::from_secs(5)).unwrap(), PeerEvent::UserActive { from: id_s });

        // Coupure d'Atelier : la principale le voit, Salon reste connectée.
        drop(atelier);
        drop(primary);
        drop(peer_rx);
    }

    #[test]
    fn local_input_is_told_from_injected_events() {
        let now = Instant::now();
        let s = Duration::from_secs;
        // Rien d'injecté : toute entrée vient de la machine elle-même.
        assert!(is_local_input(now, s(1), None));
        // Dernière entrée = dernier événement injecté par Baskul.
        assert!(!is_local_input(now, s(1), Some(now - s(1))));
        // Entrée 1 s après le dernier événement injecté : trackpad du Mac.
        assert!(is_local_input(now, s(1), Some(now - s(2))));
    }
}
