//! macOS.
//!
//! - Machine pilotée : injection d'événements Quartz (`CGEventPost`).
//!   Les touches sont envoyées par **keycode** (position physique) : c'est la
//!   disposition clavier du Mac qui produit les caractères et compose les
//!   touches mortes, exactement comme avec un clavier branché sur le Mac.
//! - Machine principale (sens inverse, expérimental) : `CGEventTap`.
//! - Icône dans la barre des menus.

use crate::Args;
use crate::app::{self, ClipboardHooks, Services};
use crate::config::MachineCfg;
use crate::config::{IsoSwap, ScrollCfg, ScrollMode, Settings, SideButtonsMode, Smoothness, TrayIcon};
use crate::geometry::{Layout, Point, Rect};
use crate::i18n;
use crate::icon::{self, Tone};
use crate::keymap::{self, *};
use crate::net::LinkHandle;
use crate::net::PeerId;
use crate::primary::{Decision, Host, PeerEvent, PrimaryCore, PrimaryOptions};
use crate::protocol::{ClipHeader, ClipImage, ImageFormat, Message, MouseButton, Role, SwitchTo};
use crate::secondary::Emulator;
use crate::smooth;
use crate::status::{self, Conn, Snapshot};
use crossbeam_channel::Receiver;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{AnyObject, NSObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSAnimatablePropertyContainer, NSAnimationContext, NSBackingStoreType, NSBox, NSBoxType, NSColor, NSFont, NSPanel,
    NSPopUpMenuWindowLevel, NSScreen, NSTextField, NSTitlePosition, NSView, NSWindowCollectionBehavior,
    NSWindowStyleMask,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSBitmapImageFileType, NSBitmapImageRep,
    NSControlStateValueOff, NSControlStateValueOn, NSEvent, NSEventModifierFlags, NSEventType, NSImage, NSMenu,
    NSMenuItem, NSPasteboard, NSPasteboardItem, NSPasteboardTypeFileURL, NSPasteboardTypeHTML, NSPasteboardTypeString,
    NSPasteboardTypeTIFF, NSPasteboardWriting, NSStatusBar, NSStatusItem, NSWindowOcclusionState,
};
use objc2_foundation::{
    NSActivityOptions, NSArray, NSData, NSDictionary, NSObjectProtocol, NSPoint, NSProcessInfo, NSRect, NSSize,
    NSString, NSTimer, NSURL,
};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ffi::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Fonctions système (CoreGraphics, CoreFoundation, IOKit, Carbon)
// ---------------------------------------------------------------------------

type CGEventRef = *mut c_void;
type CGEventSourceRef = *mut c_void;
type CFTypeRef = *const c_void;
type CFDataRef = *const c_void;
type CFStringRef = *const c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

type TapCallback = unsafe extern "C" fn(*mut c_void, u32, CGEventRef, *mut c_void) -> CGEventRef;

#[cfg_attr(target_os = "macos", link(name = "CoreGraphics", kind = "framework"))]
unsafe extern "C" {
    fn CGEventSourceCreate(state: i32) -> CGEventSourceRef;
    fn CGEventSourceSetLocalEventsSuppressionInterval(source: CGEventSourceRef, seconds: f64);
    fn CGEventSourceFlagsState(state: i32) -> u64;
    fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    fn CGEventSourceSecondsSinceLastEventType(state: i32, ty: u32) -> f64;
    fn CGEventCreate(source: CGEventSourceRef) -> CGEventRef;
    fn CGEventCreateKeyboardEvent(source: CGEventSourceRef, keycode: u16, down: bool) -> CGEventRef;
    fn CGEventCreateMouseEvent(source: CGEventSourceRef, ty: u32, pos: CGPoint, button: u32) -> CGEventRef;
    fn CGEventCreateScrollWheelEvent2(
        source: CGEventSourceRef,
        units: u32,
        count: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> CGEventRef;
    fn CGEventCreateData(alloc: *const c_void, ev: CGEventRef) -> CFDataRef;
    fn CGEventCreateFromData(alloc: *const c_void, data: CFDataRef) -> CGEventRef;
    fn CGEventSetType(ev: CGEventRef, ty: u32);
    fn CGEventSetFlags(ev: CGEventRef, flags: u64);
    fn CGEventGetFlags(ev: CGEventRef) -> u64;
    fn CGEventSetIntegerValueField(ev: CGEventRef, field: u32, value: i64);
    fn CGEventSetDoubleValueField(ev: CGEventRef, field: u32, value: f64);
    fn CGEventGetIntegerValueField(ev: CGEventRef, field: u32) -> i64;
    fn CGEventGetDoubleValueField(ev: CGEventRef, field: u32) -> f64;
    fn CGEventGetLocation(ev: CGEventRef) -> CGPoint;
    fn CGEventPost(tap: u32, ev: CGEventRef);
    fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayBounds(display: u32) -> CGRect;
    fn CGMainDisplayID() -> u32;
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
    fn CGWarpMouseCursorPosition(p: CGPoint) -> i32;
    fn CGAssociateMouseAndMouseCursorPosition(connected: bool) -> i32;
    fn CGDisplayHideCursor(display: u32) -> i32;
    fn CGDisplayShowCursor(display: u32) -> i32;
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        mask: u64,
        callback: TapCallback,
        user: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

#[repr(C)]
struct CFRunLoopSourceContext {
    version: isize,
    info: *mut c_void,
    retain: *const c_void,
    release: *const c_void,
    copy_description: *const c_void,
    equal: *const c_void,
    hash: *const c_void,
    schedule: *const c_void,
    cancel: *const c_void,
    perform: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[cfg_attr(target_os = "macos", link(name = "CoreFoundation", kind = "framework"))]
unsafe extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFDataGetLength(d: CFDataRef) -> isize;
    fn CFDataGetBytePtr(d: CFDataRef) -> *const u8;
    fn CFDataCreate(alloc: *const c_void, bytes: *const u8, len: isize) -> CFDataRef;
    fn CFStringCreateWithCString(alloc: *const c_void, s: *const c_char, encoding: u32) -> CFStringRef;
    fn CFRunLoopGetMain() -> CFRunLoopRef;
    fn CFRunLoopWakeUp(rl: CFRunLoopRef);
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopSourceCreate(
        alloc: *const c_void,
        order: isize,
        ctx: *mut CFRunLoopSourceContext,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopSourceSignal(source: CFRunLoopSourceRef);
    fn CFMachPortCreateRunLoopSource(alloc: *const c_void, port: CFMachPortRef, order: isize) -> CFRunLoopSourceRef;
    static kCFRunLoopCommonModes: CFStringRef;
    static kCFBooleanTrue: CFTypeRef;
}

#[cfg_attr(target_os = "macos", link(name = "IOKit", kind = "framework"))]
unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> *mut c_void;
    fn IOServiceGetMatchingService(main_port: u32, matching: *mut c_void) -> u32;
    fn IOServiceOpen(service: u32, owning_task: u32, ty: u32, connect: *mut u32) -> i32;
    fn IOObjectRelease(obj: u32) -> i32;
    fn IOPMAssertionDeclareUserActivity(name: CFStringRef, ty: u32, id: *mut u32) -> i32;
}

unsafe extern "C" {
    /// Port de la tâche courante (ce que renvoie la macro `mach_task_self()`).
    static mach_task_self_: u32;
}

// Fonctions privées ou dépréciées : résolues à l'exécution (dlsym) plutôt
// qu'à l'édition de liens, pour que la compilation ne dépende jamais de leur
// présence dans le SDK. Si l'une manque, Baskul utilise un repli.
#[cfg_attr(target_os = "macos", link(name = "Carbon", kind = "framework"))]
unsafe extern "C" {}

type FnModLockGet = unsafe extern "C" fn(u32, i32, *mut bool) -> i32;
type FnModLockSet = unsafe extern "C" fn(u32, i32, bool) -> i32;
type FnCgsDefault = unsafe extern "C" fn() -> i32;
type FnCgsSetProp = unsafe extern "C" fn(i32, i32, CFStringRef, CFTypeRef) -> i32;
type FnLmKbdType = unsafe extern "C" fn() -> u8;
type FnKbLayoutType = unsafe extern "C" fn(i16) -> u32;

struct DynSyms {
    mod_lock_get: Option<FnModLockGet>,
    mod_lock_set: Option<FnModLockSet>,
    cgs_default: Option<FnCgsDefault>,
    cgs_set_prop: Option<FnCgsSetProp>,
    lm_kbd_type: Option<FnLmKbdType>,
    kb_layout_type: Option<FnKbLayoutType>,
}

fn lookup(name: &std::ffi::CStr) -> Option<*mut c_void> {
    // SAFETY: recherche d'un symbole dans les bibliothèques déjà chargées.
    let p = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr()) };
    if p.is_null() {
        log::debug!("symbole {name:?} introuvable");
        None
    } else {
        Some(p)
    }
}

fn syms() -> &'static DynSyms {
    static SYMS: std::sync::OnceLock<DynSyms> = std::sync::OnceLock::new();
    SYMS.get_or_init(|| {
        // SAFETY: chaque pointeur provient de dlsym pour une fonction C dont la
        // signature correspond au type indiqué.
        unsafe {
            DynSyms {
                mod_lock_get: lookup(c"IOHIDGetModifierLockState")
                    .map(|p| std::mem::transmute::<*mut c_void, FnModLockGet>(p)),
                mod_lock_set: lookup(c"IOHIDSetModifierLockState")
                    .map(|p| std::mem::transmute::<*mut c_void, FnModLockSet>(p)),
                cgs_default: lookup(c"_CGSDefaultConnection")
                    .map(|p| std::mem::transmute::<*mut c_void, FnCgsDefault>(p)),
                cgs_set_prop: lookup(c"CGSSetConnectionProperty")
                    .map(|p| std::mem::transmute::<*mut c_void, FnCgsSetProp>(p)),
                lm_kbd_type: lookup(c"LMGetKbdType").map(|p| std::mem::transmute::<*mut c_void, FnLmKbdType>(p)),
                kb_layout_type: lookup(c"KBGetLayoutType")
                    .map(|p| std::mem::transmute::<*mut c_void, FnKbLayoutType>(p)),
            }
        }
    })
}

fn get_caps_lock(conn: u32) -> Option<bool> {
    let f = syms().mod_lock_get?;
    let mut st = false;
    // SAFETY: connexion IOHIDSystem ouverte ; pointeur valide.
    (unsafe { f(conn, 1, &mut st) } == 0).then_some(st)
}

fn set_caps_lock(conn: u32, on: bool) -> bool {
    match syms().mod_lock_set {
        // SAFETY: connexion IOHIDSystem ouverte.
        Some(f) => unsafe { f(conn, 1, on) == 0 },
        None => false,
    }
}

#[cfg_attr(target_os = "macos", link(name = "ApplicationServices", kind = "framework"))]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

// Types d'événements (CGEventType).
const EV_LEFT_DOWN: u32 = 1;
const EV_LEFT_UP: u32 = 2;
const EV_RIGHT_DOWN: u32 = 3;
const EV_RIGHT_UP: u32 = 4;
const EV_MOUSE_MOVED: u32 = 5;
const EV_LEFT_DRAGGED: u32 = 6;
const EV_RIGHT_DRAGGED: u32 = 7;
const EV_KEY_DOWN: u32 = 10;
const EV_KEY_UP: u32 = 11;
const EV_FLAGS_CHANGED: u32 = 12;
const EV_SYSTEM_DEFINED: u32 = 14;
const EV_SCROLL: u32 = 22;
const EV_OTHER_DOWN: u32 = 25;
const EV_OTHER_UP: u32 = 26;
const EV_OTHER_DRAGGED: u32 = 27;
const EV_GESTURE: u32 = 29;
const EV_TAP_DISABLED_TIMEOUT: u32 = 0xFFFF_FFFE;
const EV_TAP_DISABLED_USER: u32 = 0xFFFF_FFFF;

