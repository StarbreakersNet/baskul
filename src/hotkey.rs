//! Raccourcis clavier de Bascule (ex. « Ctrl+Alt+Shift+B »).

use crate::keymap::{self, *};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotkey {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub key: u32,
}

impl Hotkey {
    /// Analyse une chaîne comme « Ctrl+Alt+Shift+B ». Chaîne vide → `Ok(None)`.
    pub fn parse(s: &str) -> Result<Option<Hotkey>, String> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(None);
        }
        let mut hk = Hotkey { ctrl: false, shift: false, alt: false, meta: false, key: 0 };
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        for (i, part) in parts.iter().enumerate() {
            let last = i == parts.len() - 1;
            let lower = part.to_ascii_lowercase();
            if !last {
                match lower.as_str() {
                    "ctrl" | "control" => hk.ctrl = true,
                    "shift" | "maj" => hk.shift = true,
                    "alt" | "option" | "opt" => hk.alt = true,
                    "win" | "cmd" | "command" | "meta" | "super" => hk.meta = true,
                    _ => return Err(format!("modificateur inconnu « {part} » dans « {s} »")),
                }
            } else {
                hk.key =
                    keymap::hid_from_name(part).ok_or_else(|| format!("touche inconnue « {part} » dans « {s} »"))?;
                if keymap::is_modifier(hk.key) {
                    return Err(format!("« {s} » : la dernière touche ne peut pas être un modificateur"));
                }
            }
        }
        Ok(Some(hk))
    }

    /// La touche `key` vient d'être enfoncée ; `pressed` contient les touches
    /// actuellement enfoncées (y compris `key`).
    pub fn matches(&self, key: u32, pressed: &HashSet<u32>) -> bool {
        if key != self.key {
            return false;
        }
        let held = |a: u32, b: u32| pressed.contains(&a) || pressed.contains(&b);
        held(HID_CONTROL_LEFT, HID_CONTROL_RIGHT) == self.ctrl
            && held(HID_SHIFT_LEFT, HID_SHIFT_RIGHT) == self.shift
            && held(HID_ALT_LEFT, HID_ALT_RIGHT) == self.alt
            && held(HID_META_LEFT, HID_META_RIGHT) == self.meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_match() {
        let hk = Hotkey::parse("Ctrl+Alt+Shift+B").unwrap().unwrap();
        assert!(hk.ctrl && hk.alt && hk.shift && !hk.meta);
        let b = keymap::hid_from_name("B").unwrap();
        let mut pressed: HashSet<u32> = [HID_CONTROL_LEFT, HID_ALT_RIGHT, HID_SHIFT_LEFT, b].into_iter().collect();
        assert!(hk.matches(b, &pressed));
        pressed.insert(HID_META_LEFT);
        assert!(!hk.matches(b, &pressed));
    }

    #[test]
    fn parse_errors() {
        assert_eq!(Hotkey::parse("").unwrap(), None);
        assert!(Hotkey::parse("Ctrl+Truc").is_err());
        assert!(Hotkey::parse("Bidule+B").is_err());
        assert!(Hotkey::parse("Ctrl+Shift").is_err());
        let hk = Hotkey::parse("ScrollLock").unwrap().unwrap();
        assert_eq!(hk.key, HID_SCROLL_LOCK);
    }
}
