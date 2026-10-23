//! Logique de la machine principale (celle qui a le clavier et la souris),
//! indépendante du système.
//!
//! Le module de capture de la plateforme appelle ces fonctions depuis ses
//! hooks d'entrée ; chaque fonction indique si l'événement doit être laissé
//! passer au système local (`Pass`) ou supprimé (`Swallow`, quand il est
//! envoyé à l'autre machine).

use crate::capture;
use crate::config::MediaKeys;
use crate::geometry::{Layout, Point};
use crate::hotkey::Hotkey;
use crate::keymap;
use crate::protocol::{Edge, Message, MouseButton};
use crate::status;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Laisser l'événement au système local.
    Pass,
    /// Supprimer l'événement localement.
    Swallow,
}

/// Où va une touche ou un bouton enfoncé (mémorisé pour que le relâchement
/// aille au même endroit que l'appui, même si le curseur a changé de machine).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Local,
    Remote,
    /// Touche utilisée par un raccourci de Bascule : supprimée partout.
    Consumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Local,
    Remote,
}

/// Événements venant de l'autre machine.
#[derive(Debug, Clone, PartialEq)]
pub enum PeerEvent {
    Connected,
    Disconnected,
    Leave {
        ratio: f32,
    },
    /// Fin du délai de grâce ouvert par la n-ième coupure (voir
    /// `RECONNECT_GRACE`).
    GraceOver(u64),
}

/// Coupure pendant que l'on pilote l'autre machine : on attend ce délai
/// avant de ramener le curseur ici. Une reconnexion entre-temps (redémarrage
/// de Bascule sur l'autre machine, après un changement de réglages…) reprend
/// le pilotage là où il en était.
pub const RECONNECT_GRACE: std::time::Duration = std::time::Duration::from_secs(4);

#[derive(Debug, Clone)]
pub struct PrimaryOptions {
    /// Bascule en poussant la souris contre le bord de l'écran.
    pub edge_switching: bool,
    /// Position de la machine secondaire par rapport à la principale.
    pub position: Edge,
    pub corner: i32,
    pub switch_hotkey: Option<Hotkey>,
    pub return_hotkey: Option<Hotkey>,
    pub lock_hotkey: Option<Hotkey>,
    /// Machine où agissent les touches multimédia.
    pub media_keys: MediaKeys,
}

/// Services fournis par la plateforme.
pub trait Host {
    /// Écrans locaux (mis en cache par la plateforme).
    fn layout(&self) -> &Layout;
    /// Position actuelle du curseur.
    fn cursor(&self) -> Point;
    /// Le curseur part sur l'autre machine. `at` : position actuelle du
    /// curseur. La plateforme cache le curseur et peut le recentrer, puis
    /// appeler `PrimaryCore::on_warped`.
    fn hide_cursor(&mut self, at: Point);
    /// Le curseur revient : le placer en `pos` et le réafficher.
    fn show_cursor(&mut self, pos: Point);
    /// Envoie un message à la machine secondaire.
    fn send(&mut self, msg: Message);
}

pub struct PrimaryCore {
    opts: PrimaryOptions,
    connected: bool,
    mode: Mode,
    locked: bool,
    /// Position réelle du curseur pendant que l'on pilote l'autre machine.
    anchor: Point,
    /// Le curseur a été recentré : on peut filtrer les déplacements aberrants.
    warp_done: bool,
    /// Position où le curseur a quitté l'écran local.
    exit_point: Point,
    key_targets: HashMap<u32, Target>,
    button_targets: [Option<Target>; 5],
    /// Touches physiquement enfoncées (pour les raccourcis).
    pressed: HashSet<u32>,
    /// Nombre de coupures (pour reconnaître le délai de grâce en cours).
    disconnects: u64,
}

impl PrimaryCore {
    pub fn new(opts: PrimaryOptions) -> PrimaryCore {
        PrimaryCore {
            opts,
            connected: false,
            mode: Mode::Local,
            locked: false,
            anchor: Point::default(),
            warp_done: false,
            exit_point: Point::default(),
            key_targets: HashMap::new(),
            button_targets: [None; 5],
            pressed: HashSet::new(),
            disconnects: 0,
        }
    }