// Champs (CGEventField).
const F_CLICK_STATE: u32 = 1;
const F_BUTTON_NUMBER: u32 = 3;
const F_DELTA_X: u32 = 4;
const F_DELTA_Y: u32 = 5;
const F_AUTOREPEAT: u32 = 8;
const F_KEYCODE: u32 = 9;
const F_SCROLL_DELTA_1: u32 = 11;
const F_SCROLL_DELTA_2: u32 = 12;
const F_SCROLL_IS_CONTINUOUS: u32 = 88;
const F_SCROLL_FIXED_1: u32 = 93;
const F_SCROLL_FIXED_2: u32 = 94;
const F_SCROLL_POINT_1: u32 = 96;
const F_SCROLL_POINT_2: u32 = 97;

// Drapeaux (CGEventFlags) et bits « par touche » (NX_DEVICE*).
const FL_ALPHA_SHIFT: u64 = 0x0001_0000;
const FL_SHIFT: u64 = 0x0002_0000;
const FL_CONTROL: u64 = 0x0004_0000;
const FL_ALTERNATE: u64 = 0x0008_0000;
const FL_COMMAND: u64 = 0x0010_0000;
const FL_NUMERIC_PAD: u64 = 0x0020_0000;
const FL_SECONDARY_FN: u64 = 0x0080_0000;
const FL_NON_COALESCED: u64 = 0x0000_0100;
const DEV_LCTL: u64 = 0x0001;
const DEV_LSHIFT: u64 = 0x0002;
const DEV_RSHIFT: u64 = 0x0004;
const DEV_LCMD: u64 = 0x0008;
const DEV_RCMD: u64 = 0x0010;
const DEV_LALT: u64 = 0x0020;
const DEV_RALT: u64 = 0x0040;
const DEV_RCTL: u64 = 0x2000;

const TAP_HID: u32 = 0;
const TAP_SESSION: u32 = 1;
const SOURCE_HID_STATE: i32 = 1;
const SCROLL_UNIT_PIXEL: u32 = 0;
const SCROLL_UNIT_LINE: u32 = 1;
const KEYBOARD_ISO: u32 = 0x4953_4F20; // 'ISO '
const CF_UTF8: u32 = 0x0800_0100;

fn cf_string(s: &std::ffi::CStr) -> CFStringRef {
    // SAFETY: chaîne C valide ; l'appelant libère le résultat.
    unsafe { CFStringCreateWithCString(null(), s.as_ptr(), CF_UTF8) }
}

fn current_cursor() -> Option<Point> {
    // SAFETY: événement temporaire libéré aussitôt.
    unsafe {
        let ev = CGEventCreate(null_mut());
        if ev.is_null() {
            return None;
        }
        let p = CGEventGetLocation(ev);
        CFRelease(ev);
        Some(Point::new(p.x.round() as i32, p.y.round() as i32))
    }
}

fn displays() -> Layout {
    let mut ids = [0u32; 16];
    let mut n = 0u32;
    // SAFETY: tableau de 16 identifiants.
    let err = unsafe { CGGetActiveDisplayList(ids.len() as u32, ids.as_mut_ptr(), &mut n) };
    if err != 0 {
        return Layout::default();
    }
    let rects = ids[..n as usize]
        .iter()
        .map(|id| {
            // SAFETY: identifiant d'écran actif.
            let b = unsafe { CGDisplayBounds(*id) };
            let l = b.origin.x.round() as i32;
            let t = b.origin.y.round() as i32;
            Rect::new(l, t, l + b.size.width.round() as i32, t + b.size.height.round() as i32)
        })
        .collect();
    Layout::new(rects)
}

/// Le Mac utilise-t-il un clavier ISO (claviers européens) ?
fn keyboard_is_iso() -> (bool, u8) {
    let s = syms();
    let (Some(kbd), Some(layout)) = (s.lm_kbd_type, s.kb_layout_type) else {
        return (false, 0);
    };
    // SAFETY: fonctions Carbon sans précondition.
    unsafe {
        let t = kbd();
        (layout(t as i16) == KEYBOARD_ISO, t)
    }
}

fn post_access() -> bool {
    // SAFETY: fonction sans précondition.
    unsafe { CGPreflightPostEventAccess() }
}

// ---------------------------------------------------------------------------
// Machine pilotée : émulation
// ---------------------------------------------------------------------------

/// Drapeaux d'une touche de modification (0 si ce n'en est pas une).
fn modifier_flags(kc: u16) -> u64 {
    match kc {
        mac::SHIFT => FL_SHIFT | DEV_LSHIFT,
        mac::RIGHT_SHIFT => FL_SHIFT | DEV_RSHIFT,
        mac::CONTROL => FL_CONTROL | DEV_LCTL,
        mac::RIGHT_CONTROL => FL_CONTROL | DEV_RCTL,
        mac::OPTION => FL_ALTERNATE | DEV_LALT,
        mac::RIGHT_OPTION => FL_ALTERNATE | DEV_RALT,
        mac::COMMAND => FL_COMMAND | DEV_LCMD,
        mac::RIGHT_COMMAND => FL_COMMAND | DEV_RCMD,
        mac::FUNCTION => FL_SECONDARY_FN,
        _ => 0,
    }
}

/// Drapeaux que portent naturellement certaines touches sur un Mac.
fn key_extra_flags(kc: u16) -> u64 {
    match kc {
        0x7b..=0x7e => FL_NUMERIC_PAD | FL_SECONDARY_FN, // flèches
        0x7a | 0x78 | 0x63 | 0x76 | 0x60 | 0x61 | 0x62 | 0x64 | 0x65 | 0x6d | 0x67 | 0x6f | 0x69 | 0x6b | 0x71
        | 0x6a | 0x40 | 0x4f | 0x50 | 0x5a => FL_SECONDARY_FN, // F1–F20
        0x72..=0x77 | 0x79 => FL_SECONDARY_FN,           // Aide/Inser, Début, PgPréc, Suppr, Fin, PgSuiv
        0x41 | 0x43 | 0x45 | 0x47 | 0x4b | 0x4c | 0x4e | 0x51..=0x59 | 0x5b | 0x5c | 0x5f => FL_NUMERIC_PAD,
        _ => 0,
    }
}

/// Touches multimédia → code NX_KEYTYPE de macOS.
fn media_type(hid: u32) -> Option<i64> {
    Some(match hid {
        HID_VOLUME_UP => 0,
        HID_VOLUME_DOWN => 1,
        HID_VOLUME_MUTE => 7,
        HID_EJECT => 14,
        HID_MEDIA_PLAY_PAUSE => 16,
        HID_MEDIA_NEXT => 17,
        HID_MEDIA_PREVIOUS => 18,
        _ => return None,
    })
}

/// Envoie une touche multimédia (événement « system defined », sous-type 8).
fn post_media(nx_type: i64, down: bool) {
    autoreleasepool(|_| {
        let state: i64 = if down { 0xa } else { 0xb };
        let data1 = (nx_type << 16) | (state << 8);
        let ev = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
            NSEventType::SystemDefined,
            NSPoint::new(0.0, 0.0),
            NSEventModifierFlags(if down { 0xa00 } else { 0xb00 }),
            0.0,
            0,
            None,
            8,
            data1 as isize,
            -1,
        );
        if let Some(ev) = ev {
            // SAFETY: `CGEvent` renvoie un CGEventRef possédé par l'NSEvent.
            let cg: *mut c_void = unsafe { msg_send![&*ev, CGEvent] };
            if !cg.is_null() {
                // SAFETY: événement valide.
                unsafe { CGEventPost(TAP_HID, cg) };
            }
        }
    });
}

/// Geste « balayage » précédent/suivant, comme SensibleSideButtons
/// (direction : 4 = gauche/précédent, 8 = droite/suivant).
fn post_swipe(direction: u8) {
    // SAFETY: création, modification et libération d'objets CoreGraphics /
    // CoreFoundation ; chaque objet créé est libéré.
    unsafe {
        for phase in [1u8, 4u8] {
            let base = CGEventCreate(null_mut());
            if base.is_null() {
                return;
            }
            CGEventSetType(base, EV_GESTURE);
            CGEventSetFlags(base, FL_NON_COALESCED);
            let data = CGEventCreateData(null(), base);
            CFRelease(base);
            if data.is_null() {
                return;
            }
            let len = CFDataGetLength(data) as usize;
            let mut bytes = std::slice::from_raw_parts(CFDataGetBytePtr(data), len).to_vec();
            CFRelease(data);
            // Champs ajoutés : sous-type « swipe », phase, et direction en fin de geste.
            bytes.extend_from_slice(&[0x00, 0x01, 0x40, 0x6e, 0, 0, 0, 0x10]);
            bytes.extend_from_slice(&[0x00, 0x01, 0x40, 0x84, 0, 0, 0, phase]);
            if phase == 4 {
                bytes.extend_from_slice(&[0x00, 0x01, 0x40, 0x73, 0, 0, 0, direction]);
            }
            let nd = CFDataCreate(null(), bytes.as_ptr(), bytes.len() as isize);
            if nd.is_null() {
                return;
            }
            let ev = CGEventCreateFromData(null(), nd);
            CFRelease(nd);
            if ev.is_null() {
                log::warn!("création du geste de balayage impossible");
                return;
            }
            CGEventPost(TAP_HID, ev);
            CFRelease(ev);
        }
    }
}

struct MacEmulator {
    source: CGEventSourceRef,
    /// Touches (hors modificateurs) enfoncées, par keycode.
    pressed: HashSet<u16>,
    /// Modificateurs enfoncés, par keycode.
    mods_down: HashSet<u16>,
    caps_on: bool,
    buttons: [bool; 5],
    pos: Point,
    click_button: Option<MouseButton>,
    click_time: Instant,
    click_pos: Point,
    click_count: i64,
    double_click: Duration,
    scroll: ScrollCfg,
    acc_x: f64,
    acc_y: f64,
    side: SideButtonsMode,
    /// Animation du défilement fluide, démarrée au premier cran.
    smooth: Option<SmoothScroller>,
    iso: IsoSwap,
    iso_active: bool,
    hid_conn: u32,
    last_activity: Option<Instant>,
}

// SAFETY: les objets CoreGraphics utilisés sont utilisables depuis n'importe
// quel thread ; l'émulateur n'est utilisé que par un thread à la fois (Mutex).
unsafe impl Send for MacEmulator {}

/// Défilement fluide : un thread anime les crans reçus (voir `smooth.rs`) et
/// envoie des défilements continus au pixel, comme Mac Mouse Fix.
struct SmoothScroller {
    /// Cran : instant de réception, sens, fraction de cran, modificateurs.
    tx: crossbeam_channel::Sender<(Instant, smooth::Dir, f64, u64)>,
}

