//! Table de correspondance des touches.
//!
//! Le protocole transporte uniquement des codes **USB HID** (page << 16 | usage),
//! c'est-à-dire la position *physique* de la touche. Aucun caractère n'est jamais
//! transmis : c'est la disposition clavier de la machine pilotée qui produit les
//! caractères (et compose les touches mortes).
//!
//! Données issues de la table de Chromium (`keycode_converter_data.inc`) :
//! USB HID ↔ scancode Windows (set 1, 0xE0xx pour les touches étendues) ↔ keycode
//! virtuel macOS (kVK_*, convention clavier ANSI).

/// Une entrée de la table.
#[derive(Debug, Clone, Copy)]
pub struct K {
    /// Code USB HID : (page << 16) | usage.
    pub hid: u32,
    /// Scancode Windows (0 = aucun). Touches étendues : 0xE0xx.
    pub win: u16,
    /// Keycode virtuel macOS (0xFFFF = aucun ; attention 0x0000 = touche A).
    pub mac: u16,
    /// Nom (convention UI Events `code`), utilisé dans la configuration.
    pub name: &'static str,
}

/// Valeur « pas de keycode Mac ».
pub const MAC_NONE: u16 = 0xffff;

pub const PAGE_KEYBOARD: u32 = 0x07;
pub const PAGE_CONSUMER: u32 = 0x0c;

pub const fn page(code: u32) -> u32 {
    code >> 16
}

