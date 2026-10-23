//! Raccourcis clavier de Baskul (ex. « Ctrl+Alt+Shift+B »).

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

    /// Combinaison formée par la touche `key` et les modificateurs enfoncés.
    pub fn from_pressed(key: u32, pressed: &HashSet<u32>) -> Hotkey {
        let held = |a: u32, b: u32| pressed.contains(&a) || pressed.contains(&b);
        Hotkey {
            ctrl: held(HID_CONTROL_LEFT, HID_CONTROL_RIGHT),
            shift: held(HID_SHIFT_LEFT, HID_SHIFT_RIGHT),
            alt: held(HID_ALT_LEFT, HID_ALT_RIGHT),
            meta: held(HID_META_LEFT, HID_META_RIGHT),
            key,
        }
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

/// Format de la configuration : « Ctrl+Alt+Shift+B ».
impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let meta = if cfg!(target_os = "macos") { "Cmd" } else { "Win" };
        for (on, name) in [(self.ctrl, "Ctrl"), (self.alt, "Alt"), (self.shift, "Shift"), (self.meta, meta)] {
            if on {
                write!(f, "{name}+")?;
            }
        }
        // « KeyB » → « B », « Digit1 » → « 1 ».
        let name = keymap::name_from_hid(self.key);
        let short = name.strip_prefix("Key").or_else(|| name.strip_prefix("Digit")).filter(|s| s.len() == 1);
        f.write_str(short.unwrap_or(&name))
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

    #[test]
    fn display_roundtrips() {
        for s in ["Shift+F11", "Ctrl+Alt+Shift+B", "Ctrl+1", "ScrollLock", "Alt+ArrowLeft"] {
            let hk = Hotkey::parse(s).unwrap().unwrap();
            assert_eq!(hk.to_string(), s);
        }
        let f11 = keymap::hid_from_name("F11").unwrap();
        let pressed: HashSet<u32> = [HID_SHIFT_RIGHT, f11].into_iter().collect();
        assert_eq!(Hotkey::from_pressed(f11, &pressed).to_string(), "Shift+F11");
    }
}