impl SmoothScroller {
    fn start(cfg: &ScrollCfg) -> SmoothScroller {
        let (tx, rx) = crossbeam_channel::unbounded::<(Instant, smooth::Dir, f64, u64)>();
        let mut scroller = smooth::Scroller::new(smooth::Options {
            speed: cfg.smooth_speed,
            smoothness: cfg.smoothness,
            precise: cfg.smooth_precision,
            trackpad: cfg.trackpad_simulation && cfg.smoothness == Smoothness::High,
        });
        let spawned = std::thread::Builder::new().name("baskul-scroll".into()).spawn(move || {
            // SAFETY: source d'événements propre à ce thread.
            let source = unsafe { CGEventSourceCreate(SOURCE_HID_STATE) };
            let mut flags = 0u64;
            let mut lines = (0.0, 0.0);
            loop {
                if scroller.is_idle() {
                    // Rien à animer : attente du prochain cran.
                    let Ok((at, dir, weight, f)) = rx.recv() else { return };
                    // Les distances suivent un peu la taille de l'écran sous
                    // le pointeur, comme dans Mac Mouse Fix.
                    if let Some(r) = current_cursor().and_then(|p| displays().display_at(p).copied()) {
                        scroller.set_screen(f64::from(r.width()), f64::from(r.height()));
                    }
                    scroller.tick(at, dir, weight);
                    flags = f;
                }
                while let Ok((at, dir, weight, f)) = rx.try_recv() {
                    scroller.tick(at, dir, weight);
                    flags = f;
                }
                for ev in scroller.frame(Instant::now()) {
                    post_smooth_event(source, ev, flags, &mut lines);
                }
                std::thread::sleep(smooth::FRAME);
            }
        });
        if let Err(e) = spawned {
            log::error!("défilement fluide indisponible : {e}");
        }
        SmoothScroller { tx }
    }

    /// Transmet les crans reçus (positifs vers le haut et vers la droite).
    fn wheel(&self, nx: f64, ny: f64, flags: u64) {
        let now = Instant::now();
        for (n, pos, neg) in [(ny, smooth::Dir::Up, smooth::Dir::Down), (nx, smooth::Dir::Right, smooth::Dir::Left)] {
            if n != 0.0 {
                let _ = self.tx.send((now, if n > 0.0 { pos } else { neg }, n.abs(), flags));
            }
        }
    }
}

// Champs non documentés des événements de défilement et de geste (voir Mac
// Mouse Fix, GestureScrollSimulator.m).
const F_EVENT_TYPE: u32 = 55;
const F_SCROLL_PHASE: u32 = 99;
const F_SCROLL_MOMENTUM_PHASE: u32 = 123;
const F_SCROLL_INVERTED: u32 = 137;
const F_GESTURE_HID_TYPE: u32 = 110;
const F_GESTURE_SCROLL_X: u32 = 116;
const F_GESTURE_SCROLL_Y: u32 = 119;
const F_GESTURE_PHASE: u32 = 132;
const HID_EVENT_SCROLL: i64 = 6;

/// Envoie un événement du défilement fluide. `lines` : fractions de ligne
/// pas encore envoyées (une ligne = 10 pixels).
fn post_smooth_event(source: CGEventSourceRef, ev: smooth::Event, flags: u64, lines: &mut (f64, f64)) {
    use smooth::{Event, GesturePhase, MomentumPhase};
    let (v, h, phase, momentum) = match ev {
        Event::Scroll { v, h } => {
            // macOS : valeur positive = vers le haut (axe 1) / vers la gauche (axe 2).
            // SAFETY: événement créé, envoyé puis libéré.
            unsafe {
                let e = CGEventCreateScrollWheelEvent2(source, SCROLL_UNIT_PIXEL, 2, v, -h, 0);
                if !e.is_null() {
                    CGEventSetFlags(e, flags);
                    CGEventPost(TAP_HID, e);
                    CFRelease(e);
                }
            }
            return;
        }
        Event::Gesture { v, h, phase } => (v, -h, phase as i64, 0),
        Event::Momentum { v, h, phase } => (v, -h, 0, phase as i64),
    };
    if phase == GesturePhase::Began as i64 || momentum == MomentumPhase::Begin as i64 {
        *lines = (0.0, 0.0);
    }
    // Lignes : fractions cumulées d'un événement à l'autre.
    lines.0 += v as f64 / 10.0;
    lines.1 += h as f64 / 10.0;
    let (l1, l2) = (lines.0.trunc(), lines.1.trunc());
    lines.0 -= l1;
    lines.1 -= l2;
    let fixed = |l: f64| (l * 65536.0) as i64;
    // SAFETY: événements créés, remplis, envoyés puis libérés.
    unsafe {
        let e = CGEventCreate(null_mut());
        if e.is_null() {
            return;
        }
        CGEventSetIntegerValueField(e, F_EVENT_TYPE, EV_SCROLL as i64);
        CGEventSetIntegerValueField(e, F_SCROLL_IS_CONTINUOUS, 1);
        CGEventSetIntegerValueField(e, F_SCROLL_INVERTED, 1);
        CGEventSetIntegerValueField(e, F_SCROLL_DELTA_1, l1 as i64);
        CGEventSetIntegerValueField(e, F_SCROLL_POINT_1, v as i64);
        CGEventSetIntegerValueField(e, F_SCROLL_FIXED_1, fixed(l1));
        CGEventSetIntegerValueField(e, F_SCROLL_DELTA_2, l2 as i64);
        CGEventSetIntegerValueField(e, F_SCROLL_POINT_2, h as i64);
        CGEventSetIntegerValueField(e, F_SCROLL_FIXED_2, fixed(l2));
        CGEventSetIntegerValueField(e, F_SCROLL_PHASE, phase);
        CGEventSetIntegerValueField(e, F_SCROLL_MOMENTUM_PHASE, momentum);
        CGEventSetFlags(e, flags);
        CGEventPost(TAP_SESSION, e);
        CFRelease(e);
        // Le geste lui-même (événement « gesture » de sous-type défilement).
        if phase != 0 {
            let g = CGEventCreate(null_mut());
            if g.is_null() {
                return;
            }
            CGEventSetIntegerValueField(g, F_EVENT_TYPE, EV_GESTURE as i64);
            CGEventSetIntegerValueField(g, F_GESTURE_HID_TYPE, HID_EVENT_SCROLL);
            let gesture = |d: i32| if d == 0 { -0.0 } else { d as f64 * 1.67 };
            CGEventSetDoubleValueField(g, F_GESTURE_SCROLL_X, gesture(h));
            CGEventSetDoubleValueField(g, F_GESTURE_SCROLL_Y, gesture(v));
            CGEventSetIntegerValueField(g, F_GESTURE_PHASE, phase);
            CGEventPost(TAP_SESSION, g);
            CFRelease(g);
        }
    }
}

fn open_hid_system() -> u32 {
    // SAFETY: appels IOKit ; le service est libéré, la connexion conservée.
    unsafe {
        let matching = IOServiceMatching(c"IOHIDSystem".as_ptr());
        if matching.is_null() {
            return 0;
        }
        let svc = IOServiceGetMatchingService(0, matching);
        if svc == 0 {
            return 0;
        }
        let mut conn = 0u32;
        let kr = IOServiceOpen(svc, mach_task_self_, 1, &mut conn);
        IOObjectRelease(svc);
        if kr == 0 { conn } else { 0 }
    }
}

impl MacEmulator {
    fn new(settings: &Settings) -> MacEmulator {
        // SAFETY: création d'une source d'événements.
        let source = unsafe {
            let s = CGEventSourceCreate(SOURCE_HID_STATE);
            if !s.is_null() {
                // Sans cela, macOS ignore la souris/le clavier du Mac pendant
                // 0,25 s après chaque événement injecté.
                CGEventSourceSetLocalEventsSuppressionInterval(s, 0.0);
            }
            s
        };
        let double_click = Duration::from_secs_f64(NSEvent::doubleClickInterval().clamp(0.1, 2.0));
        let hid_conn = open_hid_system();
        if hid_conn == 0 {
            log::warn!("accès IOHIDSystem impossible : Verr. Maj sera simulée");
        }
        let mut e = MacEmulator {
            source,
            pressed: HashSet::new(),
            mods_down: HashSet::new(),
            caps_on: false,
            buttons: [false; 5],
            pos: current_cursor().unwrap_or_default(),
            click_button: None,
            click_time: Instant::now(),
            click_pos: Point::default(),
            click_count: 0,
            double_click,
            scroll: settings.scroll.clone(),
            acc_x: 0.0,
            acc_y: 0.0,
            side: settings.buttons.side_buttons,
            smooth: None,
            iso: settings.iso_swap,
            iso_active: false,
            hid_conn,
            last_activity: None,
        };
        e.detect_keyboard(true);
        e.refresh_caps();
        e
    }

    fn detect_keyboard(&mut self, log_always: bool) {
        let (iso, kbd_type) = keyboard_is_iso();
        let active = match self.iso {
            IsoSwap::On => true,
            IsoSwap::Off => false,
            IsoSwap::Auto => iso,
        };
        if active != self.iso_active || log_always {
            log::info!(
                "clavier du Mac : type {kbd_type} ({}) ; échange des touches ISO « ² » / « < » : {}",
                if iso { "ISO" } else { "ANSI/JIS" },
                if active { "oui" } else { "non" }
            );
        }
        self.iso_active = active;
    }

    fn refresh_caps(&mut self) {
        if self.hid_conn != 0 {
            if let Some(st) = get_caps_lock(self.hid_conn) {
                self.caps_on = st;
                return;
            }
        }
        // SAFETY: lecture d'état.
        self.caps_on = unsafe { CGEventSourceFlagsState(SOURCE_HID_STATE) } & FL_ALPHA_SHIFT != 0;
    }

    fn flags(&self) -> u64 {
        let mut f = FL_NON_COALESCED;
        for kc in &self.mods_down {
            f |= modifier_flags(*kc);
        }
        if self.caps_on {
            f |= FL_ALPHA_SHIFT;
        }
        f
    }

    fn activity(&mut self) {
        let due = self.last_activity.is_none_or(|t| t.elapsed() > Duration::from_secs(10));
        if due {
            self.last_activity = Some(Instant::now());
            // Empêche la mise en veille de l'écran du Mac pendant qu'on le pilote.
            let name = cf_string(c"Baskul");
            let mut id = 0u32;
            // SAFETY: chaîne CF valide, libérée ensuite.
            unsafe {
                IOPMAssertionDeclareUserActivity(name, 0, &mut id);
                CFRelease(name);
            }
        }
    }

    fn post(&self, ev: CGEventRef) {
        if ev.is_null() {
            return;
        }
        // SAFETY: événement valide, libéré après envoi.
        unsafe {
            CGEventPost(TAP_HID, ev);
            CFRelease(ev);
        }
    }

    fn post_key(&self, kc: u16, down: bool, repeat: bool) {
        // SAFETY: création d'un événement clavier.
        unsafe {
            let ev = CGEventCreateKeyboardEvent(self.source, kc, down);
            if ev.is_null() {
                return;
            }
            CGEventSetFlags(ev, self.flags() | key_extra_flags(kc));
            if repeat && down {
                CGEventSetIntegerValueField(ev, F_AUTOREPEAT, 1);
            }
            self.post(ev);
        }
    }

    fn post_flags_changed(&self, kc: u16) {
        // SAFETY: création d'un événement clavier transformé en FlagsChanged.
        unsafe {
            let ev = CGEventCreateKeyboardEvent(self.source, kc, true);
            if ev.is_null() {
                return;
            }
            CGEventSetType(ev, EV_FLAGS_CHANGED);
            CGEventSetFlags(ev, self.flags());
            self.post(ev);
        }
    }

    fn toggle_caps(&mut self) {
        if self.hid_conn != 0 {
            if let Some(st) = get_caps_lock(self.hid_conn).filter(|st| set_caps_lock(self.hid_conn, !st)) {
                self.caps_on = !st;
                log::debug!("Verr. Maj {}", if self.caps_on { "activé" } else { "désactivé" });
                return;
            }
        }
        self.caps_on = !self.caps_on;
        self.post_flags_changed(mac::CAPS_LOCK);
    }

