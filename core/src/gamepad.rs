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

const VALVE: u32 = 0x28DE;
/// Premier Steam Controller (2015) : câble, dongle.
const STEAM_CONTROLLER: [u32; 2] = [0x1102, 0x1142];
/// Nouveau Steam Controller : câble, Bluetooth, dongles.
const STEAM_TRITON: [u32; 4] = [0x1302, 0x1303, 0x1304, 0x1305];
const STEAM_REPORT_VERSION: [u8; 2] = [0x01, 0x00];
const STEAM_REPORT_STATE: u8 = 0x01;
/// Numéros des rapports d'état du nouveau Steam Controller (par câble ou
/// dongle, en Bluetooth, et avec l'horodatage des pavés tactiles).
const TRITON_STATE: u8 = 0x42;
const TRITON_STATE_BLE: u8 = 0x45;
const TRITON_STATE_TIMESTAMP: u8 = 0x47;
/// Le plus grand nombre de commandes lues, pour les deux modèles : 48
/// boutons, 2 gâchettes et 4 axes (premier modèle) ; 32 boutons, 2
/// gâchettes et 8 axes (nouveau modèle).
pub const STEAM_CONTROLS: usize = 54;

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

/// Steam Controller, premier ou nouveau modèle, par câble ou par dongle
/// (les autres manettes de Valve envoient d'autres rapports).
pub fn is_steam_controller(vendor: u32, product: u32) -> bool {
    vendor == VALVE && (STEAM_CONTROLLER.contains(&product) || STEAM_TRITON.contains(&product))
}

/// Rapport HID d'un Steam Controller : position de chaque commande pour un
/// rapport d'état, `None` pour les autres (connexion sans fil, batterie…).
/// Le gyroscope et l'accéléromètre sont ignorés. Formats : `controller_structs.h`
/// de SDL.
pub fn steam_controller_levels(report: &[u8]) -> Option<[f32; STEAM_CONTROLS]> {
    match report.first() {
        Some(&(TRITON_STATE | TRITON_STATE_BLE)) => triton_levels(report, 18),
        Some(&TRITON_STATE_TIMESTAMP) => triton_levels(report, 20),
        _ => legacy_levels(report),
    }
}

/// Premier modèle. Sous Windows, le rapport commence par son numéro, toujours 0.
fn legacy_levels(report: &[u8]) -> Option<[f32; STEAM_CONTROLS]> {
    let r = match report {
        [0, rest @ ..] => rest,
        _ => report,
    };
    if r.len() < 24 || r[..2] != STEAM_REPORT_VERSION || r[2] != STEAM_REPORT_STATE {
        return None;
    }
    let buttons = u64::from_le_bytes(r[8..16].try_into().ok()?);
    let axis = |i: usize| f32::from(i16::from_le_bytes([r[i], r[i + 1]])) / 32768.0;
    let mut levels = [0.0; STEAM_CONTROLS];
    // Octets 3 et 4 du champ des boutons : les gâchettes.
    for (level, bit) in levels.iter_mut().zip((0..24).chain(40..64)) {
        *level = ((buttons >> bit) & 1) as f32;
    }
    levels[48] = f32::from(r[11]) / 255.0;
    levels[49] = f32::from(r[12]) / 255.0;
    for (level, i) in levels[50..].iter_mut().zip([16, 18, 20, 22]) {
        *level = axis(i);
    }
    Some(levels)
}