#[rustfmt::skip]
pub static TABLE: &[K] = &[
    K { hid: 0x070004, win: 0x001e, mac: 0x0000, name: "KeyA" },
    K { hid: 0x070005, win: 0x0030, mac: 0x000b, name: "KeyB" },
    K { hid: 0x070006, win: 0x002e, mac: 0x0008, name: "KeyC" },
    K { hid: 0x070007, win: 0x0020, mac: 0x0002, name: "KeyD" },
    K { hid: 0x070008, win: 0x0012, mac: 0x000e, name: "KeyE" },
    K { hid: 0x070009, win: 0x0021, mac: 0x0003, name: "KeyF" },
    K { hid: 0x07000a, win: 0x0022, mac: 0x0005, name: "KeyG" },
    K { hid: 0x07000b, win: 0x0023, mac: 0x0004, name: "KeyH" },
    K { hid: 0x07000c, win: 0x0017, mac: 0x0022, name: "KeyI" },
    K { hid: 0x07000d, win: 0x0024, mac: 0x0026, name: "KeyJ" },
    K { hid: 0x07000e, win: 0x0025, mac: 0x0028, name: "KeyK" },
    K { hid: 0x07000f, win: 0x0026, mac: 0x0025, name: "KeyL" },
    K { hid: 0x070010, win: 0x0032, mac: 0x002e, name: "KeyM" },
    K { hid: 0x070011, win: 0x0031, mac: 0x002d, name: "KeyN" },
    K { hid: 0x070012, win: 0x0018, mac: 0x001f, name: "KeyO" },
    K { hid: 0x070013, win: 0x0019, mac: 0x0023, name: "KeyP" },
    K { hid: 0x070014, win: 0x0010, mac: 0x000c, name: "KeyQ" },
    K { hid: 0x070015, win: 0x0013, mac: 0x000f, name: "KeyR" },
    K { hid: 0x070016, win: 0x001f, mac: 0x0001, name: "KeyS" },
    K { hid: 0x070017, win: 0x0014, mac: 0x0011, name: "KeyT" },
    K { hid: 0x070018, win: 0x0016, mac: 0x0020, name: "KeyU" },
    K { hid: 0x070019, win: 0x002f, mac: 0x0009, name: "KeyV" },
    K { hid: 0x07001a, win: 0x0011, mac: 0x000d, name: "KeyW" },
    K { hid: 0x07001b, win: 0x002d, mac: 0x0007, name: "KeyX" },
    K { hid: 0x07001c, win: 0x0015, mac: 0x0010, name: "KeyY" },
    K { hid: 0x07001d, win: 0x002c, mac: 0x0006, name: "KeyZ" },
    K { hid: 0x07001e, win: 0x0002, mac: 0x0012, name: "Digit1" },
    K { hid: 0x07001f, win: 0x0003, mac: 0x0013, name: "Digit2" },
    K { hid: 0x070020, win: 0x0004, mac: 0x0014, name: "Digit3" },
    K { hid: 0x070021, win: 0x0005, mac: 0x0015, name: "Digit4" },
    K { hid: 0x070022, win: 0x0006, mac: 0x0017, name: "Digit5" },
    K { hid: 0x070023, win: 0x0007, mac: 0x0016, name: "Digit6" },
    K { hid: 0x070024, win: 0x0008, mac: 0x001a, name: "Digit7" },
    K { hid: 0x070025, win: 0x0009, mac: 0x001c, name: "Digit8" },
    K { hid: 0x070026, win: 0x000a, mac: 0x0019, name: "Digit9" },
    K { hid: 0x070027, win: 0x000b, mac: 0x001d, name: "Digit0" },
    K { hid: 0x070028, win: 0x001c, mac: 0x0024, name: "Enter" },
    K { hid: 0x070029, win: 0x0001, mac: 0x0035, name: "Escape" },
    K { hid: 0x07002a, win: 0x000e, mac: 0x0033, name: "Backspace" },
    K { hid: 0x07002b, win: 0x000f, mac: 0x0030, name: "Tab" },
    K { hid: 0x07002c, win: 0x0039, mac: 0x0031, name: "Space" },
    K { hid: 0x07002d, win: 0x000c, mac: 0x001b, name: "Minus" },
    K { hid: 0x07002e, win: 0x000d, mac: 0x0018, name: "Equal" },
    K { hid: 0x07002f, win: 0x001a, mac: 0x0021, name: "BracketLeft" },
    K { hid: 0x070030, win: 0x001b, mac: 0x001e, name: "BracketRight" },
    K { hid: 0x070031, win: 0x002b, mac: 0x002a, name: "Backslash" },
    K { hid: 0x070033, win: 0x0027, mac: 0x0029, name: "Semicolon" },
    K { hid: 0x070034, win: 0x0028, mac: 0x0027, name: "Quote" },
    K { hid: 0x070035, win: 0x0029, mac: 0x0032, name: "Backquote" },
    K { hid: 0x070036, win: 0x0033, mac: 0x002b, name: "Comma" },
    K { hid: 0x070037, win: 0x0034, mac: 0x002f, name: "Period" },
    K { hid: 0x070038, win: 0x0035, mac: 0x002c, name: "Slash" },
    K { hid: 0x070039, win: 0x003a, mac: 0x0039, name: "CapsLock" },
    K { hid: 0x07003a, win: 0x003b, mac: 0x007a, name: "F1" },
    K { hid: 0x07003b, win: 0x003c, mac: 0x0078, name: "F2" },
    K { hid: 0x07003c, win: 0x003d, mac: 0x0063, name: "F3" },
    K { hid: 0x07003d, win: 0x003e, mac: 0x0076, name: "F4" },
    K { hid: 0x07003e, win: 0x003f, mac: 0x0060, name: "F5" },
    K { hid: 0x07003f, win: 0x0040, mac: 0x0061, name: "F6" },
    K { hid: 0x070040, win: 0x0041, mac: 0x0062, name: "F7" },
    K { hid: 0x070041, win: 0x0042, mac: 0x0064, name: "F8" },
    K { hid: 0x070042, win: 0x0043, mac: 0x0065, name: "F9" },
    K { hid: 0x070043, win: 0x0044, mac: 0x006d, name: "F10" },
    K { hid: 0x070044, win: 0x0057, mac: 0x0067, name: "F11" },
    K { hid: 0x070045, win: 0x0058, mac: 0x006f, name: "F12" },
    K { hid: 0x070046, win: 0xe037, mac: 0xffff, name: "PrintScreen" },
    K { hid: 0x070047, win: 0x0046, mac: 0xffff, name: "ScrollLock" },
    K { hid: 0x070048, win: 0x0045, mac: 0xffff, name: "Pause" },
    K { hid: 0x070049, win: 0xe052, mac: 0x0072, name: "Insert" },
    K { hid: 0x07004a, win: 0xe047, mac: 0x0073, name: "Home" },
    K { hid: 0x07004b, win: 0xe049, mac: 0x0074, name: "PageUp" },
    K { hid: 0x07004c, win: 0xe053, mac: 0x0075, name: "Delete" },
    K { hid: 0x07004d, win: 0xe04f, mac: 0x0077, name: "End" },
    K { hid: 0x07004e, win: 0xe051, mac: 0x0079, name: "PageDown" },
    K { hid: 0x07004f, win: 0xe04d, mac: 0x007c, name: "ArrowRight" },
    K { hid: 0x070050, win: 0xe04b, mac: 0x007b, name: "ArrowLeft" },
    K { hid: 0x070051, win: 0xe050, mac: 0x007d, name: "ArrowDown" },
    K { hid: 0x070052, win: 0xe048, mac: 0x007e, name: "ArrowUp" },
    K { hid: 0x070053, win: 0xe045, mac: 0x0047, name: "NumLock" },
    K { hid: 0x070054, win: 0xe035, mac: 0x004b, name: "NumpadDivide" },
    K { hid: 0x070055, win: 0x0037, mac: 0x0043, name: "NumpadMultiply" },
    K { hid: 0x070056, win: 0x004a, mac: 0x004e, name: "NumpadSubtract" },
    K { hid: 0x070057, win: 0x004e, mac: 0x0045, name: "NumpadAdd" },
    K { hid: 0x070058, win: 0xe01c, mac: 0x004c, name: "NumpadEnter" },
    K { hid: 0x070059, win: 0x004f, mac: 0x0053, name: "Numpad1" },
    K { hid: 0x07005a, win: 0x0050, mac: 0x0054, name: "Numpad2" },
    K { hid: 0x07005b, win: 0x0051, mac: 0x0055, name: "Numpad3" },
    K { hid: 0x07005c, win: 0x004b, mac: 0x0056, name: "Numpad4" },
    K { hid: 0x07005d, win: 0x004c, mac: 0x0057, name: "Numpad5" },
    K { hid: 0x07005e, win: 0x004d, mac: 0x0058, name: "Numpad6" },
    K { hid: 0x07005f, win: 0x0047, mac: 0x0059, name: "Numpad7" },
    K { hid: 0x070060, win: 0x0048, mac: 0x005b, name: "Numpad8" },
    K { hid: 0x070061, win: 0x0049, mac: 0x005c, name: "Numpad9" },
    K { hid: 0x070062, win: 0x0052, mac: 0x0052, name: "Numpad0" },
    K { hid: 0x070063, win: 0x0053, mac: 0x0041, name: "NumpadDecimal" },
    K { hid: 0x070064, win: 0x0056, mac: 0x000a, name: "IntlBackslash" },
    K { hid: 0x070065, win: 0xe05d, mac: 0x006e, name: "ContextMenu" },
    K { hid: 0x070066, win: 0xe05e, mac: 0xffff, name: "Power" },
    K { hid: 0x070067, win: 0x0059, mac: 0x0051, name: "NumpadEqual" },
    K { hid: 0x070068, win: 0x0064, mac: 0x0069, name: "F13" },
    K { hid: 0x070069, win: 0x0065, mac: 0x006b, name: "F14" },
    K { hid: 0x07006a, win: 0x0066, mac: 0x0071, name: "F15" },
    K { hid: 0x07006b, win: 0x0067, mac: 0x006a, name: "F16" },
    K { hid: 0x07006c, win: 0x0068, mac: 0x0040, name: "F17" },
    K { hid: 0x07006d, win: 0x0069, mac: 0x004f, name: "F18" },
    K { hid: 0x07006e, win: 0x006a, mac: 0x0050, name: "F19" },
    K { hid: 0x07006f, win: 0x006b, mac: 0x005a, name: "F20" },
    K { hid: 0x070070, win: 0x006c, mac: 0xffff, name: "F21" },
    K { hid: 0x070071, win: 0x006d, mac: 0xffff, name: "F22" },
    K { hid: 0x070072, win: 0x006e, mac: 0xffff, name: "F23" },
    K { hid: 0x070073, win: 0x0076, mac: 0xffff, name: "F24" },
    K { hid: 0x070075, win: 0xe03b, mac: 0xffff, name: "Help" },
    K { hid: 0x07007a, win: 0xe008, mac: 0xffff, name: "Undo" },
    K { hid: 0x07007b, win: 0xe017, mac: 0xffff, name: "Cut" },
    K { hid: 0x07007c, win: 0xe018, mac: 0xffff, name: "Copy" },
    K { hid: 0x07007d, win: 0xe00a, mac: 0xffff, name: "Paste" },
    K { hid: 0x07007f, win: 0xe020, mac: 0x004a, name: "AudioVolumeMute" },
    K { hid: 0x070080, win: 0xe030, mac: 0x0048, name: "AudioVolumeUp" },
    K { hid: 0x070081, win: 0xe02e, mac: 0x0049, name: "AudioVolumeDown" },
    K { hid: 0x070085, win: 0x007e, mac: 0x005f, name: "NumpadComma" },
    K { hid: 0x070087, win: 0x0073, mac: 0x005e, name: "IntlRo" },
    K { hid: 0x070088, win: 0x0070, mac: 0x0068, name: "KanaMode" },
    K { hid: 0x070089, win: 0x007d, mac: 0x005d, name: "IntlYen" },
    K { hid: 0x07008a, win: 0x0079, mac: 0xffff, name: "Convert" },
    K { hid: 0x07008b, win: 0x007b, mac: 0xffff, name: "NonConvert" },
    K { hid: 0x070090, win: 0x0072, mac: 0xffff, name: "Lang1" },
    K { hid: 0x070091, win: 0x0071, mac: 0xffff, name: "Lang2" },
    K { hid: 0x070092, win: 0x0078, mac: 0xffff, name: "Lang3" },
    K { hid: 0x070093, win: 0x0077, mac: 0xffff, name: "Lang4" },
    K { hid: 0x0700e0, win: 0x001d, mac: 0x003b, name: "ControlLeft" },
    K { hid: 0x0700e1, win: 0x002a, mac: 0x0038, name: "ShiftLeft" },
    K { hid: 0x0700e2, win: 0x0038, mac: 0x003a, name: "AltLeft" },
    K { hid: 0x0700e3, win: 0xe05b, mac: 0x0037, name: "MetaLeft" },
    K { hid: 0x0700e4, win: 0xe01d, mac: 0x003e, name: "ControlRight" },
    K { hid: 0x0700e5, win: 0x0036, mac: 0x003c, name: "ShiftRight" },
    K { hid: 0x0700e6, win: 0xe038, mac: 0x003d, name: "AltRight" },
    K { hid: 0x0700e7, win: 0xe05c, mac: 0x0036, name: "MetaRight" },
    K { hid: 0x0c00b5, win: 0xe019, mac: 0xffff, name: "MediaTrackNext" },
    K { hid: 0x0c00b6, win: 0xe010, mac: 0xffff, name: "MediaTrackPrevious" },
    K { hid: 0x0c00b7, win: 0xe024, mac: 0xffff, name: "MediaStop" },
    K { hid: 0x0c00b8, win: 0xe02c, mac: 0xffff, name: "Eject" },
    K { hid: 0x0c00cd, win: 0xe022, mac: 0xffff, name: "MediaPlayPause" },
    K { hid: 0x0c0183, win: 0xe06d, mac: 0xffff, name: "MediaSelect" },
    K { hid: 0x0c018a, win: 0xe06c, mac: 0xffff, name: "LaunchMail" },
    K { hid: 0x0c0192, win: 0xe021, mac: 0xffff, name: "LaunchApp2" },
    K { hid: 0x0c0194, win: 0xe06b, mac: 0xffff, name: "LaunchApp1" },
    K { hid: 0x0c0221, win: 0xe065, mac: 0xffff, name: "BrowserSearch" },
    K { hid: 0x0c0223, win: 0xe032, mac: 0xffff, name: "BrowserHome" },
    K { hid: 0x0c0224, win: 0xe06a, mac: 0xffff, name: "BrowserBack" },
    K { hid: 0x0c0225, win: 0xe069, mac: 0xffff, name: "BrowserForward" },
    K { hid: 0x0c0226, win: 0xe068, mac: 0xffff, name: "BrowserStop" },
    K { hid: 0x0c0227, win: 0xe067, mac: 0xffff, name: "BrowserRefresh" },
    K { hid: 0x0c022a, win: 0xe066, mac: 0xffff, name: "BrowserFavorites" },
];