    fn post_mouse(&self, ty: u32, button: u32, p: Point, click: i64, number: Option<i64>, dx: i32, dy: i32) {
        // SAFETY: création d'un événement souris.
        unsafe {
            let ev = CGEventCreateMouseEvent(self.source, ty, CGPoint { x: p.x as f64, y: p.y as f64 }, button);
            if ev.is_null() {
                return;
            }
            if click > 0 {
                CGEventSetIntegerValueField(ev, F_CLICK_STATE, click);
            }
            if let Some(n) = number {
                CGEventSetIntegerValueField(ev, F_BUTTON_NUMBER, n);
            }
            if dx != 0 || dy != 0 {
                CGEventSetIntegerValueField(ev, F_DELTA_X, dx as i64);
                CGEventSetIntegerValueField(ev, F_DELTA_Y, dy as i64);
            }
            CGEventSetFlags(ev, self.flags());
            self.post(ev);
        }
    }
}

impl MacEmulator {
    /// Bouton physique tel quel.
    fn raw_button(&mut self, button: MouseButton, down: bool) {
        if down {
            let near = (self.pos.x - self.click_pos.x).abs() <= 4 && (self.pos.y - self.click_pos.y).abs() <= 4;
            if self.click_button == Some(button) && near && self.click_time.elapsed() <= self.double_click {
                self.click_count += 1;
            } else {
                self.click_count = 1;
            }
            self.click_button = Some(button);
            self.click_time = Instant::now();
            self.click_pos = self.pos;
        }
        self.buttons[button.index()] = down;
        let click = self.click_count.max(1);
        let (ty, cg_button, number) = match (button, down) {
            (MouseButton::Left, true) => (EV_LEFT_DOWN, 0, None),
            (MouseButton::Left, false) => (EV_LEFT_UP, 0, None),
            (MouseButton::Right, true) => (EV_RIGHT_DOWN, 1, None),
            (MouseButton::Right, false) => (EV_RIGHT_UP, 1, None),
            (b, true) => (EV_OTHER_DOWN, b.index() as u32, Some(b.index() as i64)),
            (b, false) => (EV_OTHER_UP, b.index() as u32, Some(b.index() as i64)),
        };
        self.post_mouse(ty, cg_button, self.pos, click, number, 0, 0);
    }
}

impl Emulator for MacEmulator {
    fn layout(&mut self) -> Layout {
        displays()
    }

    fn move_to(&mut self, p: Point, dx: i32, dy: i32) {
        self.activity();
        self.pos = p;
        let (ty, button, number) = if self.buttons[MouseButton::Left.index()] {
            (EV_LEFT_DRAGGED, 0, None)
        } else if self.buttons[MouseButton::Right.index()] {
            (EV_RIGHT_DRAGGED, 1, None)
        } else if let Some(b) =
            [MouseButton::Middle, MouseButton::Back, MouseButton::Forward].into_iter().find(|b| self.buttons[b.index()])
        {
            let n = b.index() as i64;
            (EV_OTHER_DRAGGED, n as u32, Some(n))
        } else {
            (EV_MOUSE_MOVED, 0, None)
        };
        self.post_mouse(ty, button, p, 0, number, dx, dy);
    }

    fn button(&mut self, button: MouseButton, down: bool) {
        self.activity();
        if matches!(button, MouseButton::Back | MouseButton::Forward) && self.side == SideButtonsMode::Swipe {
            if down {
                post_swipe(if button == MouseButton::Back { 4 } else { 8 });
            }
            return;
        }
        self.raw_button(button, down);
    }

    fn wheel(&mut self, dx: i32, dy: i32) {
        self.activity();
        // Crans (1 cran Windows = 120).
        let mut ny = dy as f64 / 120.0;
        let mut nx = dx as f64 / 120.0;
        if self.scroll.invert_vertical {
            ny = -ny;
        }
        if self.scroll.invert_horizontal {
            nx = -nx;
        }
        if self.scroll.mode == ScrollMode::Smooth {
            let flags = self.flags();
            let scroll = &self.scroll;
            self.smooth.get_or_insert_with(|| SmoothScroller::start(scroll)).wheel(nx, ny, flags);
            return;
        }
        let (unit, per_notch) = match self.scroll.mode {
            ScrollMode::Lines => (SCROLL_UNIT_LINE, self.scroll.lines_per_notch),
            ScrollMode::Pixels | ScrollMode::Smooth => (SCROLL_UNIT_PIXEL, self.scroll.pixels_per_notch),
        };
        // Accumulation des fractions (molettes haute résolution, réglages
        // non entiers).
        self.acc_y += ny * per_notch;
        self.acc_x += nx * per_notch;
        let vy = self.acc_y.trunc();
        let vx = self.acc_x.trunc();
        self.acc_y -= vy;
        self.acc_x -= vx;
        if vy == 0.0 && vx == 0.0 {
            return;
        }
        // macOS : valeur positive = vers le haut (axe 1) / vers la gauche (axe 2).
        // SAFETY: création d'un événement de défilement.
        unsafe {
            let ev = CGEventCreateScrollWheelEvent2(self.source, unit, 2, vy as i32, -(vx as i32), 0);
            if ev.is_null() {
                return;
            }
            CGEventSetFlags(ev, self.flags());
            self.post(ev);
        }
    }

    fn key(&mut self, code: u32, down: bool, repeat: bool) {
        self.activity();
        if let Some(t) = media_type(code) {
            post_media(t, down);
            return;
        }
        if code == HID_CAPS_LOCK {
            if down && !repeat {
                self.toggle_caps();
            }
            return;
        }
        let Some(kc) = keymap::mac_from_hid(code) else {
            log::debug!("touche sans équivalent Mac : {}", keymap::name_from_hid(code));
            return;
        };
        let kc = if self.iso_active { keymap::iso_swap_mac(kc) } else { kc };
        if modifier_flags(kc) != 0 {
            let changed = if down { self.mods_down.insert(kc) } else { self.mods_down.remove(&kc) };
            if changed {
                self.post_flags_changed(kc);
            }
            return;
        }
        if down {
            if !repeat {
                self.refresh_caps();
            }
            self.pressed.insert(kc);
        } else {
            self.pressed.remove(&kc);
        }
        self.post_key(kc, down, repeat);
    }

    fn release_all(&mut self) {
        for kc in std::mem::take(&mut self.pressed) {
            self.post_key(kc, false, false);
        }
        let mods: Vec<u16> = self.mods_down.drain().collect();
        for kc in mods {
            self.post_flags_changed(kc);
        }
        for b in MouseButton::ALL {
            if self.buttons[b.index()] {
                self.raw_button(b, false);
            }
        }
    }

    fn entered(&mut self) {
        self.last_activity = None;
        self.activity();
        self.detect_keyboard(false);
        self.refresh_caps();
        self.acc_x = 0.0;
        self.acc_y = 0.0;
    }

    fn arrived(&mut self) {
        request_bubble();
    }

    fn cursor(&mut self) -> Option<Point> {
        current_cursor()
    }

    fn keep_active(&mut self) {
        keep_active();
    }
}

// ---------------------------------------------------------------------------
// Activité partagée
// ---------------------------------------------------------------------------

/// L'utilisateur est actif sur l'autre machine : l'écran du Mac reste allumé
/// (ni veille ni économiseur), et les apps qui mesurent le temps écoulé
/// depuis le dernier événement (Teams, Discord, Slack…) le voient présent.
fn keep_active() {
    let name = cf_string(c"Baskul");
    let mut id = 0u32;
    // SAFETY: chaîne CF valide, libérée ensuite ; événement créé, envoyé
    // puis libéré.
    unsafe {
        IOPMAssertionDeclareUserActivity(name, 0, &mut id);
        CFRelease(name);
        // Événement neutre : déplacement nul du curseur (pas pendant un clic
        // fait sur le Mac lui-même).
        if (0..3).any(|b| CGEventSourceButtonState(SOURCE_HID_STATE, b)) {
            return;
        }
        let here = CGEventCreate(null_mut());
        if here.is_null() {
            return;
        }
        let pos = CGEventGetLocation(here);
        CFRelease(here);
        let source = CGEventSourceCreate(SOURCE_HID_STATE);
        let ev = CGEventCreateMouseEvent(source, EV_MOUSE_MOVED, pos, 0);
        if !ev.is_null() {
            CGEventPost(TAP_HID, ev);
            CFRelease(ev);
        }
        if !source.is_null() {
            CFRelease(source);
        }
    }
}

/// Temps écoulé depuis la dernière entrée du Mac (clavier, souris, trackpad).
fn idle_time() -> Option<Duration> {
    const ANY_INPUT: u32 = !0;
    // SAFETY: fonction sans précondition.
    let s = unsafe { CGEventSourceSecondsSinceLastEventType(SOURCE_HID_STATE, ANY_INPUT) };
    (s.is_finite() && s >= 0.0).then(|| Duration::from_secs_f64(s))
}

// ---------------------------------------------------------------------------
// Presse-papiers
// ---------------------------------------------------------------------------

fn clip_sequence() -> Option<u64> {
    autoreleasepool(|_| Some(NSPasteboard::generalPasteboard().changeCount() as u64))
}

fn clip_sensitive() -> bool {
    // Convention nspasteboard.org suivie par les gestionnaires de mots de passe.
    const MARKERS: [&str; 3] =
        ["org.nspasteboard.ConcealedType", "org.nspasteboard.TransientType", "com.agilebits.onepassword"];
    autoreleasepool(|_| {
        let Some(types) = NSPasteboard::generalPasteboard().types() else { return false };
        types.iter().any(|t| {
            let s = t.to_string();
            MARKERS.iter().any(|m| *m == s)
        })
    })
}

/// Images du presse-papiers, dans leur format d'origine : PNG, JPEG, GIF,
/// HEIC, WebP ou BMP tels quels (plus un PNG pour les formats que l'autre
/// machine ne sait pas forcément afficher). Seul le TIFF, format de transport
/// d'AppKit quand l'application n'en fournit pas d'autre, devient un PNG.
fn clip_read_images() -> Vec<ClipImage> {
    const ORIGINALS: [ImageFormat; 6] =
        [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Gif, ImageFormat::Heic, ImageFormat::Webp, ImageFormat::Bmp];
    autoreleasepool(|_| {
        let pb = NSPasteboard::generalPasteboard();
        for format in ORIGINALS {
            let Some(data) = pb.dataForType(&NSString::from_str(format.uti())) else { continue };
            let mut out = vec![ClipImage { format, data: data.to_vec() }];
            if matches!(format, ImageFormat::Heic | ImageFormat::Webp | ImageFormat::Bmp) {
                if let Some(png) = png_from_image_data(&data) {
                    out.push(ClipImage { format: ImageFormat::Png, data: png });
                }
            }
            return out;
        }
        // SAFETY: constante d'AppKit.
        let tiff = unsafe { NSPasteboardTypeTIFF };
        let png = pb.dataForType(tiff).and_then(|d| png_from_image_data(&d));
        png.map(|data| vec![ClipImage { format: ImageFormat::Png, data }]).unwrap_or_default()
    })
}

