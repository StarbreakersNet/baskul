//! Défilement fluide (mode `smooth`), reproduisant l'algorithme de Mac Mouse
//! Fix 3 (réglages « Vitesse » et « Douceur ») :
//!
//! - la vitesse de rotation de la molette est mesurée : plus les crans sont
//!   rapprochés, plus chacun fait défiler loin ;
//! - plusieurs gestes rapides enchaînés accélèrent encore (« fast scroll ») ;
//! - chaque cran lance une animation : vitesse constante pendant une durée de
//!   base (plus courte quand la molette tourne vite), puis freinage jusqu'à
//!   l'arrêt, comme un objet qui glisse ;
//! - un cran qui suit de près reprend la distance qui restait à parcourir ;
//!   un changement de sens arrête net le défilement.
//!
//! Les constantes viennent de Mac Mouse Fix (ScrollConfig.swift,
//! ScrollAnalyzer.m, Scroll.m, BezierCappedAccelerationCurve.swift) ; seule
//! la courbe du « fast scroll » est une approximation. L'option « Précision »
//! réduit à 10 pixels un cran isolé, en gardant l'accélération des crans
//! rapprochés. Comme Mac Mouse Fix, on produit des défilements continus au
//! pixel ; avec la simulation du trackpad (GestureScrollSimulator.m), la
//! phase à vitesse constante devient un geste à deux doigts et le freinage
//! l'élan qui suit (« momentum ») : les apps réagissent comme avec un
//! trackpad (rebond en bout de page, élan géré par l'app…).

use std::time::{Duration, Instant};

use crate::config::{SmoothSpeed, Smoothness};

/// Durée visée entre deux images de l'animation (120 images par seconde).
pub const FRAME: Duration = Duration::from_micros(8_333);

/// Au-delà de cet intervalle (s), deux crans ne se suivent pas.
const TICK_INTERVAL_MAX: f64 = 0.160;
/// Plus petit intervalle (s) pris en compte.
const TICK_INTERVAL_MIN: f64 = 0.001;
/// Intervalle (s) à partir duquel l'accélération plafonne.
const ACCELERATION_END: f64 = 0.015;
/// Crans enchaînés (le premier non compté) pour former un geste.
const SWIPE_MIN_TICKS: f64 = 2.0;
/// Vitesse (px/s) à laquelle le freinage s'arrête.
const STOP_SPEED: f64 = 30.0;

/// Sens du défilement (contenu vers le haut, le bas…).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// Phase d'un geste de trackpad (valeurs `IOHIDEventPhaseBits`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GesturePhase {
    Began = 1,
    Changed = 2,
    Ended = 4,
    Cancelled = 8,
    MayBegin = 128,
}

/// Phase de l'élan après le geste (valeurs `CGMomentumScrollPhase`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MomentumPhase {
    Begin = 1,
    Continue = 2,
    End = 3,
}

/// Événement à envoyer. `v` : vertical, positif vers le haut ; `h` :
/// horizontal, positif vers la droite (pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Défilement continu, sans phase.
    Scroll { v: i32, h: i32 },
    /// Geste à deux doigts (simulation du trackpad).
    Gesture { v: i32, h: i32, phase: GesturePhase },
    /// Élan après le geste (simulation du trackpad).
    Momentum { v: i32, h: i32, phase: MomentumPhase },
}

/// Partie de l'animation : vitesse constante (geste) ou freinage (élan).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Gesture,
    Momentum,
}

impl Part {
    fn end(self) -> Event {
        match self {
            Part::Gesture => Event::Gesture { v: 0, h: 0, phase: GesturePhase::Ended },
            Part::Momentum => Event::Momentum { v: 0, h: 0, phase: MomentumPhase::End },
        }
    }
}

/// « Tape » à deux doigts : arrête l'élan que certaines apps (Xcode…)
/// ajoutent d'elles-mêmes.
const TAP: [Event; 2] = [
    Event::Gesture { v: 0, h: 0, phase: GesturePhase::MayBegin },
    Event::Gesture { v: 0, h: 0, phase: GesturePhase::Cancelled },
];

