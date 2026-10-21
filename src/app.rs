//! Assemblage commun : réseau + presse-papiers + routage des messages.

use crate::clipboard::{self, ClipEvent, ClipboardHandle, SystemBackend};
use crate::config::Settings;
use crate::crypto;
use crate::net::{self, LinkEvent, LinkHandle, NetSettings};
use crate::primary::PeerEvent;
use crate::protocol::Message;
use crate::secondary::{Emulator, SecondaryCore};
use crossbeam_channel::{Receiver, unbounded};
use std::sync::{Arc, Mutex, OnceLock};

/// Fonctions du presse-papiers propres à la plateforme.
#[derive(Clone, Copy)]
pub struct ClipboardHooks {
    pub sequence: fn() -> Option<u64>,
    pub sensitive: fn() -> bool,
}

pub struct Services {
    pub link: LinkHandle,
    pub clipboard: ClipboardHandle,
}

/// Démarre le réseau et le presse-papiers. Les événements de connexion et
/// les messages d'entrée sont transmis à `input`.
pub fn start_services(
    settings: &Settings,
    hooks: ClipboardHooks,
    input: impl Fn(LinkEvent) + Send + Sync + 'static,
) -> Services {
    let clip_cell: Arc<OnceLock<ClipboardHandle>> = Arc::new(OnceLock::new());
    let clip_for_net = clip_cell.clone();
    let link = net::start(
        NetSettings {
            endpoint: settings.endpoint.clone(),
            psk: crypto::derive_psk(&settings.secret),
            role: settings.role,
            name: settings.name.clone(),
        },
        move |ev| {
            let clip = clip_for_net.get();
            match ev {
                LinkEvent::Message(Message::ClipStart { id, total }) => {
                    if let Some(c) = clip {
                        c.notify(ClipEvent::Start { id, total });
                    }
                }
                LinkEvent::Message(Message::ClipChunk { id, data }) => {
                    if let Some(c) = clip {
                        c.notify(ClipEvent::Chunk { id, data });
                    }
                }
                LinkEvent::Connected { name, os } => {
                    if let Some(c) = clip {
                        c.notify(ClipEvent::Connected);
                    }
                    input(LinkEvent::Connected { name, os });
                }
                LinkEvent::Disconnected => {
                    if let Some(c) = clip {
                        c.notify(ClipEvent::Disconnected);
                    }
                    input(LinkEvent::Disconnected);
                }
                other => input(other),
            }
        },
    );
    let seq = hooks.sequence;
    let sensitive = hooks.sensitive;
    let clipboard =
        clipboard::start(move || SystemBackend::new(seq, sensitive), settings.clipboard.clone(), link.clone());
    let _ = clip_cell.set(clipboard.clone());
    // Si la connexion s'est établie avant que le presse-papiers soit prêt.
    if link.is_connected() {
        clipboard.notify(ClipEvent::Connected);
    }
    Services { link, clipboard }
}

/// Machine principale : les événements de l'autre machine arrivent dans le
/// canal renvoyé ; `wake` réveille le thread de capture pour qu'il les traite.
pub fn start_primary(
    settings: &Settings,
    hooks: ClipboardHooks,
    wake: impl Fn() + Send + Sync + 'static,
) -> (Services, Receiver<PeerEvent>) {
    let (tx, rx) = unbounded::<PeerEvent>();
    let services = start_services(settings, hooks, move |ev| {
        let pe = match ev {
            LinkEvent::Connected { .. } => PeerEvent::Connected,
            LinkEvent::Disconnected => PeerEvent::Disconnected,
            LinkEvent::Message(Message::Leave { ratio }) => PeerEvent::Leave { ratio },
            LinkEvent::Message(m) => {
                log::debug!("message ignoré par la machine principale : {m:?}");
                return;
            }
        };
        let _ = tx.send(pe);
        wake();
    });
    (services, rx)
}