/// Écrit le contenu reçu quand il comporte des images ou des fichiers (le
/// texte seul passe par arboard) : les fichiers, puis chaque image dans son
/// format d'origine, avec un PNG pour les formats peu lus par les
/// applications Mac.
fn clip_write(h: &ClipHeader, roots: &[PathBuf]) -> Option<bool> {
    if h.images.is_empty() && roots.is_empty() {
        return None;
    }
    Some(autoreleasepool(|_| {
        // SAFETY: constantes d'AppKit.
        let (file_url, string, html) =
            unsafe { (NSPasteboardTypeFileURL, NSPasteboardTypeString, NSPasteboardTypeHTML) };
        let mut items: Vec<Retained<NSPasteboardItem>> = Vec::new();
        for root in roots {
            let Some(path) = root.to_str() else { continue };
            let Some(url) = NSURL::fileURLWithPath(&NSString::from_str(path)).absoluteString() else { continue };
            let item = NSPasteboardItem::new();
            item.setString_forType(&url, file_url);
            items.push(item);
        }
        if items.is_empty() {
            items.push(NSPasteboardItem::new());
        }
        let first = &items[0];
        for img in &h.images {
            first.setData_forType(&NSData::with_bytes(&img.data), &NSString::from_str(img.format.uti()));
        }
        let readable =
            |f: ImageFormat| matches!(f, ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::Tiff);
        if let Some(img) = h.images.first().filter(|_| !h.images.iter().any(|i| readable(i.format))) {
            if let Some(png) = png_from_image_data(&NSData::with_bytes(&img.data)) {
                first.setData_forType(&NSData::with_bytes(&png), &NSString::from_str(ImageFormat::Png.uti()));
            }
        }
        if let Some(t) = &h.text {
            first.setString_forType(&NSString::from_str(t), string);
        }
        if let Some(x) = &h.html {
            first.setString_forType(&NSString::from_str(x), html);
        }
        let objects: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> =
            items.into_iter().map(ProtocolObject::from_retained).collect();
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        let ok = pb.writeObjects(&NSArray::from_retained_slice(&objects));
        if !ok {
            log::warn!("écriture du presse-papiers refusée par macOS");
        }
        ok
    }))
}

fn png_from_image_data(data: &NSData) -> Option<Vec<u8>> {
    let rep = NSBitmapImageRep::imageRepWithData(data)?;
    // SAFETY: dictionnaire de propriétés vide, du type attendu.
    let png = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new()) }?;
    Some(png.to_vec())
}

/// Formats présents dans le presse-papiers, pour le journal (sans les types
/// hérités ou dynamiques, sans intérêt).
fn clip_formats() -> Vec<String> {
    autoreleasepool(|_| {
        let Some(types) = NSPasteboard::generalPasteboard().types() else { return vec![] };
        types
            .iter()
            .map(|t| t.to_string())
            .filter(|t| !t.starts_with("dyn.") && !t.starts_with("CorePasteboardFlavorType") && !t.starts_with("NeXT"))
            .take(10)
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Machine principale : capture (sens inverse, expérimental)
// ---------------------------------------------------------------------------

struct MacHost {
    layout: Layout,
    layout_at: Instant,
    link: LinkHandle,
    hidden: bool,
    config_path: PathBuf,
}

impl Host for MacHost {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn cursor(&self) -> Point {
        current_cursor().unwrap_or_else(|| self.layout.bounds().center())
    }

    fn hide_cursor(&mut self, _at: Point) {
        // SAFETY: appels CoreGraphics depuis le thread principal.
        unsafe {
            if let (Some(conn), Some(set_prop)) = (syms().cgs_default, syms().cgs_set_prop) {
                let key = cf_string(c"SetsCursorInBackground");
                let cid = conn();
                set_prop(cid, cid, key, kCFBooleanTrue);
                CFRelease(key);
            }
            CGAssociateMouseAndMouseCursorPosition(false);
            if !self.hidden {
                CGDisplayHideCursor(CGMainDisplayID());
                self.hidden = true;
            }
        }
    }

    fn show_cursor(&mut self, pos: Point) {
        // SAFETY: appels CoreGraphics depuis le thread principal.
        unsafe {
            CGWarpMouseCursorPosition(CGPoint { x: pos.x as f64, y: pos.y as f64 });
            CGAssociateMouseAndMouseCursorPosition(true);
            if self.hidden {
                CGDisplayShowCursor(CGMainDisplayID());
                self.hidden = false;
            }
        }
    }

    fn send(&mut self, to: PeerId, msg: Message) {
        self.link.send_input(to, msg);
    }

    fn keep_active(&mut self) {
        keep_active();
    }

    fn save_layout(&mut self, machines: Vec<MachineCfg>) {
        app::save_layout(&self.config_path, machines);
    }

    fn arrived(&mut self, _pos: Point) {
        request_bubble();
    }
}

struct MacPrimary {
    core: PrimaryCore,
    host: MacHost,
    peer_rx: Receiver<PeerEvent>,
    last_loc: Point,
    tap: CFMachPortRef,
    iso_active: bool,
    /// Fractions de crans de défilement à transmettre.
    acc_x: f64,
    acc_y: f64,
}

thread_local! {
    static MAC_PRIMARY: RefCell<Option<MacPrimary>> = const { RefCell::new(None) };
    static MAC_UI: RefCell<Option<MacUi>> = const { RefCell::new(None) };
}

static PEER_SOURCE: AtomicUsize = AtomicUsize::new(0);
static MAIN_LOOP: AtomicUsize = AtomicUsize::new(0);

fn wake_main() {
    let src = PEER_SOURCE.load(Ordering::Acquire);
    let rl = MAIN_LOOP.load(Ordering::Acquire);
    if src != 0 && rl != 0 {
        // SAFETY: source et boucle valides pour toute la durée du programme ;
        // ces deux fonctions sont utilisables depuis n'importe quel thread.
        unsafe {
            CFRunLoopSourceSignal(src as CFRunLoopSourceRef);
            CFRunLoopWakeUp(rl as CFRunLoopRef);
        }
    }
}

unsafe extern "C" fn peer_perform(_info: *mut c_void) {
    drain_peer_events();
}

fn drain_peer_events() {
    MAC_PRIMARY.with(|p| {
        if let Ok(mut g) = p.try_borrow_mut() {
            if let Some(s) = g.as_mut() {
                while let Ok(ev) = s.peer_rx.try_recv() {
                    s.core.peer_event(&mut s.host, ev);
                }
            }
        }
    });
}

fn button_from_number(n: i64) -> Option<MouseButton> {
    match n {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Right),
        2 => Some(MouseButton::Middle),
        3 => Some(MouseButton::Back),
        4 => Some(MouseButton::Forward),
        _ => None,
    }
}

/// Bit « par touche » d'un modificateur, pour savoir s'il est enfoncé.
fn modifier_device_bit(kc: u16) -> u64 {
    match kc {
        mac::SHIFT => DEV_LSHIFT,
        mac::RIGHT_SHIFT => DEV_RSHIFT,
        mac::CONTROL => DEV_LCTL,
        mac::RIGHT_CONTROL => DEV_RCTL,
        mac::OPTION => DEV_LALT,
        mac::RIGHT_OPTION => DEV_RALT,
        mac::COMMAND => DEV_LCMD,
        mac::RIGHT_COMMAND => DEV_RCMD,
        mac::FUNCTION => FL_SECONDARY_FN,
        _ => 0,
    }
}

impl MacPrimary {
    fn on_event(&mut self, ty: u32, ev: CGEventRef) -> Decision {
        if self.host.layout_at.elapsed() > Duration::from_secs(2) {
            self.host.layout = displays();
            self.host.layout_at = Instant::now();
        }
        let remote = self.core.is_remote();
        let other = if remote { Decision::Swallow } else { Decision::Pass };
        // SAFETY: lecture de champs d'un événement valide fourni par le tap.
        unsafe {
            match ty {
                EV_MOUSE_MOVED | EV_LEFT_DRAGGED | EV_RIGHT_DRAGGED | EV_OTHER_DRAGGED => {
                    let dx = CGEventGetIntegerValueField(ev, F_DELTA_X) as i32;
                    let dy = CGEventGetIntegerValueField(ev, F_DELTA_Y) as i32;
                    if remote {
                        self.core.remote_motion(&mut self.host, dx, dy);
                        Decision::Swallow
                    } else {
                        let loc = CGEventGetLocation(ev);
                        let loc = Point::new(loc.x.round() as i32, loc.y.round() as i32);
                        let cur = self.last_loc;
                        let next = Point::new(cur.x + dx, cur.y + dy);
                        self.last_loc = loc;
                        self.core.local_motion(&mut self.host, cur, next)
                    }
                }
                EV_LEFT_DOWN | EV_LEFT_UP => self.core.button(&mut self.host, MouseButton::Left, ty == EV_LEFT_DOWN),
                EV_RIGHT_DOWN | EV_RIGHT_UP => {
                    self.core.button(&mut self.host, MouseButton::Right, ty == EV_RIGHT_DOWN)
                }
                EV_OTHER_DOWN | EV_OTHER_UP => {
                    match button_from_number(CGEventGetIntegerValueField(ev, F_BUTTON_NUMBER)) {
                        Some(b) => self.core.button(&mut self.host, b, ty == EV_OTHER_DOWN),
                        None => other,
                    }
                }
                EV_SCROLL => {
                    if !remote {
                        return Decision::Pass;
                    }
                    // Conversion en unités Windows (120 = un cran ≈ 3 lignes).
                    let continuous = CGEventGetIntegerValueField(ev, F_SCROLL_IS_CONTINUOUS) != 0;
                    let (vy, vx) = if continuous {
                        (
                            CGEventGetDoubleValueField(ev, F_SCROLL_POINT_1) * 3.0,
                            CGEventGetDoubleValueField(ev, F_SCROLL_POINT_2) * 3.0,
                        )
                    } else {
                        let fy = CGEventGetDoubleValueField(ev, F_SCROLL_FIXED_1);
                        let fx = CGEventGetDoubleValueField(ev, F_SCROLL_FIXED_2);
                        let (fy, fx) = if fy == 0.0 && fx == 0.0 {
                            (
                                CGEventGetIntegerValueField(ev, F_SCROLL_DELTA_1) as f64,
                                CGEventGetIntegerValueField(ev, F_SCROLL_DELTA_2) as f64,
                            )
                        } else {
                            (fy, fx)
                        };
                        (fy * 40.0, fx * 40.0)
                    };
                    self.acc_y += vy;
                    self.acc_x -= vx;
                    let (iy, ix) = (self.acc_y.trunc(), self.acc_x.trunc());
                    self.acc_y -= iy;
                    self.acc_x -= ix;
                    if iy != 0.0 || ix != 0.0 {
                        self.core.wheel(&mut self.host, ix as i32, iy as i32);
                    }
                    Decision::Swallow
                }
                EV_KEY_DOWN | EV_KEY_UP => {
                    let kc = CGEventGetIntegerValueField(ev, F_KEYCODE) as u16;
                    let kc = if self.iso_active { keymap::iso_swap_mac(kc) } else { kc };
                    match keymap::hid_from_mac(kc) {
                        Some(hid) => self.core.key(&mut self.host, hid, ty == EV_KEY_DOWN),
                        None => other,
                    }
                }
                EV_FLAGS_CHANGED => {
                    let kc = CGEventGetIntegerValueField(ev, F_KEYCODE) as u16;
                    let flags = CGEventGetFlags(ev);
                    if kc == mac::CAPS_LOCK {
                        // Verr. Maj : un seul événement par bascule → appui + relâchement.
                        let d = self.core.key(&mut self.host, HID_CAPS_LOCK, true);
                        self.core.key(&mut self.host, HID_CAPS_LOCK, false);
                        return d;
                    }
                    let bit = modifier_device_bit(kc);
                    let Some(hid) = keymap::hid_from_mac(kc) else { return other };
                    if bit == 0 {
                        return other;
                    }
                    self.core.key(&mut self.host, hid, flags & bit != 0)
                }
                EV_SYSTEM_DEFINED => {
                    if !remote {
                        return Decision::Pass;
                    }
                    let mut decision = Decision::Swallow;
                    autoreleasepool(|_| {
                        let nse: Option<Retained<NSEvent>> = msg_send![objc2::class!(NSEvent), eventWithCGEvent: ev];
                        let Some(nse) = nse else { return };
                        if nse.subtype().0 != 8 {
                            decision = Decision::Pass;
                            return;
                        }
                        let data1 = nse.data1();
                        let key_type = (data1 >> 16) & 0xffff;
                        let down = (data1 >> 8) & 0xff == 0xa;
                        let hid = match key_type {
                            0 => HID_VOLUME_UP,
                            1 => HID_VOLUME_DOWN,
                            7 => HID_VOLUME_MUTE,
                            14 => HID_EJECT,
                            16 => HID_MEDIA_PLAY_PAUSE,
                            17 => HID_MEDIA_NEXT,
                            18 => HID_MEDIA_PREVIOUS,
                            _ => {
                                decision = Decision::Pass;
                                return;
                            }
                        };
                        decision = self.core.key(&mut self.host, hid, down);
                    });
                    decision
                }
                _ => other,
            }
        }
    }
}

unsafe extern "C" fn tap_callback(_proxy: *mut c_void, ty: u32, ev: CGEventRef, _user: *mut c_void) -> CGEventRef {
    if ty == EV_TAP_DISABLED_TIMEOUT || ty == EV_TAP_DISABLED_USER {
        log::warn!("le système a désactivé la capture ; réactivation");
        MAC_PRIMARY.with(|p| {
            if let Ok(g) = p.try_borrow() {
                if let Some(s) = g.as_ref() {
                    // SAFETY: tap créé par nous.
                    unsafe { CGEventTapEnable(s.tap, true) };
                }
            }
        });
        return ev;
    }
    let d = MAC_PRIMARY.with(|p| match p.try_borrow_mut() {
        Ok(mut g) => match g.as_mut() {
            Some(s) => s.on_event(ty, ev),
            None => Decision::Pass,
        },
        Err(_) => Decision::Pass,
    });
    if d == Decision::Swallow { null_mut() } else { ev }
}

fn create_tap() -> CFMachPortRef {
    let types = [
        EV_LEFT_DOWN,
        EV_LEFT_UP,
        EV_RIGHT_DOWN,
        EV_RIGHT_UP,
        EV_MOUSE_MOVED,
        EV_LEFT_DRAGGED,
        EV_RIGHT_DRAGGED,
        EV_KEY_DOWN,
        EV_KEY_UP,
        EV_FLAGS_CHANGED,
        EV_SYSTEM_DEFINED,
        EV_SCROLL,
        EV_OTHER_DOWN,
        EV_OTHER_UP,
        EV_OTHER_DRAGGED,
    ];
    let mask = types.iter().fold(0u64, |m, t| m | (1u64 << t));
    // SAFETY: création d'un tap ajouté à la boucle principale.
    unsafe {
        let tap = CGEventTapCreate(TAP_SESSION, 0, 0, mask, tap_callback, null_mut());
        if tap.is_null() {
            return tap;
        }
        let src = CFMachPortCreateRunLoopSource(null(), tap, 0);
        CFRunLoopAddSource(CFRunLoopGetMain(), src, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        tap
    }
}

fn primary_permissions_ok() -> bool {
    // SAFETY: fonctions sans précondition.
    unsafe { AXIsProcessTrusted() && CGPreflightListenEventAccess() }
}

// ---------------------------------------------------------------------------
// Barre des menus
// ---------------------------------------------------------------------------

struct MacUi {
    item: Retained<NSStatusItem>,
    status_line: Retained<NSMenuItem>,
    access_item: Retained<NSMenuItem>,
    /// « Redémarrer pour mettre à jour », visible quand une version est prête.
    update_item: Retained<NSMenuItem>,
    update_ready: Option<String>,
    return_item: Option<Retained<NSMenuItem>>,
    lock_item: Option<Retained<NSMenuItem>>,
    /// Entrées « Aller sur… », une par pilotée connectée.
    goto_items: Vec<Retained<NSMenuItem>>,
    goto_names: Vec<String>,
    login_item: Retained<NSMenuItem>,
    _controller: Retained<Controller>,
    last: Option<Snapshot>,
    menu: Retained<NSMenu>,
    /// Création de l'icône (elle met un instant à apparaître).
    created_at: Instant,
    /// Depuis quand l'icône n'est plus à l'écran.
    hidden_since: Option<Instant>,
    recreated_at: Option<Instant>,
    tray_icon: TrayIcon,
    role: Role,
    config_path: PathBuf,
    args: Args,
    permissions_ok: bool,
}

define_class!(
    // SAFETY: NSObject n'a pas d'exigence particulière pour les sous-classes ;
    // Controller n'implémente pas Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BaskulController"]
    struct Controller;

    unsafe impl NSObjectProtocol for Controller {}

    unsafe impl NSApplicationDelegate for Controller {
        /// Double-clic sur Baskul.app alors qu'il tourne déjà : réglages.
        #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
        fn should_handle_reopen(&self, _app: &NSApplication, _visible: bool) -> bool {
            if let Some(p) = ui_value(|u| u.config_path.clone()) {
                super::open_settings(&p, false);
            }
            false
        }
    }

    impl Controller {
        #[unsafe(method(tick:))]
        fn tick(&self, _sender: Option<&AnyObject>) {
            ui_tick(self.mtm());
        }

        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _sender: Option<&AnyObject>) {
            if let Some(p) = ui_value(|u| u.config_path.clone()) {
                super::open_settings(&p, false);
            }
        }

        #[unsafe(method(openLog:))]
        fn open_log(&self, _sender: Option<&AnyObject>) {
            crate::logging::flush();
            if let Some(p) = crate::logging::log_path() {
                let _ = std::process::Command::new("/usr/bin/open").arg("-a").arg("Console").arg(&p).spawn();
            }
        }

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, _sender: Option<&AnyObject>) {
            if let Some(p) = ui_value(|u| u.config_path.clone()) {
                if let Some(dir) = p.parent() {
                    let _ = std::process::Command::new("/usr/bin/open").arg(dir).spawn();
                }
            }
        }

        #[unsafe(method(openAccessibility:))]
        fn open_accessibility(&self, _sender: Option<&AnyObject>) {
            let _ = std::process::Command::new("/usr/bin/open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn();
        }

        #[unsafe(method(toggleLogin:))]
        fn toggle_login(&self, _sender: Option<&AnyObject>) {
            set_login_item(!login_item_enabled());
            ui_force_refresh(self.mtm());
        }

        #[unsafe(method(returnHere:))]
        fn return_here(&self, _sender: Option<&AnyObject>) {
            MAC_PRIMARY.with(|p| {
                if let Ok(mut g) = p.try_borrow_mut() {
                    if let Some(s) = g.as_mut() {
                        s.core.return_home(&mut s.host);
                    }
                }
            });
        }

        #[unsafe(method(installUpdate:))]
        fn install_update(&self, _sender: Option<&AnyObject>) {
            if let Err(e) = crate::update::install() {
                log::warn!("{e}");
            }
        }

        #[unsafe(method(bubbleTick:))]
        fn bubble_tick(&self, _sender: Option<&AnyObject>) {
            bubble_tick();
        }

        #[unsafe(method(goTo:))]
        fn go_to(&self, sender: Option<&AnyObject>) {
            let Some(sender) = sender else { return };
            // SAFETY: l'expéditeur est une entrée de menu (NSMenuItem).
            let tag: isize = unsafe { msg_send![sender, tag] };
            let Some(name) = ui_value(|u| u.goto_names.get(tag as usize).cloned()).flatten() else { return };
            MAC_PRIMARY.with(|p| {
                if let Ok(mut g) = p.try_borrow_mut() {
                    if let Some(s) = g.as_mut() {
                        s.core.switch_to(&mut s.host, None, SwitchTo::Client(name));
                    }
                }
            });
        }

        #[unsafe(method(toggleLock:))]
        fn toggle_lock(&self, _sender: Option<&AnyObject>) {
            MAC_PRIMARY.with(|p| {
                if let Ok(mut g) = p.try_borrow_mut() {
                    if let Some(s) = g.as_mut() {
                        s.core.toggle_lock();
                    }
                }
            });
            ui_force_refresh(self.mtm());
        }

        #[unsafe(method(restart:))]
        fn restart(&self, _sender: Option<&AnyObject>) {
            let args = ui_value(|u| u.args.clone()).unwrap_or_default();
            shutdown();
            super::relaunch(&args);
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _sender: Option<&AnyObject>) {
            shutdown();
            log::info!("arrêt demandé");
            crate::logging::flush();
            std::process::exit(0);
        }
    }
);

impl Controller {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        // SAFETY: initialisation standard d'un NSObject.
        unsafe { msg_send![super(this), init] }
    }
}

fn ui_value<R>(f: impl FnOnce(&MacUi) -> R) -> Option<R> {
    MAC_UI.with(|u| u.try_borrow().ok().and_then(|g| g.as_ref().map(f)))
}

fn menu_item(
    mtm: MainThreadMarker,
    title: &str,
    action: Option<objc2::runtime::Sel>,
    target: &Controller,
) -> Retained<NSMenuItem> {
    // SAFETY: sélecteurs définis sur Controller ; cible conservée par MacUi.
    unsafe {
        let item = NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            action,
            &NSString::from_str(""),
        );
        if action.is_some() {
            item.setTarget(Some(target));
        }
        item
    }
}

