//! Logique de la machine pilotée, indépendante du système : position du
//! curseur, détection de la sortie par le bord, remappage des touches.
//! L'injection réelle des événements est faite par l'`Emulator` de la
//! plateforme.

use crate::geometry::{Layout, Point, Step};
use crate::protocol::{Edge, Message, MouseButton};
use crate::status;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Injection des événements dans le système local.
pub trait Emulator {
    /// Écrans locaux.
    fn layout(&mut self) -> Layout;
    /// Déplace le curseur en `p` (`dx`, `dy` : déplacement correspondant).
    fn move_to(&mut self, p: Point, dx: i32, dy: i32);
    fn button(&mut self, button: MouseButton, down: bool);
    /// Molette en unités Windows (120 = un cran), `dy > 0` vers le haut,
    /// `dx > 0` vers la droite.
    fn wheel(&mut self, dx: i32, dy: i32);
    /// Touche physique (code HID).
    fn key(&mut self, code: u32, down: bool, repeat: bool);
    /// Relâche tout ce qui est enfoncé.
    fn release_all(&mut self);
    /// Le curseur arrive sur cette machine (réveil de l'écran…).
    fn entered(&mut self) {}
    /// Position réelle du curseur, si disponible.
    fn cursor(&mut self) -> Option<Point> {
        None
    }
    /// L'utilisateur est actif sur la machine principale : repousse la veille
    /// de l'écran, l'économiseur et le statut « absent » des messageries.
    fn keep_active(&mut self) {}
}

/// Dernier événement injecté par Bascule sur cette machine : ce qui suit
/// vient de son propre clavier ou trackpad.
static INJECTED_AT: Mutex<Option<Instant>> = Mutex::new(None);

pub fn note_injected() {
    *INJECTED_AT.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
}

pub fn injected_at() -> Option<Instant> {
    *INJECTED_AT.lock().unwrap_or_else(|e| e.into_inner())
}

/// Si le curseur a été déplacé localement (trackpad du Mac…) depuis notre
/// dernier déplacement, on repart de sa position réelle.
const RESYNC_DISTANCE: i32 = 8;
const RESYNC_DELAY: Duration = Duration::from_millis(250);

pub struct SecondaryCore {
    active: bool,
    /// Bord par lequel le curseur repart (`None` : il ne repart que par
    /// raccourci clavier).
    exit: Option<Edge>,
    pos: Point,
    speed: f64,
    acc_x: f64,
    acc_y: f64,
    buttons: [bool; 5],
    remap: BTreeMap<u32, u32>,
    layout: Layout,
    layout_at: Option<Instant>,
    last_move: Option<Instant>,
}

impl SecondaryCore {
    pub fn new(speed: f64, remap: BTreeMap<u32, u32>) -> SecondaryCore {
        SecondaryCore {
            active: false,
            exit: None,
            pos: Point::default(),
            speed,
            acc_x: 0.0,
            acc_y: 0.0,
            buttons: [false; 5],
            remap,
            layout: Layout::default(),
            layout_at: None,
            last_move: None,
        }
    }