// Codes HID utilisés directement dans le code.
pub const HID_BACKQUOTE: u32 = 0x070035;
pub const HID_INTL_BACKSLASH: u32 = 0x070064;
pub const HID_CAPS_LOCK: u32 = 0x070039;
pub const HID_PRINT_SCREEN: u32 = 0x070046;
pub const HID_SCROLL_LOCK: u32 = 0x070047;
pub const HID_PAUSE: u32 = 0x070048;
pub const HID_NUM_LOCK: u32 = 0x070053;
pub const HID_CONTROL_LEFT: u32 = 0x0700e0;
pub const HID_SHIFT_LEFT: u32 = 0x0700e1;
pub const HID_ALT_LEFT: u32 = 0x0700e2;
pub const HID_META_LEFT: u32 = 0x0700e3;
pub const HID_CONTROL_RIGHT: u32 = 0x0700e4;
pub const HID_SHIFT_RIGHT: u32 = 0x0700e5;
pub const HID_ALT_RIGHT: u32 = 0x0700e6;
pub const HID_META_RIGHT: u32 = 0x0700e7;
pub const HID_VOLUME_MUTE: u32 = 0x07007f;
pub const HID_VOLUME_UP: u32 = 0x070080;
pub const HID_VOLUME_DOWN: u32 = 0x070081;
pub const HID_MEDIA_NEXT: u32 = 0x0c00b5;
pub const HID_MEDIA_PREVIOUS: u32 = 0x0c00b6;
pub const HID_MEDIA_STOP: u32 = 0x0c00b7;
pub const HID_EJECT: u32 = 0x0c00b8;
pub const HID_MEDIA_PLAY_PAUSE: u32 = 0x0c00cd;
pub const HID_BROWSER_BACK: u32 = 0x0c0224;
pub const HID_BROWSER_FORWARD: u32 = 0x0c0225;