/// Réglages du défilement fluide.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub speed: SmoothSpeed,
    pub smoothness: Smoothness,
    /// Option « Précision » de Mac Mouse Fix.
    pub precise: bool,
    /// Simulation du trackpad : gestes et élan au lieu de défilements simples.
    pub trackpad: bool,
}

/// Hauteur et largeur d'écran (points) pour lesquelles les distances sont
/// prévues ; Mac Mouse Fix les augmente un peu sur un écran plus grand.
const BASE_SCREEN: (f64, f64) = (1920.0, 1080.0);
/// Part de la distance maximale qui suit la taille de l'écran.
const SCREEN_WEIGHT: f64 = 0.1;

/// Paramètres dérivés des réglages.
#[derive(Debug, Clone, Copy)]
struct Params {
    /// Distance (px) d'un cran isolé, puis des crans les plus rapides.
    min_px: f64,
    max_px: f64,
    /// Bombement de la courbe d'accélération (0 = droite) : un bombement
    /// élevé fait aller plus loin les crans de vitesse moyenne.
    curvature: f64,
    /// Durée (s) de la phase à vitesse constante, crans lents → rapides.
    base_slow: f64,
    base_fast: f64,
    /// Freinage : dv/dt = −coef · v^exp.
    drag_coef: f64,
    drag_exp: f64,
    /// Gestes enchaînés : pause maximale (s) entre deux gestes, vitesse
    /// minimale (crans/s), nombre de gestes avant d'accélérer.
    swipe_max_pause: f64,
    swipe_min_speed: f64,
    fast_threshold: f64,
    /// Lissage amorcé avec un intervalle lent : accélération plus progressive.
    slow_start: bool,
}

impl Params {
    fn new(speed: SmoothSpeed, smoothness: Smoothness, precise: bool) -> Params {
        let i = match speed {
            SmoothSpeed::Low => 0,
            SmoothSpeed::Medium => 1,
            SmoothSpeed::High => 2,
        };
        let mut p = match smoothness {
            Smoothness::Regular => Params {
                min_px: [30.0, 60.0, 120.0][i],
                max_px: [90.0, 120.0, 180.0][i],
                curvature: if precise { [0.75, 0.75, 0.25][i] } else { [0.25, 0.0, 0.0][i] },
                base_slow: 0.180,
                base_fast: 0.110,
                drag_coef: 23.0,
                drag_exp: 1.0,
                swipe_max_pause: 0.375,
                swipe_min_speed: 16.0,
                fast_threshold: 3.0,
                slow_start: false,
            },
            Smoothness::High => Params {
                min_px: [60.0, 90.0, 150.0][i],
                max_px: [120.0, 180.0, 240.0][i],
                curvature: if precise { [1.5, 1.25, 0.75][i] } else { 0.0 },
                base_slow: 0.220,
                base_fast: 0.220,
                drag_coef: 40.0,
                drag_exp: 0.7,
                swipe_max_pause: 0.600,
                swipe_min_speed: 12.0,
                fast_threshold: 2.0,
                // Avec la précision, les petits gestes rapides sont déjà courts.
                slow_start: !precise,
            },
        };
        if precise {
            p.min_px = 10.0;
        }
        p
    }

    /// Distance d'un cran selon l'intervalle (lissé) entre les crans.
    /// `screen` : taille de l'écran rapportée à `BASE_SCREEN` (axe du défilement).
    fn tick_px(&self, interval: f64, screen: f64) -> f64 {
        let x = 1.0 / interval.max(ACCELERATION_END);
        let (x0, x1) = (1.0 / TICK_INTERVAL_MAX, 1.0 / ACCELERATION_END);
        let u = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
        let max = self.max_px * (1.0 - SCREEN_WEIGHT + SCREEN_WEIGHT * screen);
        self.min_px + (max - self.min_px) * accel_shape(self.curvature, u)
    }

