//! Géométrie des écrans : détection des bords, conversion des positions entre
//! les deux machines.
//!
//! Les coordonnées suivent la convention des deux systèmes : origine en haut à
//! gauche de l'écran principal, y vers le bas. Windows travaille en pixels
//! physiques (processus « per-monitor DPI aware »), macOS en points.

use crate::protocol::Edge;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

/// Rectangle, bornes droite et basse exclusives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect { left, top, right, bottom }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.left && p.x < self.right && p.y >= self.top && p.y < self.bottom
    }

    pub fn is_empty(&self) -> bool {
        self.width() <= 0 || self.height() <= 0
    }

    pub fn clamp(&self, p: Point) -> Point {
        Point { x: p.x.clamp(self.left, self.right - 1), y: p.y.clamp(self.top, self.bottom - 1) }
    }

    pub fn center(&self) -> Point {
        Point { x: self.left + self.width() / 2, y: self.top + self.height() / 2 }
    }

    /// Distance (carrée) entre un point et le rectangle.
    fn distance2(&self, p: Point) -> i64 {
        let c = self.clamp(p);
        let dx = (p.x - c.x) as i64;
        let dy = (p.y - c.y) as i64;
        dx * dx + dy * dy
    }

    fn union(&self, o: &Rect) -> Rect {
        Rect {
            left: self.left.min(o.left),
            top: self.top.min(o.top),
            right: self.right.max(o.right),
            bottom: self.bottom.max(o.bottom),
        }
    }
}

/// Ensemble des écrans d'une machine.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub displays: Vec<Rect>,
}

/// Résultat d'un déplacement sur la machine secondaire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// Nouvelle position (déjà ramenée sur un écran).
    Move(Point),
    /// Le curseur sort par le bord demandé, à cette position relative.
    Exit(f32),
}

impl Layout {
    pub fn new(displays: Vec<Rect>) -> Layout {
        let displays: Vec<Rect> = displays.into_iter().filter(|r| !r.is_empty()).collect();
        Layout { displays }
    }

    pub fn is_empty(&self) -> bool {
        self.displays.is_empty()
    }

    /// Rectangle englobant tous les écrans.
    pub fn bounds(&self) -> Rect {
        let mut it = self.displays.iter();
        match it.next() {
            None => Rect::new(0, 0, 1, 1),
            Some(first) => it.fold(*first, |acc, r| acc.union(r)),
        }
    }

    pub fn display_at(&self, p: Point) -> Option<&Rect> {
        self.displays.iter().find(|r| r.contains(p))
    }

    pub fn nearest_display(&self, p: Point) -> Rect {
        self.displays.iter().min_by_key(|r| r.distance2(p)).copied().unwrap_or_else(|| self.bounds())
    }

    /// Ramène un point sur l'écran le plus proche.
    pub fn clamp(&self, p: Point) -> Point {
        if self.display_at(p).is_some() {
            return p;
        }
        self.nearest_display(p).clamp(p)
    }

    /// Le déplacement de `cur` (position réelle du curseur) vers `next`
    /// (position demandée, éventuellement hors écran) pousse-t-il contre le
    /// bord extérieur `edge` ?
    ///
    /// `corner` : taille des coins (en pixels) où la bascule est ignorée.
    pub fn pushes_against(&self, cur: Point, next: Point, edge: Edge, corner: i32) -> bool {
        if self.is_empty() {
            return false;
        }
        let d = match self.display_at(cur) {
            Some(d) => *d,
            None => self.nearest_display(cur),
        };
        // Position projetée sur l'écran courant, pour tester la présence d'un
        // écran voisin et les coins.
        let c = d.clamp(next);
        let corner = corner.max(0);
        // Si le système a déjà ramené la position demandée sur le bord
        // (next == cur, curseur collé au bord), on considère aussi que
        // l'utilisateur pousse contre ce bord.
        let stuck = next == cur;
        match edge {
            Edge::Right => {
                next.x >= d.right - 1
                    && (next.x > cur.x || (stuck && cur.x >= d.right - 1))
                    && self.display_at(Point::new(d.right, c.y)).is_none()
                    && c.y >= d.top + corner
                    && c.y < d.bottom - corner
            }
            Edge::Left => {
                next.x <= d.left
                    && (next.x < cur.x || (stuck && cur.x <= d.left))
                    && self.display_at(Point::new(d.left - 1, c.y)).is_none()
                    && c.y >= d.top + corner
                    && c.y < d.bottom - corner
            }
            Edge::Bottom => {
                next.y >= d.bottom - 1
                    && (next.y > cur.y || (stuck && cur.y >= d.bottom - 1))
                    && self.display_at(Point::new(c.x, d.bottom)).is_none()
                    && c.x >= d.left + corner
                    && c.x < d.right - corner
            }
            Edge::Top => {
                next.y <= d.top
                    && (next.y < cur.y || (stuck && cur.y <= d.top))
                    && self.display_at(Point::new(c.x, d.top - 1)).is_none()
                    && c.x >= d.left + corner
                    && c.x < d.right - corner
            }
        }
    }