/// Nom de sauvegarde de l'icône : macOS garde sa place (déplaçable avec
/// ⌘ + glisser), y compris après un redémarrage de Baskul.
const STATUS_AUTOSAVE: &str = "fr.baskul.status";

/// Image de la barre des menus : la même icône que sous Windows (voir
/// `icon.rs`), dessinée en 36 pixels pour 18 points (écrans Retina). En
/// monochrome, c'est une image « modèle » : macOS la colore comme les autres
/// icônes de la barre des menus (claire ou sombre, sélectionnée…).
fn status_image(snap: &Snapshot, style: TrayIcon) -> Option<Retained<NSImage>> {
    const PX: u32 = 36;
    let mono = style == TrayIcon::Mono;
    let rgba = icon::render(
        PX,
        Tone::of(&snap.conn),
        snap.remote_active,
        if mono { icon::Style::Mono([0, 0, 0]) } else { icon::Style::Color },
    );
    let png = crate::clipboard::encode_png(PX, PX, &rgba)?;
    let img = NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(&png))?;
    img.setSize(NSSize::new(18.0, 18.0));
    img.setTemplate(mono);
    Some(img)
}

fn update_status_item(ui: &MacUi, mtm: MainThreadMarker, snap: &Snapshot) {
    if let Some(button) = ui.item.button(mtm) {
        match status_image(snap, ui.tray_icon) {
            Some(img) => {
                button.setImage(Some(&img));
                button.setTitle(&NSString::from_str(""));
            }
            None => {
                button.setImage(None);
                button.setTitle(&NSString::from_str("⌨︎"));
            }
        }
        button.setToolTip(Some(&NSString::from_str(&format!("Baskul — {}", snap.summary()))));
    }
    ui.status_line.setTitle(&NSString::from_str(&snap.summary()));
}

/// Remplace l'icône de la barre des menus par une nouvelle (même menu, même
/// place).
fn recreate_status_item(ui: &mut MacUi, mtm: MainThreadMarker) {
    let bar = NSStatusBar::systemStatusBar();
    bar.removeStatusItem(&ui.item);
    let item = bar.statusItemWithLength(-1.0);
    item.setAutosaveName(Some(&NSString::from_str(STATUS_AUTOSAVE)));
    item.setMenu(Some(&ui.menu));
    ui.item = item;
    ui.created_at = Instant::now();
    ui.recreated_at = Some(ui.created_at);
    ui.hidden_since = None;
    let snap = status::snapshot();
    update_status_item(ui, mtm, &snap);
    ui.last = Some(snap);
}