    /// Durée de la phase à vitesse constante : elle raccourcit quand la
    /// molette tourne vite, surtout pour les crans très rapprochés.
    fn base_duration(&self, interval: f64) -> f64 {
        const CURVATURE: f64 = 4.0;
        let u = ((TICK_INTERVAL_MAX - interval) / (TICK_INTERVAL_MAX - TICK_INTERVAL_MIN)).clamp(0.0, 1.0);
        let e = ((u * CURVATURE).exp() - 1.0) / (CURVATURE.exp() - 1.0);
        self.base_slow + (self.base_fast - self.base_slow) * e
    }

    /// Multiplicateur des gestes rapides enchaînés.
    fn fast_scroll(&self, swipes: u32) -> f64 {
        let n = f64::from(swipes) + 1.0;
        if n < self.fast_threshold { 1.0 } else { (1.33 * (0.25 * (n - self.fast_threshold)).exp()).min(8.0) }
    }

    /// Distance parcourue en freinant de la vitesse `v` jusqu'à l'arrêt.
    fn glide(&self, v: f64) -> f64 {
        if v <= STOP_SPEED {
            return 0.0;
        }
        let (c, e) = (self.drag_coef, self.drag_exp);
        if e == 1.0 { (v - STOP_SPEED) / c } else { (v.powf(2.0 - e) - STOP_SPEED.powf(2.0 - e)) / ((2.0 - e) * c) }
    }

    /// Durée du freinage depuis la vitesse `v`.
    fn glide_time(&self, v: f64) -> f64 {
        if v <= STOP_SPEED {
            return 0.0;
        }
        let (c, e) = (self.drag_coef, self.drag_exp);
        if e == 1.0 {
            (v / STOP_SPEED).ln() / c
        } else {
            (v.powf(1.0 - e) - STOP_SPEED.powf(1.0 - e)) / ((1.0 - e) * c)
        }
    }

    /// Vitesse après `s` secondes de freinage depuis `v`.
    fn glide_speed(&self, v: f64, s: f64) -> f64 {
        let (c, e) = (self.drag_coef, self.drag_exp);
        if e == 1.0 { v * (-c * s).exp() } else { (v.powf(1.0 - e) - (1.0 - e) * c * s).max(0.0).powf(1.0 / (1.0 - e)) }
    }
}

/// Courbe d'accélération de Mac Mouse Fix (BezierCappedAccelerationCurve),
/// ramenée à [0, 1] × [0, 1] : courbe de Bézier dont le premier point de
/// contrôle est en bas et les suivants en haut, espacés de 1 / (bombement + 1)
/// sur l'axe horizontal (le dernier ramené à 1).
fn accel_shape(curvature: f64, u: f64) -> f64 {
    let degree = curvature + 1.0;
    let mut xs = vec![0.0];
    let mut i = 1.0;
    loop {
        xs.push((i / degree).min(1.0));
        if i >= degree {
            break;
        }
        i += 1.0;
    }
    // Abscisses croissantes : x(t) croît, on cherche t tel que x(t) = u.
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..50 {
        let t = (lo + hi) / 2.0;
        if bezier(&xs, t) < u {
            lo = t;
        } else {
            hi = t;
        }
    }
    // Ordonnées 0, 1, 1… : 1 − (1 − t)^n.
    1.0 - (1.0 - (lo + hi) / 2.0).powi(xs.len() as i32 - 1)
}

/// Courbe de Bézier à une dimension (algorithme de De Casteljau).
fn bezier(points: &[f64], t: f64) -> f64 {
    let mut p = points.to_vec();
    for n in (1..p.len()).rev() {
        for k in 0..n {
            p[k] += (p[k + 1] - p[k]) * t;
        }
    }
    p[0]
}