/// Keycodes macOS (HIToolbox/Events.h) utilisés directement.
pub mod mac {
    pub const ANSI_GRAVE: u16 = 0x32;
    pub const ISO_SECTION: u16 = 0x0a;
    pub const ANSI_LEFT_BRACKET: u16 = 0x21;
    pub const ANSI_RIGHT_BRACKET: u16 = 0x1e;
    pub const COMMAND: u16 = 0x37;
    pub const SHIFT: u16 = 0x38;
    pub const CAPS_LOCK: u16 = 0x39;
    pub const OPTION: u16 = 0x3a;
    pub const CONTROL: u16 = 0x3b;
    pub const RIGHT_COMMAND: u16 = 0x36;
    pub const RIGHT_SHIFT: u16 = 0x3c;
    pub const RIGHT_OPTION: u16 = 0x3d;
    pub const RIGHT_CONTROL: u16 = 0x3e;
    pub const FUNCTION: u16 = 0x3f;
    pub const F13: u16 = 0x69;
    pub const F14: u16 = 0x6b;
    pub const F15: u16 = 0x71;
}

/// Est-ce une touche de modification (Ctrl, Shift, Alt, Win/Cmd) ?
pub fn is_modifier(hid: u32) -> bool {
    (HID_CONTROL_LEFT..=HID_META_RIGHT).contains(&hid)
}