fn ui_force_refresh(mtm: MainThreadMarker) {
    MAC_UI.with(|u| {
        if let Ok(mut g) = u.try_borrow_mut() {
            if let Some(ui) = g.as_mut() {
                ui.last = None;
            }
        }
    });
    ui_tick(mtm);
}

fn ui_tick(mtm: MainThreadMarker) {
    // Redémarrage demandé par l'app de réglages.
    if super::take_restart_request() {
        let args = ui_value(|u| u.args.clone()).unwrap_or_default();
        shutdown();
        super::relaunch(&args);
    }
    let role = match ui_value(|u| u.role) {
        Some(r) => r,
        None => return,
    };
    // Autorisations.
    let ok = match role {
        Role::Secondary => post_access(),
        Role::Primary => primary_permissions_ok(),
    };
    let was_ok = ui_value(|u| u.permissions_ok).unwrap_or(true);
    if ok != was_ok {
        MAC_UI.with(|u| {
            if let Ok(mut g) = u.try_borrow_mut() {
                if let Some(ui) = g.as_mut() {
                    ui.permissions_ok = ok;
                }
            }
        });
        if ok {
            log::info!("autorisations macOS accordées");
            status::set_warning(None);
            if role == Role::Primary {
                ensure_tap();
            }
        } else {
            status::set_warning(Some(i18n::t("status.needAccessibility")));
        }
    }
    if role == Role::Primary {
        drain_peer_events();
    }
    let snap = status::snapshot();
    let locked = MAC_PRIMARY.with(|p| p.try_borrow().ok().and_then(|g| g.as_ref().map(|s| s.core.is_locked())));
    MAC_UI.with(|u| {
        let Ok(mut g) = u.try_borrow_mut() else { return };
        let Some(ui) = g.as_mut() else { return };
        if ui.last.as_ref() != Some(&snap) {
            update_status_item(ui, mtm, &snap);
            ui.last = Some(snap);
        }
        if !ui.item.isVisible() {
            log::warn!("icône de la barre des menus masquée : réaffichage");
            ui.item.setVisible(true);
        }
        // Image perdue (bouton vide) : on la redessine.
        if ui.item.button(mtm).is_some_and(|b| b.image().is_none() && b.title().length() == 0) && ui.last.is_some() {
            log::warn!("icône de la barre des menus sans image : nouveau dessin");
            update_status_item(ui, mtm, &status::snapshot());
        }
        // Icône absente de l'écran (cachée par macOS, ou disparue) : notée
        // dans le journal, puis recréée si cela dure.
        let on_screen = ui.created_at.elapsed() < Duration::from_secs(5)
            || ui
                .item
                .button(mtm)
                .and_then(|b| b.window())
                .is_some_and(|w| w.occlusionState().contains(NSWindowOcclusionState::Visible));
        match (on_screen, ui.hidden_since) {
            (true, Some(_)) => {
                log::info!("icône de la barre des menus de nouveau visible");
                ui.hidden_since = None;
            }
            (false, None) => {
                log::info!("icône de la barre des menus absente de l'écran");
                ui.hidden_since = Some(Instant::now());
            }
            (false, Some(since)) => {
                let due = ui.recreated_at.is_none_or(|t| t.elapsed() > Duration::from_secs(120));
                if since.elapsed() > Duration::from_secs(10) && due {
                    log::warn!("icône de la barre des menus toujours absente : recréation");
                    recreate_status_item(ui, mtm);
                }
            }
            (true, None) => {}
        }
        ui.access_item.setHidden(ui.permissions_ok);
        let ready = crate::update::ready_version();
        if ready != ui.update_ready {
            if let Some(v) = &ready {
                ui.update_item.setTitle(&NSString::from_str(&i18n::tf("menu.updateReady", &[("version", v)])));
            }
            ui.update_item.setHidden(ready.is_none());
            ui.update_ready = ready;
        }
        ui.login_item.setState(if login_item_enabled() { NSControlStateValueOn } else { NSControlStateValueOff });
        if let (Some(item), Some(l)) = (&ui.lock_item, locked) {
            item.setState(if l { NSControlStateValueOn } else { NSControlStateValueOff });
        }
        update_goto_items(ui, mtm);
    });
}

/// Entrées « Aller sur… » : une par pilotée connectée, cochée pour celle où
/// agissent le clavier et la souris.
fn update_goto_items(ui: &mut MacUi, mtm: MainThreadMarker) {
    let Some(anchor) = &ui.return_item else { return };
    let snap = status::snapshot();
    let names: Vec<String> = snap.peers.iter().map(|p| p.name.clone()).collect();
    if names != ui.goto_names {
        for item in ui.goto_items.drain(..) {
            ui.menu.removeItem(&item);
        }
        let at = ui.menu.indexOfItem(anchor) + 1;
        for (i, name) in names.iter().enumerate() {
            let item = menu_item(mtm, &i18n::tf("menu.goTo", &[("name", name)]), Some(sel!(goTo:)), &ui._controller);
            item.setTag(i as isize);
            ui.menu.insertItem_atIndex(&item, at + i as isize);
            ui.goto_items.push(item);
        }
        ui.goto_names = names;
    }
    for (item, name) in ui.goto_items.iter().zip(&ui.goto_names) {
        let on = snap.active_peer.as_deref() == Some(name.as_str());
        item.setState(if on { NSControlStateValueOn } else { NSControlStateValueOff });
    }
}

fn ensure_tap() {
    let need =
        MAC_PRIMARY.with(|p| p.try_borrow().ok().and_then(|g| g.as_ref().map(|s| s.tap.is_null())).unwrap_or(false));
    if !need {
        return;
    }
    let tap = create_tap();
    if tap.is_null() {
        log::error!("capture du clavier et de la souris impossible (autorisations ?)");
        status::set_warning(Some(i18n::t("status.captureFailed")));
        return;
    }
    log::info!("capture du clavier et de la souris active");
    MAC_PRIMARY.with(|p| {
        if let Ok(mut g) = p.try_borrow_mut() {
            if let Some(s) = g.as_mut() {
                s.tap = tap;
            }
        }
    });
}

fn shutdown() {
    MAC_PRIMARY.with(|p| {
        if let Ok(mut g) = p.try_borrow_mut() {
            if let Some(s) = g.as_mut() {
                s.core.reset(&mut s.host);
            }
        }
    });
    std::thread::sleep(Duration::from_millis(150));
}

// ---------------------------------------------------------------------------
// Bulle d'arrivée
// ---------------------------------------------------------------------------

/// Nom affiché par la bulle (absent : bulle désactivée).
static BUBBLE_NAME: OnceLock<String> = OnceLock::new();
/// Durée d'affichage avant le fondu, puis durée du fondu.
const BUBBLE_SHOW: f64 = 0.9;
const BUBBLE_FADE: f64 = 0.35;

struct Bubble {
    panel: Retained<NSPanel>,
    background: Retained<NSBox>,
    label: Retained<NSTextField>,
    shown_at: Instant,
}

thread_local! {
    static BUBBLE: RefCell<Option<Bubble>> = const { RefCell::new(None) };
    static BUBBLE_FADING: Cell<bool> = const { Cell::new(false) };
}

unsafe extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_async_f(queue: *const c_void, context: *mut c_void, work: unsafe extern "C" fn(*mut c_void));
}

/// Le clavier et la souris arrivent sur cette machine : son nom s'affiche un
/// instant près du curseur. Utilisable depuis n'importe quel thread.
fn request_bubble() {
    if BUBBLE_NAME.get().is_none() {
        return;
    }
    unsafe extern "C" fn show(_: *mut c_void) {
        if let Some(mtm) = MainThreadMarker::new() {
            autoreleasepool(|_| show_bubble(mtm));
        }
    }
    // SAFETY: file d'attente principale de libdispatch, fonction sans contexte.
    unsafe { dispatch_async_f(&raw const _dispatch_main_q, null_mut(), show) };
}

/// Fenêtre de la bulle : sans bordure, ne prend pas le focus, laisse passer
/// les clics, visible sur tous les bureaux et par-dessus le plein écran.
fn make_bubble(mtm: MainThreadMarker) -> Bubble {
    let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(120.0, 36.0));
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
        NSPanel::alloc(mtm),
        rect,
        NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
        NSBackingStoreType::Buffered,
        false,
    );
    panel.setOpaque(false);
    panel.setBackgroundColor(Some(&NSColor::clearColor()));
    panel.setHasShadow(true);
    panel.setIgnoresMouseEvents(true);
    panel.setHidesOnDeactivate(false);
    panel.setFloatingPanel(true);
    panel.setLevel(NSPopUpMenuWindowLevel);
    panel.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    let content = NSView::initWithFrame(NSView::alloc(mtm), rect);
    let background = NSBox::initWithFrame(NSBox::alloc(mtm), rect);
    background.setBoxType(NSBoxType::Custom);
    background.setTitlePosition(NSTitlePosition::NoTitle);
    background.setBorderWidth(0.0);
    background.setFillColor(&NSColor::colorWithWhite_alpha(0.12, 0.85));
    let label = NSTextField::labelWithString(&NSString::from_str(""), mtm);
    label.setFont(Some(&NSFont::boldSystemFontOfSize(15.0)));
    label.setTextColor(Some(&NSColor::whiteColor()));
    content.addSubview(&background);
    content.addSubview(&label);
    panel.setContentView(Some(&content));
    Bubble { panel, background, label, shown_at: Instant::now() }
}

fn show_bubble(mtm: MainThreadMarker) {
    let Some(name) = BUBBLE_NAME.get() else { return };
    let Some(controller) = ui_value(|u| u._controller.clone()) else { return };
    BUBBLE.with(|b| {
        let mut b = b.borrow_mut();
        let bubble = b.get_or_insert_with(|| make_bubble(mtm));
        // Logo Apple (caractère réservé, dessiné par les polices du système).
        bubble.label.setStringValue(&NSString::from_str(&format!("\u{F8FF}  {name}")));
        bubble.label.sizeToFit();
        let text = bubble.label.frame().size;
        let (pad_x, pad_y) = (16.0, 7.0);
        let size = NSSize::new(text.width + 2.0 * pad_x, text.height + 2.0 * pad_y);
        // En bas à droite du curseur, sans sortir de l'écran.
        let m = NSEvent::mouseLocation();
        let mut origin = NSPoint::new(m.x + 18.0, m.y - 22.0 - size.height);
        for screen in NSScreen::screens(mtm).iter() {
            let f = screen.frame();
            let inside = m.x >= f.origin.x
                && m.x <= f.origin.x + f.size.width
                && m.y >= f.origin.y
                && m.y <= f.origin.y + f.size.height;
            if inside {
                let v = screen.visibleFrame();
                origin.x = origin.x.min(v.origin.x + v.size.width - size.width - 6.0).max(v.origin.x + 6.0);
                origin.y = origin.y.max(v.origin.y + 6.0).min(v.origin.y + v.size.height - size.height - 6.0);
            }
        }
        let full = NSRect::new(NSPoint::new(0.0, 0.0), size);
        bubble.panel.setFrame_display(NSRect::new(origin, size), false);
        bubble.background.setFrame(full);
        bubble.background.setCornerRadius(size.height / 2.0);
        bubble.label.setFrameOrigin(NSPoint::new(pad_x, pad_y));
        // Une arrivée pendant le fondu l'interrompt.
        NSAnimationContext::beginGrouping();
        NSAnimationContext::currentContext().setDuration(0.0);
        bubble.panel.animator().setAlphaValue(1.0);
        NSAnimationContext::endGrouping();
        bubble.panel.setAlphaValue(1.0);
        bubble.panel.orderFrontRegardless();
        bubble.shown_at = Instant::now();
    });
    BUBBLE_FADING.set(false);
    for delay in [BUBBLE_SHOW, BUBBLE_SHOW + BUBBLE_FADE + 0.05] {
        // SAFETY: minuterie unique ciblant `bubbleTick:` sur notre contrôleur.
        unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                delay,
                &controller,
                sel!(bubbleTick:),
                None,
                false,
            );
        }
    }
}