/// Mesure du rythme des crans (ScrollAnalyzer de Mac Mouse Fix).
#[derive(Debug, Default)]
struct Analyzer {
    prev_time: Option<Instant>,
    prev_dir: Option<Dir>,
    /// Crans enchaînés dans le geste en cours (le premier non compté).
    ticks: f64,
    /// Gestes rapides enchaînés.
    swipes: u32,
    sequence_ticks: f64,
    sequence_start: Option<Instant>,
    /// Derniers intervalles, pour une moyenne glissante.
    recent: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct Analysis {
    reversed: bool,
    /// Premier cran d'un geste.
    first: bool,
    swipes: u32,
    /// Intervalle lissé entre les crans (s).
    interval: f64,
}

impl Analyzer {
    fn update(&mut self, p: &Params, now: Instant, dir: Dir, weight: f64) -> Analysis {
        let reversed = self.prev_dir.is_some_and(|d| d != dir);
        if reversed {
            *self = Analyzer::default();
        }
        self.prev_dir = Some(dir);
        let since = |t: Instant| now.saturating_duration_since(t).as_secs_f64();
        let dt = self.prev_time.map_or(f64::INFINITY, since).max(TICK_INTERVAL_MIN);
        self.prev_time = Some(now);
        self.sequence_ticks += weight;
        let first = dt > TICK_INTERVAL_MAX;
        if first {
            let speed = self.sequence_start.map_or(0.0, |s| self.sequence_ticks / since(s).max(TICK_INTERVAL_MIN));
            if self.ticks >= SWIPE_MIN_TICKS && dt <= p.swipe_max_pause && speed >= p.swipe_min_speed {
                self.swipes += 1;
            } else {
                self.swipes = 0;
                self.sequence_start = Some(now);
                self.sequence_ticks = 0.0;
            }
            self.ticks = 0.0;
            self.recent.clear();
            if p.slow_start {
                self.recent.push(TICK_INTERVAL_MAX);
            }
        } else {
            self.ticks += weight;
            if self.recent.len() == 3 {
                self.recent.remove(0);
            }
            // Fraction de cran (molette haute résolution) : intervalle
            // ramené à un cran entier.
            self.recent.push((dt / weight).min(TICK_INTERVAL_MAX));
        }
        let interval = if self.recent.is_empty() {
            TICK_INTERVAL_MAX
        } else {
            self.recent.iter().sum::<f64>() / self.recent.len() as f64
        };
        Analysis { reversed, first, swipes: self.swipes, interval }
    }
}

/// Une animation : vitesse constante pendant `base` secondes, puis freinage.
#[derive(Debug, Clone, Copy)]
struct Animation {
    start: Instant,
    dir: Dir,
    distance: f64,
    speed: f64,
    base: f64,
    duration: f64,
    /// Distance atteinte à la dernière image.
    done: f64,
}

impl Animation {
    fn new(p: &Params, start: Instant, dir: Dir, distance: f64, base: f64) -> Animation {
        // Vitesse telle que vitesse × base + freinage = distance (fonction
        // croissante de la vitesse : dichotomie).
        let (mut lo, mut hi) = (0.0, distance / base);
        for _ in 0..60 {
            let mid = (lo + hi) / 2.0;
            if mid * base + p.glide(mid) < distance {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Animation { start, dir, distance, speed: hi, base, duration: base + p.glide_time(hi), done: 0.0 }
    }

    fn position(&self, p: &Params, t: f64) -> f64 {
        if t >= self.duration {
            self.distance
        } else if t <= self.base {
            self.speed * t
        } else {
            let v = p.glide_speed(self.speed, t - self.base);
            (self.speed * self.base + p.glide(self.speed) - p.glide(v)).min(self.distance)
        }
    }

    fn is_running(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start).as_secs_f64() < self.duration
    }
}

/// Défilement fluide d'une molette : reçoit les crans, produit les pixels à
/// envoyer à chaque image.
#[derive(Debug)]
pub struct Scroller {
    params: Params,
    analyzer: Analyzer,
    anim: Option<Animation>,
    /// Fraction de pixel pas encore envoyée.
    carry: f64,
    /// Simulation du trackpad : gestes et élan au lieu de défilements simples.
    trackpad: bool,
    /// Partie en cours (geste ou élan) des événements déjà envoyés.
    part: Option<Part>,
    /// Événements à envoyer à la prochaine image (arrêt net).
    pending: Vec<Event>,
    /// Largeur et hauteur de l'écran rapportées à `BASE_SCREEN`.
    screen: (f64, f64),
}

impl Scroller {
    pub fn new(o: Options) -> Scroller {
        Scroller {
            params: Params::new(o.speed, o.smoothness, o.precise),
            analyzer: Analyzer::default(),
            anim: None,
            carry: 0.0,
            trackpad: o.trackpad,
            part: None,
            pending: vec![],
            screen: (1.0, 1.0),
        }
    }