    fn resync<E: Emulator>(&mut self, emu: &mut E) {
        let idle = self.last_move.is_none_or(|t| t.elapsed() > RESYNC_DELAY);
        if !idle {
            return;
        }
        if let Some(real) = emu.cursor() {
            if (real.x - self.pos.x).abs() > RESYNC_DISTANCE || (real.y - self.pos.y).abs() > RESYNC_DISTANCE {
                if self.layout.display_at(real).is_some() {
                    log::debug!("curseur déplacé localement : {:?} → {:?}", self.pos, real);
                    self.pos = real;
                }
            }
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn refresh_layout<E: Emulator>(&mut self, emu: &mut E, force: bool) {
        let stale = self.layout_at.is_none_or(|t| t.elapsed() > Duration::from_secs(1));
        if force || stale {
            let l = emu.layout();
            if l != self.layout {
                log::info!("écrans : {:?}", l.displays);
            }
            self.layout = l;
            self.layout_at = Some(Instant::now());
        }
    }

    /// Traite un message de la machine principale. Renvoie éventuellement un
    /// message à lui renvoyer.
    pub fn handle<E: Emulator>(&mut self, emu: &mut E, msg: Message) -> Option<Message> {
        match msg {
            Message::Enter { edge, ratio } => {
                self.refresh_layout(emu, true);
                let p = self.layout.point_on_edge(edge, ratio, 0);
                log::debug!("le curseur arrive par {edge:?} en {p:?}");
                self.activate(emu, p, Some(edge));
                None
            }
            Message::Jump { exit } => {
                self.refresh_layout(emu, true);
                // Le curseur reprend sa position réelle (ou la dernière connue).
                let p = emu.cursor().unwrap_or(self.pos);
                let p = if self.layout.display_at(p).is_some() { p } else { self.layout.nearest_display(p).center() };
                log::debug!("le curseur arrive par raccourci en {p:?}");
                self.activate(emu, p, exit);
                None
            }
            Message::Motion { dx, dy } => {
                if !self.active {
                    return None;
                }
                self.refresh_layout(emu, false);
                self.resync(emu);
                let fx = dx as f64 * self.speed + self.acc_x;
                let fy = dy as f64 * self.speed + self.acc_y;
                let ix = fx.trunc();
                let iy = fy.trunc();
                self.acc_x = fx - ix;
                self.acc_y = fy - iy;
                let (ix, iy) = (ix as i32, iy as i32);
                if ix == 0 && iy == 0 {
                    return None;
                }
                // Pas de sortie pendant un glisser (bouton enfoncé).
                let exit = if self.buttons.iter().any(|b| *b) { None } else { self.exit };
                match self.layout.step(self.pos, ix, iy, exit) {
                    Step::Move(p) => {
                        let (mdx, mdy) = (p.x - self.pos.x, p.y - self.pos.y);
                        self.pos = p;
                        self.last_move = Some(Instant::now());
                        // Même immobile contre un bord, on transmet le
                        // déplacement demandé (utile pour le Dock, les jeux…).
                        emu.move_to(p, if mdx == 0 { ix } else { mdx }, if mdy == 0 { iy } else { mdy });
                        None
                    }
                    Step::Exit(ratio) => {
                        self.active = false;
                        log::debug!("le curseur repart (ratio {ratio:.3})");
                        status::set_remote_active(false);
                        Some(Message::Leave { ratio })
                    }
                }
            }
            Message::Button { button, down } => {
                self.buttons[button.index()] = down;
                emu.button(button, down);
                None
            }
            Message::Wheel { dx, dy } => {
                emu.wheel(dx, dy);
                None
            }
            Message::Key { code, down, repeat } => {
                let code = self.remap.get(&code).copied().unwrap_or(code);
                emu.key(code, down, repeat);
                None
            }
            // La machine principale reprend la main (raccourci, menu,
            // verrouillage de session…) : tout est relâché ici.
            Message::ReleaseAll => {
                self.buttons = [false; 5];
                emu.release_all();
                if self.active {
                    self.active = false;
                    status::set_remote_active(false);
                }
                None
            }
            _ => None,
        }
    }

    fn activate<E: Emulator>(&mut self, emu: &mut E, p: Point, exit: Option<Edge>) {
        self.exit = exit;
        self.pos = p;
        self.acc_x = 0.0;
        self.acc_y = 0.0;
        self.active = true;
        self.last_move = Some(Instant::now());
        emu.entered();
        emu.move_to(p, 0, 0);
        status::set_remote_active(true);
    }

    /// Connexion perdue : on relâche tout.
    pub fn disconnected<E: Emulator>(&mut self, emu: &mut E) {
        self.active = false;
        self.buttons = [false; 5];
        emu.release_all();
        status::set_remote_active(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[derive(Default)]
    struct FakeEmu {
        cursor: Option<Point>,
        moves: Vec<(Point, i32, i32)>,
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(MouseButton, bool)>,
        released: usize,
    }

    impl Emulator for FakeEmu {
        fn layout(&mut self) -> Layout {
            Layout::new(vec![Rect::new(0, 0, 1440, 900)])
        }
        fn move_to(&mut self, p: Point, dx: i32, dy: i32) {
            self.moves.push((p, dx, dy));
        }
        fn button(&mut self, button: MouseButton, down: bool) {
            self.buttons.push((button, down));
        }
        fn wheel(&mut self, _dx: i32, _dy: i32) {}
        fn key(&mut self, code: u32, down: bool, repeat: bool) {
            self.keys.push((code, down, repeat));
        }
        fn release_all(&mut self) {
            self.released += 1;
        }
        fn cursor(&mut self) -> Option<Point> {
            self.cursor
        }
    }

    #[test]
    fn enter_move_and_leave() {
        let mut emu = FakeEmu::default();
        let mut core = SecondaryCore::new(1.0, BTreeMap::new());
        assert_eq!(core.handle(&mut emu, Message::Enter { edge: Edge::Left, ratio: 0.5 }), None);
        assert_eq!(emu.moves.last().unwrap().0, Point::new(0, 450));
        core.handle(&mut emu, Message::Motion { dx: 10, dy: 0 });
        assert_eq!(emu.moves.last().unwrap().0, Point::new(10, 450));
        // Retour vers la gauche : sortie.
        let r = core.handle(&mut emu, Message::Motion { dx: -20, dy: 0 });
        assert!(matches!(r, Some(Message::Leave { .. })));
        assert!(!core.is_active());
        // Les déplacements suivants sont ignorés jusqu'au prochain Enter.
        let n = emu.moves.len();
        core.handle(&mut emu, Message::Motion { dx: 5, dy: 5 });
        assert_eq!(emu.moves.len(), n);
    }

    #[test]
    fn jump_resumes_at_cursor_position() {
        let mut emu = FakeEmu { cursor: Some(Point::new(800, 200)), ..Default::default() };
        let mut core = SecondaryCore::new(1.0, BTreeMap::new());
        core.handle(&mut emu, Message::Jump { exit: Some(Edge::Left) });
        assert!(core.is_active());
        assert_eq!(emu.moves.last().unwrap().0, Point::new(800, 200));
        core.handle(&mut emu, Message::Motion { dx: -10, dy: 5 });
        assert_eq!(emu.moves.last().unwrap().0, Point::new(790, 205));
        // Sortie par la gauche, comme après un passage par le bord.
        assert!(matches!(core.handle(&mut emu, Message::Motion { dx: -900, dy: 0 }), Some(Message::Leave { .. })));
    }

    #[test]
    fn jump_without_exit_never_leaves() {
        let mut emu = FakeEmu::default();
        let mut core = SecondaryCore::new(1.0, BTreeMap::new());
        core.handle(&mut emu, Message::Enter { edge: Edge::Left, ratio: 0.5 });
        core.handle(&mut emu, Message::Motion { dx: 100, dy: 0 });
        // Position réelle inconnue : dernière position connue.
        core.handle(&mut emu, Message::Jump { exit: None });
        assert_eq!(emu.moves.last().unwrap().0, Point::new(100, 450));
        for _ in 0..3 {
            assert_eq!(core.handle(&mut emu, Message::Motion { dx: -1000, dy: 0 }), None);
        }
        assert!(core.is_active());
        assert_eq!(emu.moves.last().unwrap().0, Point::new(0, 450));
    }

    #[test]
    fn no_leave_while_dragging() {
        let mut emu = FakeEmu::default();
        let mut core = SecondaryCore::new(1.0, BTreeMap::new());
        core.handle(&mut emu, Message::Enter { edge: Edge::Left, ratio: 0.5 });
        core.handle(&mut emu, Message::Button { button: MouseButton::Left, down: true });
        assert_eq!(core.handle(&mut emu, Message::Motion { dx: -50, dy: 0 }), None);
        assert_eq!(emu.moves.last().unwrap().0, Point::new(0, 450));
        core.handle(&mut emu, Message::Button { button: MouseButton::Left, down: false });
        assert!(core.handle(&mut emu, Message::Motion { dx: -5, dy: 0 }).is_some());
    }

    #[test]
    fn speed_accumulates_fractions() {
        let mut emu = FakeEmu::default();
        let mut core = SecondaryCore::new(0.5, BTreeMap::new());
        core.handle(&mut emu, Message::Enter { edge: Edge::Left, ratio: 0.5 });
        for _ in 0..10 {
            core.handle(&mut emu, Message::Motion { dx: 1, dy: 0 });
        }
        assert_eq!(emu.moves.last().unwrap().0, Point::new(5, 450));
    }

    #[test]
    fn remap_applies() {
        let mut emu = FakeEmu::default();
        let mut remap = BTreeMap::new();
        remap.insert(0x070039, 0x0700e0);
        let mut core = SecondaryCore::new(1.0, remap);
        core.handle(&mut emu, Message::Key { code: 0x070039, down: true, repeat: false });
        core.handle(&mut emu, Message::Key { code: 0x070004, down: true, repeat: false });
        assert_eq!(emu.keys, vec![(0x0700e0, true, false), (0x070004, true, false)]);
    }

    #[test]
    fn release_all_ends_remote_control() {
        let mut emu = FakeEmu::default();
        let mut core = SecondaryCore::new(1.0, BTreeMap::new());
        core.handle(&mut emu, Message::Jump { exit: None });
        assert!(core.is_active());
        core.handle(&mut emu, Message::ReleaseAll);
        assert!(!core.is_active());
        // Les déplacements arrivés trop tard sont ignorés.
        let n = emu.moves.len();
        core.handle(&mut emu, Message::Motion { dx: 5, dy: 5 });
        assert_eq!(emu.moves.len(), n);
    }

    #[test]
    fn disconnect_releases() {
        let mut emu = FakeEmu::default();
        let mut core = SecondaryCore::new(1.0, BTreeMap::new());
        core.handle(&mut emu, Message::Enter { edge: Edge::Left, ratio: 0.5 });
        core.disconnected(&mut emu);
        assert_eq!(emu.released, 1);
        assert!(!core.is_active());
    }
}