/// Touches multimédia (volume, lecture…).
pub fn is_media(hid: u32) -> bool {
    matches!(
        hid,
        HID_VOLUME_MUTE
            | HID_VOLUME_UP
            | HID_VOLUME_DOWN
            | HID_MEDIA_NEXT
            | HID_MEDIA_PREVIOUS
            | HID_MEDIA_STOP
            | HID_EJECT
            | HID_MEDIA_PLAY_PAUSE
    )
}

pub fn entry_by_hid(hid: u32) -> Option<&'static K> {
    TABLE.iter().find(|k| k.hid == hid)
}

/// Scancode Windows (0xE0xx pour les touches étendues) → HID.
pub fn hid_from_win(scancode: u16) -> Option<u32> {
    if scancode == 0 {
        return None;
    }
    if let Some(k) = TABLE.iter().find(|k| k.win == scancode) {
        return Some(k.hid);
    }
    // Variantes produites par certaines combinaisons.
    match scancode {
        0x0054 => Some(HID_PRINT_SCREEN), // Alt+Impr.écran (SysRq)
        0xe046 => Some(HID_PAUSE),        // Ctrl+Pause (Break)
        0xe045 => Some(HID_NUM_LOCK),
        // Le crochet clavier bas niveau de Windows marque le Maj droit comme
        // touche étendue.
        0xe036 => Some(HID_SHIFT_RIGHT),
        _ => None,
    }
}

