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
//! ScrollAnalyzer.m, Scroll.m) ; seule la courbe du « fast scroll » est une
//! approximation. Comme Mac Mouse Fix, on produit des défilements continus au
//! pixel, sans phases de geste.

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

/// Paramètres dérivés des réglages.
#[derive(Debug, Clone, Copy)]
struct Params {
    /// Distance (px) d'un cran isolé, puis des crans les plus rapides.
    min_px: f64,
    max_px: f64,
    /// Bombement de la courbe d'accélération (0 = droite).
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
    fn new(speed: SmoothSpeed, smoothness: Smoothness) -> Params {
        let i = match speed {
            SmoothSpeed::Low => 0,
            SmoothSpeed::Medium => 1,
            SmoothSpeed::High => 2,
        };
        match smoothness {
            Smoothness::Regular => Params {
                min_px: [30.0, 60.0, 120.0][i],
                max_px: [90.0, 120.0, 180.0][i],
                curvature: [0.25, 0.0, 0.0][i],
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
                curvature: 0.0,
                base_slow: 0.220,
                base_fast: 0.220,
                drag_coef: 40.0,
                drag_exp: 0.7,
                swipe_max_pause: 0.600,
                swipe_min_speed: 12.0,
                fast_threshold: 2.0,
                slow_start: true,
            },
        }
    }

    /// Distance d'un cran selon l'intervalle (lissé) entre les crans.
    fn tick_px(&self, interval: f64) -> f64 {
        let x = 1.0 / interval.max(ACCELERATION_END);
        let (x0, x1) = (1.0 / TICK_INTERVAL_MAX, 1.0 / ACCELERATION_END);
        let u = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
        let shape = u + self.curvature * u * (1.0 - u);
        self.min_px + (self.max_px - self.min_px) * shape
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
}

impl Scroller {
    pub fn new(speed: SmoothSpeed, smoothness: Smoothness) -> Scroller {
        Scroller { params: Params::new(speed, smoothness), analyzer: Analyzer::default(), anim: None, carry: 0.0 }
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
        let px = p.tick_px(a.interval) * weight * p.fast_scroll(a.swipes);
        self.anim = Some(Animation::new(&p, now, dir, px + left, p.base_duration(a.interval)));
    }

    /// Plus rien à animer.
    pub fn is_idle(&self) -> bool {
        self.anim.is_none()
    }

    /// Pixels à envoyer pour l'image affichée à `now` : (vertical, positif
    /// vers le haut ; horizontal, positif vers la droite).
    pub fn frame(&mut self, now: Instant) -> (i32, i32) {
        let p = self.params;
        let Some(an) = self.anim.as_mut() else { return (0, 0) };
        let t = now.saturating_duration_since(an.start).as_secs_f64();
        let pos = an.position(&p, t);
        let total = pos - an.done + self.carry;
        an.done = pos;
        let px = total.round();
        self.carry = total - px;
        let dir = an.dir;
        if t >= an.duration {
            self.anim = None;
        }
        let px = px as i32;
        match dir {
            Dir::Up => (px, 0),
            Dir::Down => (-px, 0),
            Dir::Left => (0, -px),
            Dir::Right => (0, px),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(1);

    /// Simule une molette : crans `(instant en ms, sens, poids)`, images
    /// toutes les 8 ms. Renvoie les pixels (vertical, horizontal) par image.
    fn simulate(s: &mut Scroller, ticks: &[(u64, Dir, f64)]) -> Vec<(i32, i32)> {
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

    fn vertical(frames: &[(i32, i32)]) -> i32 {
        frames.iter().map(|f| f.0).sum()
    }

    fn regular() -> Scroller {
        Scroller::new(SmoothSpeed::Medium, Smoothness::Regular)
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
        let mut s = Scroller::new(SmoothSpeed::Medium, Smoothness::High);
        let frames = simulate(&mut s, &[(0, Dir::Up, 1.0)]);
        assert_eq!(vertical(&frames), 90);
        assert!(frames.len() * 8 > 320, "{} ms", frames.len() * 8);
    }

    #[test]
    fn speed_setting_scales_distance() {
        let low = vertical(&simulate(&mut Scroller::new(SmoothSpeed::Low, Smoothness::Regular), &[(0, Dir::Up, 1.0)]));
        let high =
            vertical(&simulate(&mut Scroller::new(SmoothSpeed::High, Smoothness::Regular), &[(0, Dir::Up, 1.0)]));
        assert_eq!((low, high), (30, 120));
    }
}