    pub fn is_remote(&self) -> bool {
        self.mode == Mode::Remote
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    fn input_target(&self) -> Target {
        match (self.mode, self.connected) {
            (Mode::Remote, true) => Target::Remote,
            // Coupure pendant le pilotage (délai de grâce) : rien ne part ni
            // n'arrive aux applications d'ici.
            (Mode::Remote, false) => Target::Consumed,
            (Mode::Local, _) => Target::Local,
        }
    }

    fn local_buttons_down(&self) -> bool {
        self.button_targets.contains(&Some(Target::Local))
    }

    // ------------------------------------------------------------------
    // Souris
    // ------------------------------------------------------------------

    /// Déplacement en mode local : `cur` = position réelle du curseur,
    /// `next` = position demandée (éventuellement hors écran).
    pub fn local_motion<H: Host>(&mut self, host: &mut H, cur: Point, next: Point) -> Decision {
        if self.mode != Mode::Local
            || !self.connected
            || !self.opts.edge_switching
            || self.locked
            || self.local_buttons_down()
        {
            return Decision::Pass;
        }
        let layout = host.layout();
        if !layout.pushes_against(cur, next, self.opts.position, self.opts.corner) {
            return Decision::Pass;
        }
        let ratio = layout.ratio_on_edge(layout.clamp(next), self.opts.position);
        log::debug!("bascule vers l'autre machine (ratio {ratio:.3})");
        self.go_remote(host, cur, Message::Enter { edge: self.opts.position.opposite(), ratio });
        Decision::Swallow
    }

    /// Le curseur part sur l'autre machine depuis `cur` ; `msg` l'y fait entrer.
    fn go_remote<H: Host>(&mut self, host: &mut H, cur: Point, msg: Message) {
        self.mode = Mode::Remote;
        self.exit_point = cur;
        self.anchor = cur;
        self.warp_done = false;
        host.send(msg);
        host.hide_cursor(cur);
        status::set_remote_active(true);
    }

    /// La plateforme a recentré le curseur (mode distant).
    pub fn on_warped(&mut self, anchor: Point) {
        if self.mode == Mode::Remote {
            self.anchor = anchor;
            self.warp_done = true;
        }
    }

    /// Déplacement absolu en mode distant (Windows : le curseur réel reste
    /// immobile, la position demandée moins la position réelle donne le
    /// déplacement).
    pub fn remote_abs_motion<H: Host>(&mut self, host: &mut H, pt: Point) {
        let dx = pt.x - self.anchor.x;
        let dy = pt.y - self.anchor.y;
        if dx == 0 && dy == 0 {
            return;
        }
        if self.warp_done && self.is_bogus(host, dx, dy) {
            log::debug!("déplacement aberrant ignoré ({dx}, {dy})");
            return;
        }
        self.remote_motion(host, dx, dy);
    }

    /// Déplacement relatif en mode distant.
    pub fn remote_motion<H: Host>(&mut self, host: &mut H, dx: i32, dy: i32) {
        if self.mode == Mode::Remote && self.connected && (dx != 0 || dy != 0) {
            host.send(Message::Motion { dx, dy });
        }
    }

    /// Juste après le recentrage, un événement émis avant celui-ci peut donner
    /// un déplacement d'environ la distance centre → bord : on l'ignore.
    fn is_bogus<H: Host>(&self, host: &H, dx: i32, dy: i32) -> bool {
        const ZONE: i32 = 10;
        let a = self.anchor;
        let d = host.layout().nearest_display(a);
        -dx + ZONE > a.x - d.left || dx + ZONE > d.right - a.x || -dy + ZONE > a.y - d.top || dy + ZONE > d.bottom - a.y
    }

    pub fn button<H: Host>(&mut self, host: &mut H, button: MouseButton, down: bool) -> Decision {
        let i = button.index();
        let target = if down {
            let t = self.button_targets[i].unwrap_or_else(|| self.input_target());
            self.button_targets[i] = Some(t);
            t
        } else {
            self.button_targets[i].take().unwrap_or_else(|| self.input_target())
        };
        match target {
            Target::Local => Decision::Pass,
            Target::Remote => {
                if self.connected {
                    host.send(Message::Button { button, down });
                }
                Decision::Swallow
            }
            Target::Consumed => Decision::Swallow,
        }
    }

    pub fn wheel<H: Host>(&mut self, host: &mut H, dx: i32, dy: i32) -> Decision {
        match self.input_target() {
            Target::Remote => {
                host.send(Message::Wheel { dx, dy });
                Decision::Swallow
            }
            Target::Consumed => Decision::Swallow,
            Target::Local => Decision::Pass,
        }
    }

    // ------------------------------------------------------------------
    // Clavier
    // ------------------------------------------------------------------

    pub fn key<H: Host>(&mut self, host: &mut H, code: u32, down: bool) -> Decision {
        let decision = if down {
            self.pressed.insert(code);
            let (target, repeat) = match self.key_targets.get(&code) {
                Some(t) => (*t, true),
                None => {
                    let t = if !keymap::is_modifier(code) && capture::is_waiting() {
                        // Saisie d'un raccourci par l'app de réglages.
                        capture::finish(Hotkey::from_pressed(code, &self.pressed).to_string());
                        Target::Consumed
                    } else {
                        self.new_key_target(host, code)
                    };
                    self.key_targets.insert(code, t);
                    (t, false)
                }
            };
            self.dispatch_key(host, target, code, true, repeat)
        } else {
            let target = self.key_targets.remove(&code).unwrap_or_else(|| self.key_input_target(code));
            self.dispatch_key(host, target, code, false, false)
        };
        if !down {
            self.pressed.remove(&code);
        }
        decision
    }

    fn key_input_target(&self, code: u32) -> Target {
        if keymap::is_media(code) {
            match self.opts.media_keys {
                MediaKeys::Primary => return Target::Local,
                MediaKeys::Secondary if self.connected => return Target::Remote,
                _ => {}
            }
        }
        self.input_target()
    }

    fn new_key_target<H: Host>(&mut self, host: &mut H, code: u32) -> Target {
        if self.mode == Mode::Remote {
            if let Some(hk) = &self.opts.return_hotkey {
                if hk.matches(code, &self.pressed) {
                    log::info!("raccourci : retour sur la machine principale");
                    self.come_back(host);
                    return Target::Consumed;
                }
            }
        } else if self.connected {
            if let Some(hk) = &self.opts.switch_hotkey {
                if hk.matches(code, &self.pressed) {
                    log::info!("raccourci : passage sur l'autre machine");
                    let cur = host.cursor();
                    self.go_remote(host, cur, Message::Jump { exit: self.remote_exit() });
                    return Target::Consumed;
                }
            }
        }
        if let Some(hk) = &self.opts.lock_hotkey {
            if hk.matches(code, &self.pressed) {
                let locked = self.toggle_lock();
                log::info!("curseur {}", if locked { "verrouillé" } else { "déverrouillé" });
                return Target::Consumed;
            }
        }
        self.key_input_target(code)
    }

    fn dispatch_key<H: Host>(&mut self, host: &mut H, target: Target, code: u32, down: bool, repeat: bool) -> Decision {
        match target {
            Target::Local => Decision::Pass,
            Target::Remote => {
                if self.connected {
                    host.send(Message::Key { code, down, repeat });
                }
                Decision::Swallow
            }
            Target::Consumed => Decision::Swallow,
        }
    }

    // ------------------------------------------------------------------
    // Événements de l'autre machine
    // ------------------------------------------------------------------

    pub fn peer_event<H: Host>(&mut self, host: &mut H, ev: PeerEvent) {
        match ev {
            PeerEvent::Connected => {
                self.connected = true;
                if self.mode == Mode::Remote {
                    // Reconnexion pendant le pilotage : l'autre machine (peut-être
                    // redémarrée) reprend le curseur là où il est.
                    log::info!("reconnexion : reprise du pilotage de l'autre machine");
                    host.send(Message::Jump { exit: self.remote_exit() });
                }
            }
            PeerEvent::Disconnected => {
                self.connected = false;
                self.disconnects += 1;
                // Les relâchements des touches envoyées là-bas n'ont plus de
                // destination : on les supprime (le système local n'a jamais vu
                // l'appui correspondant).
                for t in self.key_targets.values_mut() {
                    if *t == Target::Remote {
                        *t = Target::Consumed;
                    }
                }
                for t in self.button_targets.iter_mut().flatten() {
                    if *t == Target::Remote {
                        *t = Target::Consumed;
                    }
                }
                if self.mode == Mode::Remote {
                    log::info!("connexion perdue pendant le pilotage : attente d'une reconnexion");
                }
            }
            PeerEvent::GraceOver(n) => {
                if n == self.disconnects && !self.connected && self.mode == Mode::Remote {
                    log::info!("connexion perdue : retour du curseur sur la machine principale");
                    let p = self.exit_point;
                    self.force_local(host, p);
                }
            }
            PeerEvent::Leave { ratio } => {
                if self.mode != Mode::Remote {
                    return;
                }
                if self.locked {
                    // Verrouillé : on renvoie le curseur sur l'autre machine.
                    host.send(Message::Enter { edge: self.opts.position.opposite(), ratio });
                    return;
                }
                let pos = host.layout().point_on_edge(self.opts.position, ratio, 1);
                log::debug!("retour sur la machine principale en {pos:?}");
                self.force_local(host, pos);
            }
        }
    }

    /// Remise à zéro complète (verrouillage de session, mise en veille…) :
    /// tout est relâché de l'autre côté et le curseur revient ici.
    pub fn reset<H: Host>(&mut self, host: &mut H) {
        if self.connected {
            host.send(Message::ReleaseAll);
        }
        self.key_targets.clear();
        self.button_targets = [None; 5];
        self.pressed.clear();
        if self.mode == Mode::Remote {
            let p = self.exit_point;
            self.force_local(host, p);
        }
    }

    /// Ramène le curseur sur la machine principale (menu de l'icône).
    pub fn return_home<H: Host>(&mut self, host: &mut H) {
        if self.mode == Mode::Remote {
            self.come_back(host);
        }
    }

    /// Retour décidé ici (raccourci, menu) : l'autre machine relâche tout et
    /// sait qu'elle n'est plus pilotée (son icône change).
    fn come_back<H: Host>(&mut self, host: &mut H) {
        if self.connected {
            host.send(Message::ReleaseAll);
        }
        let p = self.exit_point;
        self.force_local(host, p);
    }

    pub fn toggle_lock(&mut self) -> bool {
        self.locked = !self.locked;
        status::set_locked(self.locked);
        self.locked
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Bord par lequel le curseur peut quitter l'autre machine.
    fn remote_exit(&self) -> Option<Edge> {
        self.opts.edge_switching.then(|| self.opts.position.opposite())
    }

    fn force_local<H: Host>(&mut self, host: &mut H, pos: Point) {
        self.mode = Mode::Local;
        self.warp_done = false;
        host.show_cursor(pos);
        status::set_remote_active(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::keymap::*;

    struct FakeHost {
        layout: Layout,
        cursor: Point,
        sent: Vec<Message>,
        hidden_at: Option<Point>,
        shown_at: Option<Point>,
    }

    impl Host for FakeHost {
        fn layout(&self) -> &Layout {
            &self.layout
        }
        fn cursor(&self) -> Point {
            self.cursor
        }
        fn hide_cursor(&mut self, at: Point) {
            self.hidden_at = Some(at);
        }
        fn show_cursor(&mut self, pos: Point) {
            self.shown_at = Some(pos);
        }
        fn send(&mut self, msg: Message) {
            self.sent.push(msg);
        }
    }

    fn setup() -> (PrimaryCore, FakeHost) {
        let mut core = PrimaryCore::new(PrimaryOptions {
            edge_switching: true,
            position: Edge::Right,
            corner: 0,
            switch_hotkey: Hotkey::parse("Shift+F12").unwrap(),
            return_hotkey: Hotkey::parse("Ctrl+Alt+Shift+B").unwrap(),
            lock_hotkey: Hotkey::parse("ScrollLock").unwrap(),
            media_keys: MediaKeys::Cursor,
        });
        let mut host = FakeHost {
            layout: Layout::new(vec![Rect::new(0, 0, 1920, 1080)]),
            cursor: Point::new(700, 300),
            sent: vec![],
            hidden_at: None,
            shown_at: None,
        };
        core.peer_event(&mut host, PeerEvent::Connected);
        (core, host)
    }

    fn go_remote(core: &mut PrimaryCore, host: &mut FakeHost) {
        let d = core.local_motion(host, Point::new(1919, 540), Point::new(1925, 540));
        assert_eq!(d, Decision::Swallow);
        assert!(core.is_remote());
        core.on_warped(Point::new(960, 540));
    }

    #[test]
    fn switch_and_return() {
        let (mut core, mut host) = setup();
        assert_eq!(core.local_motion(&mut host, Point::new(100, 100), Point::new(105, 100)), Decision::Pass);
        go_remote(&mut core, &mut host);
        assert!(matches!(host.sent[0], Message::Enter { edge: Edge::Left, .. }));
        assert_eq!(host.hidden_at, Some(Point::new(1919, 540)));

        core.remote_abs_motion(&mut host, Point::new(965, 538));
        assert_eq!(host.sent.last(), Some(&Message::Motion { dx: 5, dy: -2 }));
        // Artefact du recentrage : ignoré.
        let n = host.sent.len();
        core.remote_abs_motion(&mut host, Point::new(1925, 540));
        assert_eq!(host.sent.len(), n);

        core.peer_event(&mut host, PeerEvent::Leave { ratio: 0.5 });
        assert!(!core.is_remote());
        let p = host.shown_at.unwrap();
        assert_eq!(p.x, 1918);
        assert!((p.y - 540).abs() <= 1);
    }

    #[test]
    fn no_switch_while_dragging_or_disconnected() {
        let (mut core, mut host) = setup();
        assert_eq!(core.button(&mut host, MouseButton::Left, true), Decision::Pass);
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        assert_eq!(core.button(&mut host, MouseButton::Left, false), Decision::Pass);
        core.peer_event(&mut host, PeerEvent::Disconnected);
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
    }

    #[test]
    fn key_release_follows_press() {
        let (mut core, mut host) = setup();
        let a = hid_from_name("A").unwrap();
        // Shift appuyé en local, puis passage sur l'autre machine.
        assert_eq!(core.key(&mut host, HID_SHIFT_LEFT, true), Decision::Pass);
        go_remote(&mut core, &mut host);
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Key { code: a, down: true, repeat: false }));
        // Répétition automatique.
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Key { code: a, down: true, repeat: true }));
        // Le relâchement de Shift reste local.
        assert_eq!(core.key(&mut host, HID_SHIFT_LEFT, false), Decision::Pass);
        // Retour local alors que A est encore enfoncé : son relâchement va là-bas.
        core.peer_event(&mut host, PeerEvent::Leave { ratio: 0.5 });
        assert_eq!(core.key(&mut host, a, false), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Key { code: a, down: false, repeat: false }));
    }

    #[test]
    fn return_hotkey() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        let b = hid_from_name("B").unwrap();
        for k in [HID_CONTROL_LEFT, HID_ALT_LEFT, HID_SHIFT_LEFT] {
            assert_eq!(core.key(&mut host, k, true), Decision::Swallow);
        }
        let n = host.sent.len();
        assert_eq!(core.key(&mut host, b, true), Decision::Swallow);
        assert!(!core.is_remote());
        // La touche du raccourci n'est pas transmise ; l'autre machine est
        // prévenue qu'elle n'est plus pilotée.
        assert_eq!(host.sent[n..], [Message::ReleaseAll]);
        assert_eq!(host.shown_at, Some(Point::new(1919, 540)));
        // Relâchements : B est consommée, les modificateurs vont là-bas.
        assert_eq!(core.key(&mut host, b, false), Decision::Swallow);
        assert_eq!(host.sent.len(), n + 1);
        assert_eq!(core.key(&mut host, HID_CONTROL_LEFT, false), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Key { code: HID_CONTROL_LEFT, down: false, repeat: false }));
    }

    #[test]
    fn disconnect_returns_cursor_and_swallows_pending_releases() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        let a = hid_from_name("A").unwrap();
        core.key(&mut host, a, true);
        core.button(&mut host, MouseButton::Right, true);
        core.peer_event(&mut host, PeerEvent::Disconnected);
        core.peer_event(&mut host, PeerEvent::GraceOver(1));
        assert!(!core.is_remote());
        assert_eq!(core.key(&mut host, a, false), Decision::Swallow);
        assert_eq!(core.button(&mut host, MouseButton::Right, false), Decision::Swallow);
        // Ensuite tout redevient local.
        assert_eq!(core.key(&mut host, a, true), Decision::Pass);
    }

    #[test]
    fn short_disconnect_resumes_remote_control() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        host.sent.clear();
        core.peer_event(&mut host, PeerEvent::Disconnected);
        // Pendant la coupure : le curseur reste là-bas, rien n'arrive ici.
        assert!(core.is_remote());
        let a = hid_from_name("A").unwrap();
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(core.key(&mut host, a, false), Decision::Swallow);
        assert_eq!(core.button(&mut host, MouseButton::Left, true), Decision::Swallow);
        assert_eq!(core.button(&mut host, MouseButton::Left, false), Decision::Swallow);
        assert_eq!(core.wheel(&mut host, 0, 120), Decision::Swallow);
        assert!(host.sent.is_empty());
        // Reconnexion : l'autre machine reprend le curseur.
        core.peer_event(&mut host, PeerEvent::Connected);
        assert!(core.is_remote());
        assert_eq!(host.sent, [Message::Jump { exit: Some(Edge::Left) }]);
        // Le délai de la coupure passée n'a plus d'effet.
        core.peer_event(&mut host, PeerEvent::GraceOver(1));
        assert!(core.is_remote());
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Key { code: a, down: true, repeat: false }));
    }

    #[test]
    fn return_hotkey_during_disconnect() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        core.peer_event(&mut host, PeerEvent::Disconnected);
        let b = hid_from_name("B").unwrap();
        press(&mut core, &mut host, &[HID_CONTROL_LEFT, HID_ALT_LEFT, HID_SHIFT_LEFT, b]);
        assert!(!core.is_remote());
        assert_eq!(host.shown_at, Some(Point::new(1919, 540)));
    }

    #[test]
    fn lock_prevents_switching() {
        let (mut core, mut host) = setup();
        assert_eq!(core.key(&mut host, HID_SCROLL_LOCK, true), Decision::Swallow);
        assert_eq!(core.key(&mut host, HID_SCROLL_LOCK, false), Decision::Swallow);
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        core.key(&mut host, HID_SCROLL_LOCK, true);
        core.key(&mut host, HID_SCROLL_LOCK, false);
        go_remote(&mut core, &mut host);
        // Verrouillé pendant le pilotage : un « Leave » renvoie le curseur là-bas.
        core.key(&mut host, HID_SCROLL_LOCK, true);
        core.peer_event(&mut host, PeerEvent::Leave { ratio: 0.3 });
        assert!(core.is_remote());
        assert_eq!(host.sent.last(), Some(&Message::Enter { edge: Edge::Left, ratio: 0.3 }));
    }

    fn press(core: &mut PrimaryCore, host: &mut FakeHost, keys: &[u32]) -> Vec<Decision> {
        let mut d: Vec<Decision> = keys.iter().map(|k| core.key(host, *k, true)).collect();
        d.extend(keys.iter().rev().map(|k| core.key(host, *k, false)));
        d
    }

    #[test]
    fn switch_hotkey_jumps_and_returns_in_place() {
        let (mut core, mut host) = setup();
        let f12 = hid_from_name("F12").unwrap();
        let d = press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        // Maj reste locale (appui et relâchement), F12 est consommée.
        assert_eq!(d, [Decision::Pass, Decision::Swallow, Decision::Swallow, Decision::Pass]);
        assert!(core.is_remote());
        assert_eq!(host.sent, [Message::Jump { exit: Some(Edge::Left) }]);
        assert_eq!(host.hidden_at, Some(Point::new(700, 300)));
        core.on_warped(Point::new(960, 540));

        // Retour : le curseur réapparaît là où il était.
        let b = hid_from_name("B").unwrap();
        press(&mut core, &mut host, &[HID_CONTROL_LEFT, HID_ALT_LEFT, HID_SHIFT_LEFT, b]);
        assert!(!core.is_remote());
        assert_eq!(host.shown_at, Some(Point::new(700, 300)));
    }

    #[test]
    fn hotkey_capture_for_settings() {
        let (mut core, mut host) = setup();
        capture::set_supported(true);
        assert!(capture::start());
        // Shift+F12 est relevée au lieu de faire basculer, et n'arrive pas
        // aux applications.
        let f12 = hid_from_name("F12").unwrap();
        let d = press(&mut core, &mut host, &[HID_SHIFT_RIGHT, f12]);
        assert_eq!(d, [Decision::Pass, Decision::Swallow, Decision::Swallow, Decision::Pass]);
        assert!(!core.is_remote());
        assert_eq!(capture::poll(), capture::State::Done("Shift+F12".into()));
        // Ensuite, le raccourci fonctionne de nouveau.
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
    }

    #[test]
    fn same_hotkey_toggles() {
        let (mut core, mut host) = setup();
        core.opts.return_hotkey = core.opts.switch_hotkey.clone();
        let f12 = hid_from_name("F12").unwrap();
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(!core.is_remote());
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
    }

    #[test]
    fn switch_hotkey_passes_through_when_disconnected() {
        let (mut core, mut host) = setup();
        core.peer_event(&mut host, PeerEvent::Disconnected);
        let f12 = hid_from_name("F12").unwrap();
        let d = press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert_eq!(d, [Decision::Pass; 4]);
        assert!(!core.is_remote());
        assert!(host.sent.is_empty());
    }

    #[test]
    fn edge_switching_disabled() {
        let (mut core, mut host) = setup();
        core.opts.edge_switching = false;
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        assert!(!core.is_remote());
        // Le raccourci fonctionne, sans bord de sortie pour la machine secondaire.
        let f12 = hid_from_name("F12").unwrap();
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
        assert_eq!(host.sent, [Message::Jump { exit: None }]);
    }

    #[test]
    fn media_keys_on_secondary_even_when_local() {
        let (mut core, mut host) = setup();
        core.opts.media_keys = MediaKeys::Secondary;
        assert!(!core.is_remote());
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Key { code: HID_VOLUME_UP, down: true, repeat: false }));
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, false), Decision::Swallow);
        // Les autres touches restent locales.
        assert_eq!(core.key(&mut host, hid_from_name("A").unwrap(), true), Decision::Pass);
        // Sans connexion, elles agissent ici.
        core.peer_event(&mut host, PeerEvent::Disconnected);
        assert_eq!(core.key(&mut host, HID_VOLUME_DOWN, true), Decision::Pass);
    }

    #[test]
    fn media_keys_local_option() {
        let (mut core, mut host) = setup();
        core.opts.media_keys = MediaKeys::Primary;
        go_remote(&mut core, &mut host);
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, true), Decision::Pass);
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, false), Decision::Pass);
        assert_eq!(core.wheel(&mut host, 0, 120), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&Message::Wheel { dx: 0, dy: 120 }));
    }
}
