//! Manettes de la machine principale : ni les hooks clavier et souris ni le
//! système ne voient leur utilisation, la couche système les sonde donc
//! elle-même. Seul un vrai geste compte comme activité : une manette posée,
//! dont un stick dérive un peu ou qui envoie des rapports en continu, ne
//! garde aucune machine éveillée.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// Écart minimal, sur une position de -1 à 1 (ou de 0 à 1), pour qu'un
/// mouvement compte : bien au-delà du bruit et de la dérive d'un stick.
const STEP: f32 = 0.25;

const PAGE_DESKTOP: u32 = 0x01;
const PAGE_SIMULATION: u32 = 0x02;
const PAGE_BUTTON: u32 = 0x09;
const USAGE_X: u32 = 0x30;
const USAGE_DIAL: u32 = 0x37;
const USAGE_HAT: u32 = 0x39;
const USAGE_ACCELERATOR: u32 = 0xC4;
const USAGE_BRAKE: u32 = 0xC5;

/// Dernière position comptée de chaque commande (bouton, gâchette, axe d'un
/// stick), par manette.
#[derive(Debug, Default)]
pub struct GamepadWatch {
    levels: HashMap<(u64, u32), f32>,
}

impl GamepadWatch {
    /// Nouvelle position d'une commande. Renvoie `true` pour un vrai geste ;
    /// la première position connue d'une commande sert de référence.
    pub fn moved(&mut self, pad: u64, control: u32, level: f32) -> bool {
        match self.levels.entry((pad, control)) {
            Entry::Vacant(e) => {
                e.insert(level);
                false
            }
            Entry::Occupied(mut e) => {
                let moved = (level - *e.get()).abs() >= STEP;
                if moved {
                    e.insert(level);
                }
                moved
            }
        }
    }

    /// État complet d'une manette : la commande `i` est à `levels[i]`.
    pub fn moved_any(&mut self, pad: u64, levels: &[f32]) -> bool {
        let mut moved = false;
        for (control, level) in (0u32..).zip(levels) {
            moved |= self.moved(pad, control, *level);
        }
        moved
    }

    /// Manette débranchée : une autre peut reprendre son identifiant.
    pub fn forget(&mut self, pad: u64) {
        self.levels.retain(|(p, _), _| *p != pad);
    }
}

/// Position d'une commande HID d'une manette (page et usage HID), ou `None`
/// si elle n'en est pas une (gyroscope, données du fabricant…). Boutons :
/// 0 ou 1 ; axes : de -1 à 1 ; croix directionnelle : une valeur par
/// direction, 0 au repos.
pub fn hid_level(page: u32, usage: u32, value: i64, min: i64, max: i64) -> Option<f32> {
    let is_axis = matches!((page, usage), (PAGE_DESKTOP, USAGE_X..=USAGE_DIAL))
        || matches!((page, usage), (PAGE_SIMULATION, USAGE_ACCELERATOR | USAGE_BRAKE));
    match (page, usage) {
        (PAGE_BUTTON, _) => Some(if value > min.max(0) { 1.0 } else { 0.0 }),
        (PAGE_DESKTOP, USAGE_HAT) => Some(if (min..=max).contains(&value) { 1.0 + (value - min) as f32 } else { 0.0 }),
        _ if is_axis && max > min => Some(2.0 * (value - min) as f32 / (max - min) as f32 - 1.0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_real_moves_count() {
        let mut w = GamepadWatch::default();
        assert!(!w.moved(0, 1, 0.02), "première position : référence");
        // Bruit, puis dérive lente : jamais assez loin de la référence.
        assert!(!w.moved(0, 1, -0.03));
        assert!(!w.moved(0, 1, 0.1));
        assert!(!w.moved(0, 1, 0.2));
        assert!(w.moved(0, 1, 0.9));
        assert!(!w.moved(0, 1, 0.85));
        // Bouton : chaque appui et chaque relâchement.
        assert!(!w.moved(0, 2, 0.0));
        assert!(w.moved(0, 2, 1.0));
        assert!(w.moved(0, 2, 0.0));
    }

    #[test]
    fn pads_are_watched_separately() {
        let mut w = GamepadWatch::default();
        assert!(!w.moved_any(0, &[0.0, 0.0, 0.0]));
        assert!(!w.moved_any(1, &[1.0, 1.0, 1.0]));
        // Toutes les commandes sont mises à jour, pas seulement la première qui bouge.
        assert!(w.moved_any(0, &[1.0, 0.0, 1.0]));
        assert!(!w.moved_any(0, &[1.0, 0.0, 1.0]));
        w.forget(0);
        assert!(!w.moved_any(0, &[0.0, 0.0, 0.0]));
        assert!(!w.moved_any(1, &[1.0, 1.0, 1.0]));
    }

    #[test]
    fn hid_controls_are_normalized() {
        assert_eq!(hid_level(PAGE_BUTTON, 3, 1, 0, 1), Some(1.0));
        assert_eq!(hid_level(PAGE_BUTTON, 3, 0, 0, 1), Some(0.0));
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_X, 0, 0, 255), Some(-1.0));
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_X, 255, 0, 255), Some(1.0));
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_X + 5, 0, -32768, 32767).map(|v| v.abs() < 0.01), Some(true));
        assert_eq!(hid_level(PAGE_SIMULATION, USAGE_BRAKE, 255, 0, 255), Some(1.0));
        // Croix : neutre hors de l'intervalle, une valeur distincte par direction.
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_HAT, 8, 0, 7), Some(0.0));
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_HAT, 0, 0, 7), Some(1.0));
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_HAT, 2, 0, 7), Some(3.0));
        // Gyroscope, données du fabricant : ignorés.
        assert_eq!(hid_level(0xFF00, 0x20, 12, 0, 255), None);
        assert_eq!(hid_level(PAGE_DESKTOP, 0x3B, 12, 0, 255), None);
        assert_eq!(hid_level(PAGE_DESKTOP, USAGE_X, 3, 0, 0), None);
    }
}