/// Fondu, puis fenêtre retirée (une nouvelle arrivée entre-temps repousse
/// l'échéance).
fn bubble_tick() {
    BUBBLE.with(|b| {
        let b = b.borrow();
        let Some(bubble) = b.as_ref() else { return };
        let elapsed = bubble.shown_at.elapsed().as_secs_f64();
        if elapsed >= BUBBLE_SHOW + BUBBLE_FADE {
            bubble.panel.orderOut(None);
        } else if elapsed >= BUBBLE_SHOW - 0.05 && !BUBBLE_FADING.get() {
            BUBBLE_FADING.set(true);
            NSAnimationContext::beginGrouping();
            NSAnimationContext::currentContext().setDuration(BUBBLE_FADE);
            bubble.panel.animator().setAlphaValue(0.0);
            NSAnimationContext::endGrouping();
        }
    });
}

// ---------------------------------------------------------------------------
// Ouverture à la connexion (LaunchAgent)
// ---------------------------------------------------------------------------

const LOGIN_LABEL: &str = "fr.baskul.app";

fn login_plist() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join("Library/LaunchAgents").join(format!("{LOGIN_LABEL}.plist")))
}

fn login_item_enabled() -> bool {
    login_plist().is_some_and(|p| p.exists())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Ancien nom (Bascule) : son ouverture à la connexion passe à Baskul.
fn migrate_old_login_item() {
    let Some(old) = dirs::home_dir().map(|h| h.join("Library/LaunchAgents/fr.bascule.app.plist")) else { return };
    if old.exists() && std::fs::remove_file(&old).is_ok() {
        log::info!("ouverture à la connexion reprise de {}", crate::config::OLD_NAME);
        set_login_item(true);
    }
}

fn set_login_item(on: bool) {
    let Some(path) = login_plist() else { return };
    if !on {
        match std::fs::remove_file(&path) {
            Ok(()) => log::info!("ouverture à la connexion désactivée"),
            Err(e) => log::warn!("suppression de {} impossible : {e}", path.display()),
        }
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let bundle = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).map(Path::to_path_buf);
    let program = match &bundle {
        Some(b) => format!(
            "<string>/usr/bin/open</string><string>-a</string><string>{}</string>",
            xml_escape(&b.display().to_string())
        ),
        None => format!("<string>{}</string>", xml_escape(&exe.display().to_string())),
    };
    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\"><dict>\n\
         <key>Label</key><string>{LOGIN_LABEL}</string>\n\
         <key>ProgramArguments</key><array>{program}</array>\n\
         <key>RunAtLoad</key><true/>\n\
         <key>ProcessType</key><string>Interactive</string>\n\
         </dict></plist>\n"
    );
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match std::fs::write(&path, plist) {
        Ok(()) => log::info!("ouverture à la connexion activée"),
        Err(e) => log::warn!("écriture de {} impossible : {e}", path.display()),
    }
}

// ---------------------------------------------------------------------------
// Fonctions publiques
// ---------------------------------------------------------------------------

fn applescript_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

pub fn early_init(_args: &Args) {}

pub fn fatal(msg: &str) -> ! {
    log::error!("{msg}");
    crate::logging::flush();
    eprintln!("Baskul : {msg}");
    let script = format!("display alert \"Baskul\" message {} as critical buttons {{\"OK\"}}", applescript_string(msg));
    let _ = std::process::Command::new("/usr/bin/osascript").arg("-e").arg(script).status();
    std::process::exit(1)
}

pub fn first_run(path: &Path) {
    super::open_settings(path, true);
}

pub fn open_in_editor(path: &Path) {
    let _ = std::process::Command::new("/usr/bin/open").arg("-t").arg(path).spawn();
}

pub fn run(settings: Settings, config_path: PathBuf, args: Args) -> ! {
    let Some(mtm) = MainThreadMarker::new() else {
        fatal("Baskul doit démarrer sur le thread principal");
    };
    migrate_old_login_item();
    let app = NSApplication::sharedApplication(mtm);
    // Pas d'icône dans le Dock : uniquement la barre des menus.
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    // Pas d'« App Nap » : sans fenêtre visible, macOS ralentirait sinon les
    // minuteries et les threads de Baskul (latence de la souris).
    let activity = NSProcessInfo::processInfo().beginActivityWithOptions_reason(
        NSActivityOptions::UserInitiatedAllowingIdleSystemSleep | NSActivityOptions::LatencyCritical,
        &NSString::from_str("Partage du clavier et de la souris"),
    );
    std::mem::forget(activity);

    crate::api::set_restart_handler(super::request_restart);
    if settings.ui.arrival_bubble {
        let _ = BUBBLE_NAME.set(settings.name.clone());
    }
    let controller = Controller::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*controller)));
    let item = NSStatusBar::systemStatusBar().statusItemWithLength(-1.0);
    item.setAutosaveName(Some(&NSString::from_str(STATUS_AUTOSAVE)));
    let menu = NSMenu::new(mtm);
    menu.setAutoenablesItems(false);

    let status_line = menu_item(mtm, "Baskul", None, &controller);
    status_line.setEnabled(false);
    menu.addItem(&status_line);
    let access_item = menu_item(mtm, &i18n::t("menu.allowAccessibility"), Some(sel!(openAccessibility:)), &controller);
    access_item.setHidden(true);
    menu.addItem(&access_item);
    let update_item = menu_item(mtm, "", Some(sel!(installUpdate:)), &controller);
    update_item.setHidden(true);
    menu.addItem(&update_item);
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    let (return_item, lock_item) = if settings.role == Role::Primary {
        let r = menu_item(mtm, &i18n::t("menu.returnHere"), Some(sel!(returnHere:)), &controller);
        menu.addItem(&r);
        // Les entrées « Aller sur… » des pilotées connectées s'insèrent ici.
        let l = menu_item(mtm, &i18n::t("menu.lock"), Some(sel!(toggleLock:)), &controller);
        menu.addItem(&l);
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        (Some(r), Some(l))
    } else {
        (None, None)
    };
    menu.addItem(&menu_item(mtm, &i18n::t("menu.settings"), Some(sel!(openSettings:)), &controller));
    menu.addItem(&menu_item(mtm, &i18n::t("menu.openLog"), Some(sel!(openLog:)), &controller));
    menu.addItem(&menu_item(mtm, &i18n::t("menu.openFolder"), Some(sel!(openFolder:)), &controller));
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    let login_item = menu_item(mtm, &i18n::t("menu.openAtLogin"), Some(sel!(toggleLogin:)), &controller);
    menu.addItem(&login_item);
    menu.addItem(&NSMenuItem::separatorItem(mtm));
    menu.addItem(&menu_item(mtm, &i18n::t("menu.restart"), Some(sel!(restart:)), &controller));
    menu.addItem(&menu_item(mtm, &i18n::t("menu.quit"), Some(sel!(quit:)), &controller));
    item.setMenu(Some(&menu));

    let hooks = ClipboardHooks {
        sequence: clip_sequence,
        sensitive: clip_sensitive,
        read_images: clip_read_images,
        write: clip_write,
        formats: clip_formats,
    };
    let permissions_ok;
    let _services: Services = match settings.role {
        Role::Secondary => {
            permissions_ok = post_access();
            if !permissions_ok {
                log::warn!("autorisation « Accessibilité » manquante : demande à macOS");
                // SAFETY: affiche la demande d'autorisation du système.
                unsafe { CGRequestPostEventAccess() };
                status::set_warning(Some(i18n::t("status.needAccessibility")));
            }
            app::start_secondary(&settings, hooks, MacEmulator::new(&settings), idle_time)
        }
        Role::Primary => {
            log::warn!("macOS en machine principale : fonctionnement expérimental");
            // Source de réveil de la boucle principale pour les événements réseau.
            // SAFETY: contexte copié par CFRunLoopSourceCreate ; source et boucle
            // conservées pour toute la durée du programme.
            unsafe {
                let mut ctx = CFRunLoopSourceContext {
                    version: 0,
                    info: null_mut(),
                    retain: null(),
                    release: null(),
                    copy_description: null(),
                    equal: null(),
                    hash: null(),
                    schedule: null(),
                    cancel: null(),
                    perform: Some(peer_perform),
                };
                let src = CFRunLoopSourceCreate(null(), 0, &mut ctx);
                let rl = CFRunLoopGetMain();
                CFRunLoopAddSource(rl, src, kCFRunLoopCommonModes);
                PEER_SOURCE.store(src as usize, Ordering::Release);
                MAIN_LOOP.store(rl as usize, Ordering::Release);
            }
            let (services, rx) = app::start_primary(&settings, hooks, wake_main);
            let (iso, _) = keyboard_is_iso();
            let iso_active = match settings.iso_swap {
                IsoSwap::On => true,
                IsoSwap::Off => false,
                IsoSwap::Auto => iso,
            };
            let opts = PrimaryOptions {
                edge_switching: settings.edge_switching,
                new_client_side: settings.secondary_position,
                corner: settings.corner_size,
                layout: settings.layout.clone(),
                switch_hotkey: settings.switch_hotkey.clone(),
                return_hotkey: settings.return_hotkey.clone(),
                next_hotkey: settings.next_hotkey.clone(),
                lock_hotkey: settings.lock_hotkey.clone(),
                media_keys: settings.media_keys,
                mirror_activity: settings.mirror_activity,
            };
            // Le crochet clavier peut relever les raccourcis pour les réglages.
            crate::capture::set_supported(true);
            MAC_PRIMARY.with(|p| {
                *p.borrow_mut() = Some(MacPrimary {
                    core: PrimaryCore::new(opts),
                    host: MacHost {
                        layout: displays(),
                        layout_at: Instant::now(),
                        link: services.link.clone(),
                        hidden: false,
                        config_path: config_path.clone(),
                    },
                    peer_rx: rx,
                    last_loc: current_cursor().unwrap_or_default(),
                    tap: null_mut(),
                    iso_active,
                    acc_x: 0.0,
                    acc_y: 0.0,
                })
            });
            permissions_ok = primary_permissions_ok();
            if permissions_ok {
                ensure_tap();
            } else {
                log::warn!("autorisations manquantes (Accessibilité, Surveillance de l'entrée)");
                // SAFETY: affiche les demandes d'autorisation du système.
                unsafe {
                    CGRequestListenEventAccess();
                    CGRequestPostEventAccess();
                }
                status::set_warning(Some(i18n::t("status.needAccessibility")));
            }
            services
        }
    };

    MAC_UI.with(|u| {
        *u.borrow_mut() = Some(MacUi {
            item,
            status_line,
            access_item,
            update_item,
            update_ready: None,
            return_item,
            lock_item,
            goto_items: Vec::new(),
            goto_names: Vec::new(),
            login_item,
            _controller: controller.clone(),
            last: None,
            menu: menu.clone(),
            created_at: Instant::now(),
            hidden_since: None,
            recreated_at: None,
            tray_icon: settings.ui.tray_icon,
            role: settings.role,
            config_path,
            args,
            permissions_ok,
        })
    });
    ui_tick(mtm);
    // SAFETY: minuterie répétée ciblant `tick:` sur notre contrôleur.
    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            0.5,
            &controller,
            sel!(tick:),
            None,
            true,
        );
    }
    app.run();
    shutdown();
    std::process::exit(0)
}