    /// Taille (points) de l'écran sous le pointeur.
    pub fn set_screen(&mut self, width: f64, height: f64) {
        if width > 0.0 && height > 0.0 {
            self.screen = (width / BASE_SCREEN.0, height / BASE_SCREEN.1);
        }
    }

    /// Cran de molette reçu à `now`. `weight` vaut 1 pour un cran entier, une
    /// fraction pour les molettes haute résolution.
    pub fn tick(&mut self, now: Instant, dir: Dir, weight: f64) {
        let p = self.params;
        let a = self.analyzer.update(&p, now, dir, weight);
        let running = self.anim.filter(|an| an.is_running(now));
        if a.reversed && running.is_some() {
            // Changement de sens pendant le défilement : arrêt net.
            self.anim = None;
            self.carry = 0.0;
            if let Some(part) = self.part.take() {
                self.pending.push(part.end());
                self.pending.extend(TAP);
            }
            return;
        }
        // Un cran qui suit de près reprend ce qui restait à parcourir ; un
        // nouveau geste repart de zéro.
        let left = match running {
            Some(an) if !(a.first && a.swipes == 0) => an.distance - an.done,
            _ => {
                self.carry = 0.0;
                0.0
            }
        };
        let screen = if matches!(dir, Dir::Left | Dir::Right) { self.screen.0 } else { self.screen.1 };
        let px = p.tick_px(a.interval, screen) * weight * p.fast_scroll(a.swipes);
        self.anim = Some(Animation::new(&p, now, dir, px + left, p.base_duration(a.interval)));
    }

    /// Plus rien à animer ni à envoyer.
    pub fn is_idle(&self) -> bool {
        self.anim.is_none() && self.pending.is_empty()
    }