/// HID → scancode Windows.
pub fn win_from_hid(hid: u32) -> Option<u16> {
    entry_by_hid(hid).map(|k| k.win).filter(|w| *w != 0)
}

/// HID → keycode macOS (convention ANSI ; l'échange ISO est géré par l'appelant).
pub fn mac_from_hid(hid: u32) -> Option<u16> {
    if let Some(k) = entry_by_hid(hid) {
        if k.mac != MAC_NONE {
            return Some(k.mac);
        }
    }
    // Touches absentes des claviers Apple : convention Apple des claviers
    // étendus (F13/F14/F15 à la place de Impr.écran / Arrêt défil / Pause).
    match hid {
        HID_PRINT_SCREEN => Some(mac::F13),
        HID_SCROLL_LOCK => Some(mac::F14),
        HID_PAUSE => Some(mac::F15),
        _ => None,
    }
}

/// Keycode macOS → HID (convention ANSI).
pub fn hid_from_mac(keycode: u16) -> Option<u32> {
    if keycode == MAC_NONE {
        return None;
    }
    TABLE.iter().find(|k| k.mac == keycode).map(|k| k.hid)
}

/// Échange des deux touches qui diffèrent entre claviers ANSI et ISO sur Mac.
///
/// Sur un Mac à clavier ISO (cas des claviers français), la touche à gauche du
/// « 1 » produit le keycode 0x0A (kVK_ISO_Section) et la touche à droite du Shift
/// gauche produit 0x32 (kVK_ANSI_Grave) — l'inverse de la convention ANSI de la
/// table. Cette fonction applique l'échange dans les deux sens.
pub fn iso_swap_mac(keycode: u16) -> u16 {
    match keycode {
        mac::ANSI_GRAVE => mac::ISO_SECTION,
        mac::ISO_SECTION => mac::ANSI_GRAVE,
        k => k,
    }
}

/// Nom (ou alias) → HID. Insensible à la casse.
pub fn hid_from_name(name: &str) -> Option<u32> {
    let n = name.trim();
    if n.is_empty() {
        return None;
    }
    if let Some(k) = TABLE.iter().find(|k| k.name.eq_ignore_ascii_case(n)) {
        return Some(k.hid);
    }
    // Code numérique : 0x070039
    if let Some(hex) = n.strip_prefix("0x").or_else(|| n.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16).ok();
    }
    let lower = n.to_ascii_lowercase();
    // Lettres et chiffres seuls : "B" → KeyB, "1" → Digit1.
    if n.len() == 1 {
        let c = n.chars().next()?.to_ascii_uppercase();
        if c.is_ascii_uppercase() {
            return hid_from_name(&format!("Key{c}"));
        }
        if c.is_ascii_digit() {
            return hid_from_name(&format!("Digit{c}"));
        }
    }
    let alias = match lower.as_str() {
        "esc" | "echap" | "échap" => "Escape",
        "ctrl" | "control" | "lctrl" => "ControlLeft",
        "rctrl" => "ControlRight",
        "shift" | "lshift" => "ShiftLeft",
        "rshift" => "ShiftRight",
        "alt" | "lalt" | "option" => "AltLeft",
        "altgr" | "ralt" => "AltRight",
        "win" | "cmd" | "command" | "meta" | "super" => "MetaLeft",
        "del" | "suppr" => "Delete",
        "ins" => "Insert",
        "pgup" => "PageUp",
        "pgdn" | "pgdown" => "PageDown",
        "left" => "ArrowLeft",
        "right" => "ArrowRight",
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "return" | "entrée" | "entree" => "Enter",
        "capslock" | "verrmaj" => "CapsLock",
        "scrolllock" | "arrêtdéfil" | "arretdefil" => "ScrollLock",
        "printscreen" | "prtsc" | "impr" => "PrintScreen",
        "menu" | "apps" => "ContextMenu",
        _ => return None,
    };
    hid_from_name(alias)
}