    /// Position relative (0..=1) d'un point le long d'un bord, mesurée sur le
    /// rectangle englobant.
    pub fn ratio_on_edge(&self, p: Point, edge: Edge) -> f32 {
        let b = self.bounds();
        let r = if edge.is_horizontal_axis() {
            (p.y - b.top) as f64 / (b.height().max(1) - 1).max(1) as f64
        } else {
            (p.x - b.left) as f64 / (b.width().max(1) - 1).max(1) as f64
        };
        r.clamp(0.0, 1.0) as f32
    }

    /// Point situé sur le bord extérieur `edge`, à la position relative
    /// `ratio`, décalé de `inset` pixels vers l'intérieur.
    pub fn point_on_edge(&self, edge: Edge, ratio: f32, inset: i32) -> Point {
        let b = self.bounds();
        let ratio = (ratio as f64).clamp(0.0, 1.0);
        if self.is_empty() {
            return b.center();
        }
        if edge.is_horizontal_axis() {
            let y = b.top + ((b.height() - 1).max(0) as f64 * ratio).round() as i32;
            // Écrans qui couvrent cette ligne ; sinon le plus proche verticalement.
            let candidates: Vec<&Rect> = self.displays.iter().filter(|r| y >= r.top && y < r.bottom).collect();
            let d = if candidates.is_empty() {
                self.nearest_display(Point::new(if edge == Edge::Left { b.left } else { b.right - 1 }, y))
            } else if edge == Edge::Left {
                **candidates.iter().min_by_key(|r| r.left).unwrap()
            } else {
                **candidates.iter().max_by_key(|r| r.right).unwrap()
            };
            let x = if edge == Edge::Left { d.left + inset } else { d.right - 1 - inset };
            d.clamp(Point::new(x, y))
        } else {
            let x = b.left + ((b.width() - 1).max(0) as f64 * ratio).round() as i32;
            let candidates: Vec<&Rect> = self.displays.iter().filter(|r| x >= r.left && x < r.right).collect();
            let d = if candidates.is_empty() {
                self.nearest_display(Point::new(x, if edge == Edge::Top { b.top } else { b.bottom - 1 }))
            } else if edge == Edge::Top {
                **candidates.iter().min_by_key(|r| r.top).unwrap()
            } else {
                **candidates.iter().max_by_key(|r| r.bottom).unwrap()
            };
            let y = if edge == Edge::Top { d.top + inset } else { d.bottom - 1 - inset };
            d.clamp(Point::new(x, y))
        }
    }

