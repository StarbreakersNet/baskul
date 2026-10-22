//! Défilement fluide (mode `smooth`) : chaque cran de molette ajoute une
//! distance à parcourir, parcourue en quelques images avec un ralentissement
//! progressif, à la manière d'un trackpad ou de Mac Mouse Fix.

use std::time::Duration;

/// Durée entre deux images de l'animation (environ 120 images par seconde).
pub const FRAME: Duration = Duration::from_millis(8);

/// Constante de temps du ralentissement : 95 % de la distance est parcourue
/// en trois fois cette durée (150 ms).
const TAU: f64 = 0.05;

/// Mouvement restant sur un axe, en pixels.
#[derive(Debug, Default, Clone, Copy)]
pub struct Axis {
    remaining: f64,
    /// Fraction de pixel pas encore envoyée.
    carry: f64,
}

impl Axis {
    /// Nouveau cran. Un changement de sens annule le reste du mouvement.
    pub fn push(&mut self, px: f64) {
        if px * self.remaining < 0.0 {
            *self = Axis::default();
        }
        self.remaining += px;
    }

    pub fn is_idle(&self) -> bool {
        self.remaining.abs() < 0.5
    }

    /// Pixels à envoyer pour l'image suivante.
    pub fn step(&mut self) -> i32 {
        if self.is_idle() {
            *self = Axis::default();
            return 0;
        }
        let k = 1.0 - (-FRAME.as_secs_f64() / TAU).exp();
        let mut take = self.remaining * k;
        // Au moins un pixel par image, pour finir sans traîner.
        if take.abs() < 1.0 {
            take = self.remaining.clamp(-1.0, 1.0);
        }
        self.remaining -= take;
        let total = take + self.carry;
        let px = total.round();
        self.carry = total - px;
        px as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(axis: &mut Axis) -> Vec<i32> {
        let mut steps = vec![];
        while !axis.is_idle() {
            steps.push(axis.step());
            assert!(steps.len() < 500, "animation sans fin");
        }
        steps
    }

    #[test]
    fn one_notch_is_eased_out() {
        let mut a = Axis::default();
        a.push(60.0);
        let steps = run(&mut a);
        let total: i32 = steps.iter().sum();
        assert!((59..=60).contains(&total), "total {total}");
        // Rapide au début, puis de plus en plus lent.
        assert!(steps[0] > steps[steps.len() / 2]);
        assert!(steps.iter().all(|s| *s >= 0));
        // Animation courte : moins de 300 ms.
        assert!(steps.len() as u32 * FRAME.as_millis() as u32 <= 300, "{} images", steps.len());
    }

    #[test]
    fn notches_add_up_and_reverse_cancels() {
        let mut a = Axis::default();
        a.push(60.0);
        a.step();
        a.push(60.0);
        let total: i32 = a.step() + run(&mut a).iter().sum::<i32>();
        assert!(total > 100);
        // Changement de sens : on repart de zéro dans l'autre sens.
        a.push(60.0);
        a.step();
        a.push(-30.0);
        let back: i32 = run(&mut a).iter().sum();
        assert!((-30..=-29).contains(&back), "{back}");
    }
}