    /// Événements à envoyer pour l'image affichée à `now`.
    pub fn frame(&mut self, now: Instant) -> Vec<Event> {
        let p = self.params;
        let mut out = std::mem::take(&mut self.pending);
        let Some(an) = self.anim.as_mut() else { return out };
        let t = now.saturating_duration_since(an.start).as_secs_f64();
        let pos = an.position(&p, t);
        let total = pos - an.done + self.carry;
        an.done = pos;
        let px = total.round();
        self.carry = total - px;
        let (dir, part) = (an.dir, if t <= an.base { Part::Gesture } else { Part::Momentum });
        let done = t >= an.duration;
        if done {
            self.anim = None;
        }
        let px = px as i32;
        let (v, h) = match dir {
            Dir::Up => (px, 0),
            Dir::Down => (-px, 0),
            Dir::Left => (0, -px),
            Dir::Right => (0, px),
        };
        if !self.trackpad {
            if px != 0 {
                out.push(Event::Scroll { v, h });
            }
            return out;
        }
        // Simulation du trackpad (comme Mac Mouse Fix) : geste pendant la
        // phase à vitesse constante, élan pendant le freinage ; un nouveau
        // cran pendant l'élan termine celui-ci et commence un nouveau geste.
        if self.part.is_some_and(|q| q != part) {
            out.push(self.part.take().unwrap().end());
        }
        if px != 0 {
            let started = self.part.is_none();
            out.push(match part {
                Part::Gesture => {
                    Event::Gesture { v, h, phase: if started { GesturePhase::Began } else { GesturePhase::Changed } }
                }
                Part::Momentum => Event::Momentum {
                    v,
                    h,
                    phase: if started { MomentumPhase::Begin } else { MomentumPhase::Continue },
                },
            });
            self.part = Some(part);
        }
        if done {
            if let Some(part) = self.part.take() {
                out.push(part.end());
                if part == Part::Momentum {
                    out.extend(TAP);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(1);

    /// Simule une molette : crans `(instant en ms, sens, poids)`, images
    /// toutes les 8 ms. Renvoie les événements de chaque image.
    fn run(s: &mut Scroller, ticks: &[(u64, Dir, f64)]) -> Vec<Vec<Event>> {
        let t0 = Instant::now();
        let mut frames = vec![];
        let mut next = ticks.iter().peekable();
        let mut t = 0;
        while next.peek().is_some() || !s.is_idle() {
            while let Some(&&(at, dir, w)) = next.peek() {
                if at > t {
                    break;
                }
                s.tick(t0 + MS * at as u32, dir, w);
                next.next();
            }
            frames.push(s.frame(t0 + MS * t as u32));
            t += 8;
            assert!(t < 60_000, "animation sans fin");
        }
        frames
    }

    /// Pixels (vertical, horizontal) de chaque image.
    fn simulate(s: &mut Scroller, ticks: &[(u64, Dir, f64)]) -> Vec<(i32, i32)> {
        run(s, ticks)
            .iter()
            .map(|frame| {
                frame.iter().fold((0, 0), |(av, ah), e| match *e {
                    Event::Scroll { v, h } | Event::Gesture { v, h, .. } | Event::Momentum { v, h, .. } => {
                        (av + v, ah + h)
                    }
                })
            })
            .collect()
    }

    /// Tous les événements, dans l'ordre.
    fn events(s: &mut Scroller, ticks: &[(u64, Dir, f64)]) -> Vec<Event> {
        run(s, ticks).concat()
    }

    fn vertical(frames: &[(i32, i32)]) -> i32 {
        frames.iter().map(|f| f.0).sum()
    }

    fn scroller(speed: SmoothSpeed, smoothness: Smoothness, precise: bool, trackpad: bool) -> Scroller {
        Scroller::new(Options { speed, smoothness, precise, trackpad })
    }

    fn regular() -> Scroller {
        scroller(SmoothSpeed::Medium, Smoothness::Regular, false, false)
    }

    #[test]
    fn single_notch_glides_to_a_stop() {
        let mut s = regular();
        let frames = simulate(&mut s, &[(0, Dir::Up, 1.0)]);
        assert_eq!(vertical(&frames), 60);
        // Environ 280 ms, comme Mac Mouse Fix.
        let ms = frames.len() * 8;
        assert!((250..=320).contains(&ms), "{ms} ms");
        // Vitesse constante, puis freinage.
        let moving: Vec<i32> = frames.iter().map(|f| f.0).filter(|&v| v != 0).collect();
        assert!(moving[1].abs_diff(moving[2]) <= 1, "{moving:?}");
        assert!(moving[moving.len() - 1] < moving[2], "{moving:?}");
        assert!(frames.iter().all(|f| f.0 >= 0 && f.1 == 0));
    }

    #[test]
    fn directions_and_horizontal() {
        let mut s = regular();
        assert_eq!(vertical(&simulate(&mut s, &[(0, Dir::Down, 1.0)])), -60);
        let frames = simulate(&mut s, &[(0, Dir::Right, 1.0)]);
        assert_eq!(frames.iter().map(|f| f.1).sum::<i32>(), 60);
        assert_eq!(vertical(&frames), 0);
    }

    #[test]
    fn fast_notches_go_further() {
        let mut s = regular();
        let slow: Vec<_> = (0..5).map(|i| (i * 300, Dir::Up, 1.0)).collect();
        let slow = vertical(&simulate(&mut s, &slow));
        let fast: Vec<_> = (0..5).map(|i| (i * 15, Dir::Up, 1.0)).collect();
        let fast = vertical(&simulate(&mut regular(), &fast));
        assert_eq!(slow, 300);
        assert!(fast > 420, "rapide : {fast}");
    }

    #[test]
    fn close_notches_keep_the_rest_of_the_animation() {
        // Deux crans à 100 ms : la seconde animation reprend le reste de la
        // première, rien n'est perdu.
        let mut s = regular();
        let total = vertical(&simulate(&mut s, &[(0, Dir::Up, 1.0), (100, Dir::Up, 1.0)]));
        assert!(total >= 120, "{total}");
    }

    #[test]
    fn reversing_stops_at_once() {
        let mut s = regular();
        let frames = simulate(&mut s, &[(0, Dir::Up, 1.0), (40, Dir::Down, 1.0)]);
        let up = vertical(&frames);
        assert!(up > 0 && up < 30, "{up}");
        assert!(frames.iter().all(|f| f.0 >= 0));
        // Le cran suivant dans le nouveau sens défile normalement.
        assert_eq!(vertical(&simulate(&mut s, &[(500, Dir::Down, 1.0)])), -60);
    }

    #[test]
    fn repeated_flicks_speed_up() {
        // Gestes de 6 crans rapides séparés par de courtes pauses.
        let mut ticks = vec![];
        for swipe in 0..6 {
            for i in 0..6 {
                ticks.push((swipe * 300 + i * 20, Dir::Up, 1.0));
            }
        }
        let mut s = regular();
        let t0 = Instant::now();
        let mut swipes = vec![];
        for chunk in ticks.chunks(6) {
            for &(at, dir, w) in chunk {
                s.tick(t0 + MS * at as u32, dir, w);
            }
            swipes.push(s.analyzer.swipes);
        }
        assert_eq!(swipes, [0, 1, 2, 3, 4, 5]);
        let p = s.params;
        assert_eq!(p.fast_scroll(1), 1.0);
        assert!(p.fast_scroll(5) > 2.0 * p.fast_scroll(2), "{}", p.fast_scroll(5));
        // Une vraie pause remet à zéro.
        s.tick(t0 + MS * 3_000, Dir::Up, 1.0);
        assert_eq!(s.analyzer.swipes, 0);
    }

    #[test]
    fn high_resolution_wheels() {
        // Quatre quarts de cran espacés comme un cran lent : un cran.
        let mut s = regular();
        let quarters: Vec<_> = (0..4).map(|i| (i * 400, Dir::Up, 0.25)).collect();
        assert_eq!(vertical(&simulate(&mut s, &quarters)), 60);
        // Des quarts de cran rapprochés (4 par 40 ms) : un cran toutes les
        // 40 ms, ce qui accélère.
        let fast: Vec<_> = (0..8).map(|i| (i * 10, Dir::Up, 0.25)).collect();
        let total = vertical(&simulate(&mut regular(), &fast));
        assert!(total > 125, "{total}");
    }

    #[test]
    fn high_smoothness_glides_longer() {
        let mut s = scroller(SmoothSpeed::Medium, Smoothness::High, false, false);
        let frames = simulate(&mut s, &[(0, Dir::Up, 1.0)]);
        assert_eq!(vertical(&frames), 90);
        assert!(frames.len() * 8 > 320, "{} ms", frames.len() * 8);
    }

    #[test]
    fn speed_setting_scales_distance() {
        let one =
            |speed| vertical(&simulate(&mut scroller(speed, Smoothness::Regular, false, false), &[(0, Dir::Up, 1.0)]));
        assert_eq!((one(SmoothSpeed::Low), one(SmoothSpeed::High)), (30, 120));
    }

    #[test]
    fn acceleration_curve_matches_mac_mouse_fix() {
        // Sans bombement : droite.
        for u in [0.0, 0.3, 0.5, 1.0] {
            assert!((accel_shape(0.0, u) - u).abs() < 1e-9);
        }
        // Bombement entier n − 1 : 1 − (1 − u)^n (points équidistants).
        for u in [0.2, 0.5, 0.8] {
            assert!((accel_shape(1.0, u) - (1.0 - (1.0 - u).powi(2))).abs() < 1e-9);
            assert!((accel_shape(2.0, u) - (1.0 - (1.0 - u).powi(3))).abs() < 1e-9);
        }
        // Bombement 0,75 : points (0, 0), (4/7, 1), (1, 1). À t = 0,5 :
        // x = 2 · 0,5 · 0,5 · 4/7 + 0,25 = 0,5357…, y = 0,75.
        assert!((accel_shape(0.75, 0.25 + 2.0 / 7.0) - 0.75).abs() < 1e-9);
        // Croissante, de 0 à 1, au-dessus de la droite.
        let mut prev = 0.0;
        for k in 0..=100 {
            let u = k as f64 / 100.0;
            let y = accel_shape(1.25, u);
            assert!(y >= prev - 1e-12 && y >= u - 1e-9, "{u} → {y}");
            prev = y;
        }
        assert!((prev - 1.0).abs() < 1e-9);
    }

    #[test]
    fn precision_shortens_single_notches_only() {
        let precise = || scroller(SmoothSpeed::Medium, Smoothness::High, true, false);
        // Cran isolé : 10 pixels.
        assert_eq!(vertical(&simulate(&mut precise(), &[(0, Dir::Up, 1.0)])), 10);
        // Crans rapides : l'accélération reste (même distance maximale).
        let p = precise().params;
        let normal = scroller(SmoothSpeed::Medium, Smoothness::High, false, false).params;
        assert!((p.tick_px(ACCELERATION_END, 1.0) - normal.tick_px(ACCELERATION_END, 1.0)).abs() < 1e-9);
        // Vitesse moyenne : la courbe bombée relève la distance.
        let mid = 0.040;
        let linear = 10.0 + (180.0 - 10.0) * (1.0 / mid - 6.25) / (1.0 / ACCELERATION_END - 6.25);
        assert!(p.tick_px(mid, 1.0) > linear + 10.0, "{}", p.tick_px(mid, 1.0));
        // Pas de démarrage progressif en douceur élevée avec la précision.
        assert!(!p.slow_start && normal.slow_start);
    }

    #[test]
    fn bigger_screens_scroll_a_bit_further() {
        let mut s = regular();
        s.set_screen(2560.0, 1440.0);
        let p = s.params;
        let (w, h) = s.screen;
        // Distance maximale : 120 × (0,9 + 0,1 × 1440 / 1080) = 124.
        assert!((p.tick_px(ACCELERATION_END, h) - 124.0).abs() < 1e-9);
        assert!((p.tick_px(ACCELERATION_END, w) - 120.0 * (0.9 + 0.1 * 2560.0 / 1920.0)).abs() < 1e-9);
        // Un cran isolé ne change pas.
        assert!((p.tick_px(TICK_INTERVAL_MAX, h) - 60.0).abs() < 1e-9);
    }

    fn trackpad() -> Scroller {
        scroller(SmoothSpeed::Medium, Smoothness::High, false, true)
    }

    fn phases(evs: &[Event]) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        for e in evs {
            let name = match e {
                Event::Scroll { .. } => "scroll".to_string(),
                Event::Gesture { phase, .. } => format!("{phase:?}"),
                Event::Momentum { phase, .. } => format!("momentum {phase:?}"),
            };
            // Les répétitions sont regroupées.
            if out.last() != Some(&name) {
                out.push(name);
            }
        }
        out
    }

    #[test]
    fn trackpad_simulation_gesture_then_momentum() {
        let evs = events(&mut trackpad(), &[(0, Dir::Up, 1.0)]);
        assert_eq!(
            phases(&evs),
            [
                "Began",
                "Changed",
                "Ended",
                "momentum Begin",
                "momentum Continue",
                "momentum End",
                "MayBegin",
                "Cancelled"
            ]
        );
        let total: i32 = evs
            .iter()
            .map(|e| match *e {
                Event::Gesture { v, .. } | Event::Momentum { v, .. } => v,
                Event::Scroll { .. } => panic!("défilement simple inattendu"),
            })
            .sum();
        assert_eq!(total, 90);
    }

    #[test]
    fn trackpad_new_notch_during_momentum_starts_new_gesture() {
        // Second cran pendant l'élan du premier (vers 300 ms).
        let evs = events(&mut trackpad(), &[(0, Dir::Up, 1.0), (300, Dir::Up, 1.0)]);
        let p = phases(&evs);
        let i = p.iter().position(|x| x == "momentum Begin").unwrap();
        assert_eq!(p[i + 1..i + 5], ["momentum Continue", "momentum End", "Began", "Changed"]);
    }

    #[test]
    fn trackpad_reversal_ends_and_taps() {
        let evs = events(&mut trackpad(), &[(0, Dir::Up, 1.0), (40, Dir::Down, 1.0)]);
        assert_eq!(phases(&evs), ["Began", "Changed", "Ended", "MayBegin", "Cancelled"]);
    }
}