/// Nouveau modèle : numéro du rapport, puis l'état ; `pads` : position du
/// pavé tactile gauche, que l'horodatage des pavés décale.
fn triton_levels(r: &[u8], pads: usize) -> Option<[f32; STEAM_CONTROLS]> {
    if r.len() < pads + 10 {
        return None;
    }
    let buttons = u32::from_le_bytes(r[2..6].try_into().ok()?);
    let axis = |i: usize| f32::from(i16::from_le_bytes([r[i], r[i + 1]])) / 32768.0;
    let mut levels = [0.0; STEAM_CONTROLS];
    for (bit, level) in levels.iter_mut().take(32).enumerate() {
        *level = ((buttons >> bit) & 1) as f32;
    }
    // Gâchettes et sticks, puis les pavés (la pression de chacun est ignorée).
    let offsets = [6, 8, 10, 12, 14, 16, pads, pads + 2, pads + 6, pads + 8];
    for (level, i) in levels[32..42].iter_mut().zip(offsets) {
        *level = axis(i);
    }
    Some(levels)
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

    /// Rapport d'état tel que Windows le donne : numéro 0, en-tête, état.
    fn steam_report(buttons: u32, left_trigger: u8, left_x: i16, gyro: i16) -> Vec<u8> {
        let mut r = vec![0u8; 65];
        r[1..5].copy_from_slice(&[0x01, 0x00, STEAM_REPORT_STATE, 60]);
        r[5..9].copy_from_slice(&7u32.to_le_bytes());
        r[9..12].copy_from_slice(&buttons.to_le_bytes()[..3]);
        r[12] = left_trigger;
        r[17..19].copy_from_slice(&left_x.to_le_bytes());
        r[25..27].copy_from_slice(&u16::from(left_trigger).to_le_bytes());
        r[35..37].copy_from_slice(&gyro.to_le_bytes());
        r
    }

    #[test]
    fn steam_controller_reports_are_read() {
        assert!(is_steam_controller(0x28DE, 0x1142));
        assert!(!is_steam_controller(0x28DE, 0x1205));
        let rest = steam_controller_levels(&steam_report(0, 0, 0, 0)).unwrap();
        assert!(rest.iter().all(|l| *l == 0.0));
        // Bouton A, gâchette gauche à fond, stick à droite.
        let pressed = steam_controller_levels(&steam_report(0x80, 255, 32767, 0)).unwrap();
        assert_eq!((pressed[7], pressed[48], pressed[49]), (1.0, 1.0, 0.0));
        assert!(pressed[50] > 0.99);
        // Sans le numéro de rapport (macOS), même lecture.
        assert_eq!(steam_controller_levels(&steam_report(0x80, 255, 32767, 0)[1..]), Some(pressed));
        // Rapport sans fil (connexion), ou trop court : ignoré.
        let mut wireless = steam_report(0, 0, 0, 0);
        wireless[3] = 0x03;
        assert_eq!(steam_controller_levels(&wireless), None);
        assert_eq!(steam_controller_levels(&[0, 1, 0, 1, 4]), None);
    }

    /// Rapport d'état du nouveau modèle (0x42), tel que Windows le donne.
    fn triton_report(buttons: u32, left_trigger: i16, right_stick_y: i16, gyro: i16) -> Vec<u8> {
        let mut r = vec![0u8; 64];
        r[0] = TRITON_STATE;
        r[1] = 9;
        r[2..6].copy_from_slice(&buttons.to_le_bytes());
        r[6..8].copy_from_slice(&left_trigger.to_le_bytes());
        r[16..18].copy_from_slice(&right_stick_y.to_le_bytes());
        r[40..42].copy_from_slice(&gyro.to_le_bytes());
        r
    }

    #[test]
    fn new_steam_controller_reports_are_read() {
        assert!(is_steam_controller(0x28DE, 0x1304));
        assert!(!is_steam_controller(0x28DE, 0x1205));
        let rest = triton_report(0, 0, 0, 0);
        assert!(steam_controller_levels(&rest).unwrap().iter().all(|l| *l == 0.0));
        // Bouton A (bit 0), gâchette gauche à fond, stick droit vers le haut.
        let pressed = steam_controller_levels(&triton_report(1, 32767, -32768, 0)).unwrap();
        assert_eq!((pressed[0], pressed[37]), (1.0, -1.0));
        assert!(pressed[32] > 0.99);
        // Avec l'horodatage des pavés : boutons et sticks au même endroit.
        let mut ts = triton_report(1, 32767, -32768, 0);
        ts[0] = TRITON_STATE_TIMESTAMP;
        assert_eq!(steam_controller_levels(&ts).map(|l| l[..38].to_vec()), Some(pressed[..38].to_vec()));
        // Batterie, état sans fil, rapport trop court : ignorés.
        assert_eq!(steam_controller_levels(&[0x43, 1, 2, 3]), None);
        assert_eq!(steam_controller_levels(&[0x79, 2]), None);
        assert_eq!(steam_controller_levels(&rest[..20]), None);
        // Posé sur le bureau : seul le gyroscope bouge.
        let mut w = GamepadWatch::default();
        assert!(!w.moved_any(0, &steam_controller_levels(&triton_report(0, 0, 0, 0)).unwrap()));
        assert!(!w.moved_any(0, &steam_controller_levels(&triton_report(0, 0, 0, 9000)).unwrap()));
        assert!(w.moved_any(0, &steam_controller_levels(&triton_report(0, 20000, 0, 0)).unwrap()));
    }

    #[test]
    fn a_still_steam_controller_is_not_activity() {
        let mut w = GamepadWatch::default();
        let levels = |r: Vec<u8>| steam_controller_levels(&r).unwrap();
        assert!(!w.moved_any(0, &levels(steam_report(0, 0, 120, 0))));
        // Posé sur le bureau : seul le gyroscope bouge, avec un peu de bruit du stick.
        assert!(!w.moved_any(0, &levels(steam_report(0, 0, -300, 5000))));
        assert!(!w.moved_any(0, &levels(steam_report(0, 0, 200, -8000))));
        assert!(w.moved_any(0, &levels(steam_report(0x80, 0, 200, 0))));
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