/// HID → nom lisible (pour les logs).
pub fn name_from_hid(hid: u32) -> String {
    match entry_by_hid(hid) {
        Some(k) => k.name.to_string(),
        None => format!("0x{hid:06x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_unique_hid_codes() {
        for (i, a) in TABLE.iter().enumerate() {
            for b in &TABLE[i + 1..] {
                assert_ne!(a.hid, b.hid, "doublon HID {}", a.name);
            }
        }
    }

    #[test]
    fn table_has_unique_win_scancodes() {
        for (i, a) in TABLE.iter().enumerate() {
            if a.win == 0 {
                continue;
            }
            for b in &TABLE[i + 1..] {
                assert_ne!(a.win, b.win, "doublon scancode {} / {}", a.name, b.name);
            }
        }
    }

    #[test]
    fn table_has_unique_mac_keycodes() {
        for (i, a) in TABLE.iter().enumerate() {
            if a.mac == MAC_NONE {
                continue;
            }
            for b in &TABLE[i + 1..] {
                assert_ne!(a.mac, b.mac, "doublon keycode mac {} / {}", a.name, b.name);
            }
        }
    }

    #[test]
    fn letters_roundtrip() {
        // A : scancode 0x1E, HID 0x070004, keycode mac 0x00
        assert_eq!(hid_from_win(0x1e), Some(0x070004));
        assert_eq!(mac_from_hid(0x070004), Some(0x00));
        assert_eq!(hid_from_mac(0x00), Some(0x070004));
    }

    #[test]
    fn modifiers_are_physical() {
        // Ctrl gauche reste Ctrl, Win → Cmd, Alt → Option, AltGr → Option droite.
        assert_eq!(mac_from_hid(hid_from_win(0x001d).unwrap()), Some(mac::CONTROL));
        assert_eq!(mac_from_hid(hid_from_win(0xe05b).unwrap()), Some(mac::COMMAND));
        assert_eq!(mac_from_hid(hid_from_win(0x0038).unwrap()), Some(mac::OPTION));
        assert_eq!(mac_from_hid(hid_from_win(0xe038).unwrap()), Some(mac::RIGHT_OPTION));
        assert_eq!(mac_from_hid(hid_from_win(0xe01d).unwrap()), Some(mac::RIGHT_CONTROL));
    }

    #[test]
    fn extended_keys() {
        assert_eq!(hid_from_win(0xe048), Some(0x070052)); // flèche haut
        assert_eq!(hid_from_win(0x0048), Some(0x070060)); // pavé num 8
        assert_eq!(hid_from_win(0xe01c), Some(0x070058)); // Entrée pavé num
        assert_eq!(hid_from_win(0x0054), Some(HID_PRINT_SCREEN));
        assert_eq!(hid_from_win(0x0045), Some(HID_PAUSE));
        assert_eq!(hid_from_win(0xe045), Some(HID_NUM_LOCK));
        // Maj droit, avec ou sans le drapeau « étendu ».
        assert_eq!(hid_from_win(0x0036), Some(HID_SHIFT_RIGHT));
        assert_eq!(mac_from_hid(hid_from_win(0xe036).unwrap()), Some(mac::RIGHT_SHIFT));
    }

    #[test]
    fn iso_keys() {
        // Touche « ² » (gauche du 1) et touche « < > » (droite du Shift gauche).
        assert_eq!(hid_from_win(0x0029), Some(HID_BACKQUOTE));
        assert_eq!(hid_from_win(0x0056), Some(HID_INTL_BACKSLASH));
        assert_eq!(iso_swap_mac(mac_from_hid(HID_BACKQUOTE).unwrap()), mac::ISO_SECTION);
        assert_eq!(iso_swap_mac(mac_from_hid(HID_INTL_BACKSLASH).unwrap()), mac::ANSI_GRAVE);
    }

    #[test]
    fn apple_fallbacks() {
        assert_eq!(mac_from_hid(HID_PRINT_SCREEN), Some(mac::F13));
        assert_eq!(mac_from_hid(HID_PAUSE), Some(mac::F15));
    }

    #[test]
    fn names() {
        assert_eq!(hid_from_name("b"), Some(0x070005));
        assert_eq!(hid_from_name("KeyB"), Some(0x070005));
        assert_eq!(hid_from_name("F12"), Some(0x070045));
        assert_eq!(hid_from_name("escape"), Some(0x070029));
        assert_eq!(hid_from_name("Ctrl"), Some(HID_CONTROL_LEFT));
        assert_eq!(hid_from_name("0x070039"), Some(HID_CAPS_LOCK));
        assert_eq!(hid_from_name("nimporte"), None);
    }
}