    /// Applique un déplacement sur la machine secondaire. Si le curseur
    /// franchit le bord `exit_edge` là où aucun écran ne continue, renvoie
    /// `Step::Exit` ; sinon renvoie la nouvelle position ramenée sur un écran.
    pub fn step(&self, cur: Point, dx: i32, dy: i32, exit_edge: Option<Edge>) -> Step {
        let next = Point::new(cur.x.saturating_add(dx), cur.y.saturating_add(dy));
        if self.display_at(next).is_some() {
            return Step::Move(next);
        }
        if self.is_empty() {
            return Step::Move(cur);
        }
        let d = match self.display_at(cur) {
            Some(d) => *d,
            None => self.nearest_display(cur),
        };
        if let Some(edge) = exit_edge {
            let crossed = match edge {
                Edge::Left => next.x < d.left,
                Edge::Right => next.x >= d.right,
                Edge::Top => next.y < d.top,
                Edge::Bottom => next.y >= d.bottom,
            };
            if crossed {
                // Aucun écran à cet endroit (sinon display_at(next) aurait réussi) :
                // vérifie aussi au droit du bord, pour les déplacements en diagonale.
                let c = d.clamp(next);
                let beyond = match edge {
                    Edge::Left => Point::new(d.left - 1, c.y),
                    Edge::Right => Point::new(d.right, c.y),
                    Edge::Top => Point::new(c.x, d.top - 1),
                    Edge::Bottom => Point::new(c.x, d.bottom),
                };
                if self.display_at(beyond).is_none() {
                    return Step::Exit(self.ratio_on_edge(c, edge));
                }
            }
        }
        // Pas de sortie : on reste collé au bord. Si le point déborde vers un
        // écran voisin en diagonale, on prend l'écran le plus proche.
        Step::Move(self.clamp(next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single() -> Layout {
        Layout::new(vec![Rect::new(0, 0, 1920, 1080)])
    }

    #[test]
    fn push_right_edge() {
        let l = single();
        assert!(l.pushes_against(Point::new(1919, 500), Point::new(1925, 500), Edge::Right, 0));
        assert!(l.pushes_against(Point::new(1910, 500), Point::new(1919, 500), Edge::Right, 0));
        assert!(!l.pushes_against(Point::new(1910, 500), Point::new(1915, 500), Edge::Right, 0));
        // Mouvement vers la gauche le long du bord : pas de bascule.
        assert!(!l.pushes_against(Point::new(1919, 500), Point::new(1919, 510), Edge::Right, 0));
        // Mauvais bord.
        assert!(!l.pushes_against(Point::new(1919, 500), Point::new(1925, 500), Edge::Left, 0));
        // Position déjà ramenée sur le bord par le système.
        assert!(l.pushes_against(Point::new(1919, 500), Point::new(1919, 500), Edge::Right, 0));
        assert!(!l.pushes_against(Point::new(1000, 500), Point::new(1000, 500), Edge::Right, 0));
    }

    #[test]
    fn corners_are_ignored() {
        let l = single();
        assert!(!l.pushes_against(Point::new(1919, 5), Point::new(1925, 5), Edge::Right, 20));
        assert!(l.pushes_against(Point::new(1919, 50), Point::new(1925, 50), Edge::Right, 20));
    }

    #[test]
    fn no_switch_between_two_monitors() {
        // Deux écrans côte à côte : le bord droit du premier n'est pas extérieur.
        let l = Layout::new(vec![Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 3840, 1080)]);
        assert!(!l.pushes_against(Point::new(1919, 500), Point::new(1925, 500), Edge::Right, 0));
        assert!(l.pushes_against(Point::new(3839, 500), Point::new(3845, 500), Edge::Right, 0));
    }

    #[test]
    fn uneven_monitors() {
        // Écran secondaire plus petit à droite, aligné en haut.
        let l = Layout::new(vec![Rect::new(0, 0, 1920, 1080), Rect::new(1920, 0, 3200, 720)]);
        // À y = 900, il n'y a rien à droite du premier écran : bord extérieur.
        assert!(l.pushes_against(Point::new(1919, 900), Point::new(1925, 900), Edge::Right, 0));
        assert!(!l.pushes_against(Point::new(1919, 300), Point::new(1925, 300), Edge::Right, 0));
    }

    #[test]
    fn ratio_roundtrip() {
        let a = single();
        let b = Layout::new(vec![Rect::new(0, 0, 1440, 900)]);
        let r = a.ratio_on_edge(Point::new(1919, 540), Edge::Right);
        let p = b.point_on_edge(Edge::Left, r, 0);
        assert_eq!(p.x, 0);
        assert!((p.y - 450).abs() <= 1);
        assert_eq!(b.point_on_edge(Edge::Left, 0.0, 0), Point::new(0, 0));
        assert_eq!(b.point_on_edge(Edge::Left, 1.0, 0), Point::new(0, 899));
        assert_eq!(b.point_on_edge(Edge::Right, 1.0, 2), Point::new(1437, 899));
    }

    #[test]
    fn step_exit_left() {
        let l = Layout::new(vec![Rect::new(0, 0, 1440, 900)]);
        assert_eq!(l.step(Point::new(5, 100), -3, 0, Some(Edge::Left)), Step::Move(Point::new(2, 100)));
        match l.step(Point::new(2, 450), -5, 0, Some(Edge::Left)) {
            Step::Exit(r) => assert!((r - 0.5).abs() < 0.01),
            s => panic!("{s:?}"),
        }
        // Le bord droit n'est pas une sortie : on reste collé.
        assert_eq!(l.step(Point::new(1438, 100), 10, 0, Some(Edge::Left)), Step::Move(Point::new(1439, 100)));
        // Sans bord de sortie (bouton enfoncé), on reste collé à gauche.
        assert_eq!(l.step(Point::new(2, 450), -5, 0, None), Step::Move(Point::new(0, 450)));
    }

    #[test]
    fn step_across_monitors() {
        // Deux écrans Mac : l'externe à gauche du MacBook.
        let l = Layout::new(vec![Rect::new(0, 0, 1512, 982), Rect::new(-2560, -200, 0, 1240)]);
        assert_eq!(l.step(Point::new(3, 500), -10, 0, Some(Edge::Left)), Step::Move(Point::new(-7, 500)));
        assert!(matches!(l.step(Point::new(-2558, 500), -10, 0, Some(Edge::Left)), Step::Exit(_)));
    }

    #[test]
    fn point_on_edge_picks_outer_display() {
        let l = Layout::new(vec![Rect::new(0, 0, 1512, 982), Rect::new(-2560, -200, 0, 1240)]);
        let p = l.point_on_edge(Edge::Left, 0.5, 0);
        assert_eq!(p.x, -2560);
    }
}