/// Machine pilotée : les messages sont appliqués directement par
/// l'émulateur, depuis le thread réseau.
pub fn start_secondary<E: Emulator + Send + 'static>(
    settings: &Settings,
    hooks: ClipboardHooks,
    emulator: E,
) -> Services {
    let state = Arc::new(Mutex::new((SecondaryCore::new(settings.mouse_speed, settings.remap.clone()), emulator)));
    let link_cell: Arc<OnceLock<LinkHandle>> = Arc::new(OnceLock::new());
    let link_for_cb = link_cell.clone();
    let services = start_services(settings, hooks, move |ev| {
        let mut guard = state.lock().unwrap_or_else(|e| e.into_inner());
        let (core, emu) = &mut *guard;
        match ev {
            LinkEvent::Connected { .. } => {}
            LinkEvent::Disconnected => core.disconnected(emu),
            LinkEvent::Message(m) => {
                if let Some(reply) = core.handle(emu, m) {
                    if let Some(link) = link_for_cb.get() {
                        link.send_input(reply);
                    }
                }
            }
        }
    });
    let _ = link_cell.set(services.link.clone());
    services
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Endpoint, default_config_text};
    use crate::geometry::{Layout, Point, Rect};
    use crate::protocol::{Edge, MouseButton, Role};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    enum Ev {
        Move(Point),
        Button(MouseButton, bool),
        Wheel(i32, i32),
        Key(u32, bool, bool),
        ReleaseAll,
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
    }

    fn settings(role: &str, endpoint: Endpoint) -> Settings {
        let text = default_config_text(role, "secret-de-test-integration").replace("enabled = true", "enabled = false");
        let mut s = Config::parse(&text).unwrap().validate().unwrap();
        s.endpoint = endpoint;
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

    #[test]
    fn primary_and_secondary_end_to_end() {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let hooks = ClipboardHooks { sequence: || None, sensitive: || false };

        let emu = RecEmu::default();
        let sec = settings("secondary", Endpoint::Listen(format!("127.0.0.1:{port}")));
        assert_eq!(sec.role, Role::Secondary);
        let _secondary = start_secondary(&sec, hooks, emu.clone());

        let prim = settings("primary", Endpoint::Connect(format!("127.0.0.1:{port}")));
        let (primary, peer_rx) = start_primary(&prim, hooks, || {});
        assert_eq!(peer_rx.recv_timeout(Duration::from_secs(10)).unwrap(), PeerEvent::Connected);

        // Le curseur arrive par la gauche du Mac, bouge, clique, tape une touche.
        let link = &primary.link;
        link.send_input(Message::Enter { edge: Edge::Left, ratio: 0.5 });
        link.send_input(Message::Motion { dx: 30, dy: -10 });
        link.send_input(Message::Button { button: MouseButton::Back, down: true });
        link.send_input(Message::Button { button: MouseButton::Back, down: false });
        link.send_input(Message::Wheel { dx: 0, dy: -120 });
        link.send_input(Message::Key { code: 0x070004, down: true, repeat: false });
        link.send_input(Message::Key { code: 0x070004, down: false, repeat: false });
        assert!(wait_for(|| emu.0.lock().unwrap().len() >= 7));
        let evs = emu.0.lock().unwrap().clone();
        assert_eq!(evs[0], Ev::Move(Point::new(0, 491)));
        assert_eq!(evs[1], Ev::Move(Point::new(30, 481)));
        assert_eq!(evs[2], Ev::Button(MouseButton::Back, true));
        assert_eq!(evs[4], Ev::Wheel(0, -120));
        assert_eq!(evs[5], Ev::Key(0x070004, true, false));

        // Retour vers la gauche : la machine principale reçoit « Leave ».
        link.send_input(Message::Motion { dx: -100, dy: 0 });
        match peer_rx.recv_timeout(Duration::from_secs(5)).unwrap() {
            PeerEvent::Leave { ratio } => assert!((ratio - 0.49).abs() < 0.02, "ratio {ratio}"),
            e => panic!("{e:?}"),
        }

        // Coupure : la machine pilotée relâche tout.
        drop(primary);
        drop(peer_rx);
    }
}
