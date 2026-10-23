//! Windows.
//!
//! - Machine principale : hooks bas niveau clavier/souris (`WH_KEYBOARD_LL`,
//!   `WH_MOUSE_LL`) installés sur le thread de l'interface. Quand le curseur
//!   part sur une pilotée, il est recentré et caché derrière une petite
//!   fenêtre transparente ; les événements sont alors supprimés localement et
//!   envoyés à la pilotée.
//! - Machine pilotée : injection par `SendInput`.
//! - Icône dans la zone de notification avec menu.
//!
//! Les touches sont capturées par **scancode** (position physique), jamais
//! par caractère : aucune fonction de traduction (`ToUnicode`…) n'est appelée,
//! l'état des touches mortes de Windows n'est donc jamais perturbé.

use crate::Args;
use crate::app::{self, ClipboardHooks, Services};
use crate::config::{MachineCfg, ScrollCfg, Settings, TrayIcon};
use crate::geometry::{Layout, Point, Rect};
use crate::i18n;
use crate::icon::{self, Style, Tone};
use crate::keymap::{self, *};
use crate::net::{LinkHandle, PeerId};
use crate::primary::{Decision, Host, PeerEvent, PrimaryCore, PrimaryOptions};
use crate::protocol::{ClipHeader, ClipImage, ImageFormat, Message, MouseButton, Role, SwitchTo};
use crate::secondary::Emulator;
use crate::status::{self, Conn};
use crossbeam_channel::Receiver;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{
    ERROR_SUCCESS, GlobalFree, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, ANTIALIASED_QUALITY, BI_BITFIELDS, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BITMAPV5HEADER,
    BLENDFUNCTION, CLIP_DEFAULT_PRECIS, CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW,
    DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DeleteDC, DeleteObject, EnumDisplayMonitors, FW_SEMIBOLD, GdiFlush,
    GetDC, GetMonitorInfoW, GetTextExtentPoint32W, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    MonitorFromPoint, OUT_DEFAULT_PRECIS, ReleaseDC, SelectObject, SetBkMode, SetTextColor, TRANSPARENT, TextOutW,
};
use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardFormatNameW,
    GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock};
use windows_sys::Win32::System::Power::{ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState};
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows_sys::Win32::System::RemoteDesktop::{NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification};
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::System::Threading::{
    ABOVE_NORMAL_PRIORITY_CLASS, AttachThreadInput, GetCurrentProcess, GetCurrentThread, GetCurrentThreadId,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    ProcessPowerThrottling, SetPriorityClass, SetProcessInformation, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
};
use windows_sys::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor, MDT_EFFECTIVE_DPI, SetProcessDpiAwarenessContext,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, LASTINPUTINFO, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
    ReleaseCapture, SendInput, SetCapture, VK_BROWSER_BACK, VK_BROWSER_FORWARD, VK_LCONTROL, VK_MEDIA_NEXT_TRACK,
    VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_NUMLOCK, VK_PAUSE, VK_VOLUME_DOWN, VK_VOLUME_MUTE,
    VK_VOLUME_UP,
};
use windows_sys::Win32::UI::Shell::{
    DROPFILES, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW,
    ShellExecuteW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

// ---------------------------------------------------------------------------
// Constantes et état global
// ---------------------------------------------------------------------------

const WM_APP_TRAY: u32 = WM_APP + 1;
const WM_APP_NET: u32 = WM_APP + 2;
const WM_APP_STATUS: u32 = WM_APP + 3;
const WM_APP_HIDE: u32 = WM_APP + 4;
const WM_APP_SHOW: u32 = WM_APP + 5;
const WM_APP_RESTART: u32 = WM_APP + 6;
const WM_APP_ACTIVE: u32 = WM_APP + 7;
const WM_APP_BUBBLE: u32 = WM_APP + 8;

/// Marque des entrées injectées par Baskul pour l'activité partagée : son
/// propre crochet les laisse passer sans les traiter.
const BASKUL_INPUT: usize = 0x4241_5343; // "BASC"

const TIMER_WATCHDOG: usize = 1;
const TIMER_REHOOK: usize = 2;
const TIMER_TRAY: usize = 3;
const TIMER_BUBBLE: usize = 4;

const ID_RETURN: usize = 10;
const ID_LOCK: usize = 11;
const ID_SETTINGS: usize = 12;
const ID_LOG: usize = 13;
const ID_FOLDER: usize = 14;
const ID_AUTOSTART: usize = 15;
const ID_RESTART: usize = 16;
const ID_QUIT: usize = 17;
const ID_UPDATE: usize = 18;
/// Entrées « Aller sur… » : une par pilotée connectée.
const ID_GOTO: usize = 100;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "Baskul";

/// Marqueur dans le scancode du faux Ctrl gauche que Windows génère avec AltGr.
const FAKE_LCTRL_BIT: u32 = 0x200;

static MAIN_HWND: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn post(msg: u32, w: WPARAM, l: LPARAM) {
    let h = MAIN_HWND.load(Ordering::Acquire);
    if !h.is_null() {
        // SAFETY: PostMessageW est utilisable depuis n'importe quel thread.
        unsafe { PostMessageW(h, msg, w, l) };
    }
}

fn cursor_pos() -> Option<Point> {
    let mut p = POINT { x: 0, y: 0 };
    // SAFETY: pointeur valide vers une structure POINT.
    if unsafe { GetCursorPos(&mut p) } != 0 { Some(Point::new(p.x, p.y)) } else { None }
}

fn monitors() -> Layout {
    unsafe extern "system" fn cb(hmon: HMONITOR, _hdc: HDC, _rc: *mut RECT, data: LPARAM) -> windows_sys::core::BOOL {
        // SAFETY: `data` pointe vers le Vec passé à EnumDisplayMonitors ci-dessous.
        let v = unsafe { &mut *(data as *mut Vec<Rect>) };
        let mut mi: MONITORINFO = unsafe { zeroed() };
        mi.cbSize = size_of::<MONITORINFO>() as u32;
        if unsafe { GetMonitorInfoW(hmon, &mut mi) } != 0 {
            let r = mi.rcMonitor;
            v.push(Rect::new(r.left, r.top, r.right, r.bottom));
        }
        1
    }
    let mut v: Vec<Rect> = Vec::new();
    // SAFETY: le rappel ne vit que pendant l'appel.
    unsafe { EnumDisplayMonitors(null_mut(), null(), Some(cb), &mut v as *mut Vec<Rect> as LPARAM) };
    Layout::new(v)
}

fn message_box(text: &str, flags: MESSAGEBOX_STYLE) {
    let t = wide(text);
    let c = wide("Baskul");
    // SAFETY: chaînes terminées par un zéro.
    unsafe { MessageBoxW(null_mut(), t.as_ptr(), c.as_ptr(), flags | MB_SETFOREGROUND | MB_TOPMOST) };
}

fn shell_open(file: &str, params: Option<&str>) {
    let op = wide("open");
    let f = wide(file);
    let p = params.map(wide);
    // SAFETY: chaînes terminées par un zéro.
    unsafe {
        ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            f.as_ptr(),
            p.as_ref().map_or(null(), |p| p.as_ptr()),
            null(),
            SW_SHOWNORMAL,
        )
    };
}

// ---------------------------------------------------------------------------
// Fonctions publiques
// ---------------------------------------------------------------------------

pub fn early_init(args: &Args) {
    // SAFETY: appels sans précondition.
    unsafe {
        // Journal ou résultat d'une commande dans le terminal qui a lancé Baskul.
        if args.console || args.command.is_some() {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
        // Coordonnées en pixels physiques sur tous les écrans, quel que soit
        // le facteur d'échelle.
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        // Réactivité : priorité un peu au-dessus de la normale, et pas de
        // ralentissement « économie d'énergie » de Windows 11 pour ce
        // processus sans fenêtre visible.
        SetPriorityClass(GetCurrentProcess(), ABOVE_NORMAL_PRIORITY_CLASS);
        let state = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: 0,
        };
        SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &state as *const PROCESS_POWER_THROTTLING_STATE as *const c_void,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

pub fn fatal(msg: &str) -> ! {
    log::error!("{msg}");
    crate::logging::flush();
    message_box(msg, MB_OK | MB_ICONERROR);
    std::process::exit(1)
}

pub fn first_run(path: &Path) {
    super::open_settings(path, true);
}

pub fn open_in_editor(path: &Path) {
    shell_open("notepad.exe", Some(&format!("\"{}\"", path.display())));
}

fn clip_sequence() -> Option<u64> {
    // SAFETY: appel sans précondition.
    Some(unsafe { GetClipboardSequenceNumber() } as u64)
}

fn clip_sensitive() -> bool {
    // Formats posés par les gestionnaires de mots de passe (KeePass,
    // 1Password, Bitwarden…) pour exclure leur contenu des outils de
    // synchronisation.
    for name in ["ExcludeClipboardContentFromMonitorProcessing", "Clipboard Viewer Ignore"] {
        let w = wide(name);
        // SAFETY: chaîne terminée par un zéro.
        unsafe {
            let f = RegisterClipboardFormatW(w.as_ptr());
            if f != 0 && IsClipboardFormatAvailable(f) != 0 {
                return true;
            }
        }
    }
    false
}

/// Formats présents dans le presse-papiers, pour le journal.
fn clip_formats() -> Vec<String> {
    const STANDARD: [(u32, &str); 10] = [
        (1, "CF_TEXT"),
        (2, "CF_BITMAP"),
        (3, "CF_METAFILEPICT"),
        (7, "CF_OEMTEXT"),
        (8, "CF_DIB"),
        (13, "CF_UNICODETEXT"),
        (14, "CF_ENHMETAFILE"),
        (15, "CF_HDROP"),
        (16, "CF_LOCALE"),
        (17, "CF_DIBV5"),
    ];
    let mut out = vec![];
    // SAFETY: presse-papiers ouvert puis refermé dans ce bloc ; tampon de
    // taille annoncée.
    unsafe {
        if OpenClipboard(null_mut()) == 0 {
            return out;
        }
        let mut f = 0;
        while out.len() < 10 {
            f = EnumClipboardFormats(f);
            if f == 0 {
                break;
            }
            let name = match STANDARD.iter().find(|(id, _)| *id == f) {
                Some((_, name)) => name.to_string(),
                None => {
                    let mut buf = [0u16; 128];
                    let len = GetClipboardFormatNameW(f, buf.as_mut_ptr(), buf.len() as i32);
                    if len > 0 { String::from_utf16_lossy(&buf[..len as usize]) } else { format!("#{f}") }
                }
            };
            out.push(name);
        }
        CloseClipboard();
    }
    out
}

const CF_UNICODETEXT: u32 = 13;
const CF_HDROP: u32 = 15;
const CF_DIBV5: u32 = 17;

/// Formats d'image enregistrés par les applications (navigateurs, Office,
/// Paint.NET, GIMP…) : l'image d'origine, sans conversion.
const IMAGE_FORMATS: [(&str, ImageFormat); 7] = [
    ("PNG", ImageFormat::Png),
    ("image/png", ImageFormat::Png),
    ("JFIF", ImageFormat::Jpeg),
    ("image/jpeg", ImageFormat::Jpeg),
    ("GIF", ImageFormat::Gif),
    ("image/gif", ImageFormat::Gif),
    ("image/webp", ImageFormat::Webp),
];

fn clip_format_id(name: &str) -> u32 {
    let w = wide(name);
    // SAFETY: chaîne terminée par un zéro.
    unsafe { RegisterClipboardFormatW(w.as_ptr()) }
}

/// Presse-papiers ouvert, refermé à la fin. Un autre programme peut le garder
/// ouvert un instant : quelques essais.
struct OpenClip;

impl OpenClip {
    fn open() -> Option<OpenClip> {
        // Fenêtre de Baskul comme propriétaire (sans elle, l'écriture échoue).
        let owner = MAIN_HWND.load(Ordering::Acquire);
        for _ in 0..10 {
            // SAFETY: fenêtre de Baskul (ou aucune).
            if unsafe { OpenClipboard(owner) } != 0 {
                return Some(OpenClip);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        log::warn!("presse-papiers occupé par un autre programme");
        None
    }

    fn get(&self, format: u32) -> Option<Vec<u8>> {
        // SAFETY: presse-papiers ouvert ; mémoire verrouillée le temps de la
        // copie, de la taille annoncée par le système.
        unsafe {
            let h = GetClipboardData(format);
            if h.is_null() {
                return None;
            }
            let size = GlobalSize(h);
            let p = GlobalLock(h) as *const u8;
            if p.is_null() {
                return None;
            }
            let v = std::slice::from_raw_parts(p, size).to_vec();
            GlobalUnlock(h);
            Some(v)
        }
    }

    fn set(&self, format: u32, data: &[u8]) -> bool {
        if format == 0 {
            return false;
        }
        // SAFETY: mémoire allouée et remplie ici ; elle appartient au système
        // si SetClipboardData réussit, sinon on la libère.
        unsafe {
            let h = GlobalAlloc(GMEM_MOVEABLE, data.len().max(1));
            if h.is_null() {
                return false;
            }
            let p = GlobalLock(h) as *mut u8;
            if p.is_null() {
                GlobalFree(h);
                return false;
            }
            std::ptr::copy_nonoverlapping(data.as_ptr(), p, data.len());
            GlobalUnlock(h);
            if SetClipboardData(format, h).is_null() {
                GlobalFree(h);
                return false;
            }
            true
        }
    }
}

impl Drop for OpenClip {
    fn drop(&mut self) {
        // SAFETY: presse-papiers ouvert par `open`.
        unsafe { CloseClipboard() };
    }
}

/// Image du presse-papiers dans son format d'origine (PNG, JPEG, GIF, WebP).
/// Sinon (bitmap seul) : arboard la lit et l'envoie en PNG.
fn clip_read_images() -> Vec<ClipImage> {
    let Some(clip) = OpenClip::open() else { return vec![] };
    for (name, format) in IMAGE_FORMATS {
        let id = clip_format_id(name);
        // SAFETY: presse-papiers ouvert.
        if id == 0 || unsafe { IsClipboardFormatAvailable(id) } == 0 {
            continue;
        }
        let Some(mut data) = clip.get(id) else { continue };
        if format == ImageFormat::Png {
            trim_png(&mut data);
        }
        if ImageFormat::sniff(&data) == Some(format) {
            return vec![ClipImage { format, data }];
        }
    }
    Vec::new()
}

/// La mémoire du presse-papiers est arrondie : le PNG s'arrête après son
/// bloc IEND.
fn trim_png(data: &mut Vec<u8>) {
    if let Some(i) = data.windows(4).rposition(|w| w == b"IEND") {
        data.truncate((i + 8).min(data.len()));
    }
}

/// Écrit le contenu reçu quand il comporte des images ou des fichiers (le
/// texte seul passe par arboard) : les fichiers, chaque image dans son format
/// d'origine, et un bitmap pour les applications qui ne lisent que lui.
fn clip_write(h: &ClipHeader, roots: &[PathBuf]) -> Option<bool> {
    if h.images.is_empty() && roots.is_empty() {
        return None;
    }
    let clip = OpenClip::open()?;
    // SAFETY: presse-papiers ouvert.
    unsafe { EmptyClipboard() };
    let mut ok = false;
    if !roots.is_empty() {
        ok |= clip.set(CF_HDROP, &drop_files(roots));
        // L'Explorateur colle une copie (et non un déplacement).
        const DROPEFFECT_COPY: u32 = 1;
        clip.set(clip_format_id("Preferred DropEffect"), &DROPEFFECT_COPY.to_le_bytes());
    }
    for img in &h.images {
        let name = match img.format {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JFIF",
            ImageFormat::Gif => "GIF",
            ImageFormat::Webp => "image/webp",
            _ => continue,
        };
        ok |= clip.set(clip_format_id(name), &img.data);
    }
    if let Some(dib) = h.images.iter().find_map(|i| dib_v5(&i.data)) {
        ok |= clip.set(CF_DIBV5, &dib);
    }
    if let Some(t) = &h.text {
        let w: Vec<u8> = t.encode_utf16().chain(Some(0)).flat_map(u16::to_le_bytes).collect();
        ok |= clip.set(CF_UNICODETEXT, &w);
    }
    if !ok {
        log::warn!("écriture du presse-papiers refusée par Windows");
    }
    Some(ok)
}

/// Liste de fichiers (CF_HDROP) : en-tête DROPFILES puis les chemins en
/// UTF-16, chacun terminé par un zéro, et un zéro final.
fn drop_files(paths: &[PathBuf]) -> Vec<u8> {
    let header = DROPFILES { pFiles: size_of::<DROPFILES>() as u32, pt: POINT { x: 0, y: 0 }, fNC: 0, fWide: 1 };
    // SAFETY: structure simple, lue octet par octet.
    let mut out = unsafe {
        std::slice::from_raw_parts(&header as *const DROPFILES as *const u8, size_of::<DROPFILES>()).to_vec()
    };
    for p in paths {
        out.extend(wide(&p.display().to_string()).into_iter().flat_map(u16::to_le_bytes));
    }
    out.extend([0, 0]);
    out
}

/// Bitmap (CF_DIBV5, 32 bits avec transparence, lignes de bas en haut comme
/// l'attendent Word et WordPad) ; Windows en déduit CF_DIB et CF_BITMAP.
fn dib_v5(data: &[u8]) -> Option<Vec<u8>> {
    const LCS_SRGB: u32 = 0x7352_4742;
    const LCS_GM_IMAGES: u32 = 4;
    let (w, h, rgba) = crate::clipboard::decode_image(data)?;
    // SAFETY: structure de nombres, initialisée à zéro puis remplie.
    let mut hdr: BITMAPV5HEADER = unsafe { zeroed() };
    hdr.bV5Size = size_of::<BITMAPV5HEADER>() as u32;
    hdr.bV5Width = w as i32;
    hdr.bV5Height = h as i32;
    hdr.bV5Planes = 1;
    hdr.bV5BitCount = 32;
    hdr.bV5Compression = BI_BITFIELDS;
    hdr.bV5SizeImage = w * h * 4;
    hdr.bV5RedMask = 0x00ff_0000;
    hdr.bV5GreenMask = 0x0000_ff00;
    hdr.bV5BlueMask = 0x0000_00ff;
    hdr.bV5AlphaMask = 0xff00_0000;
    hdr.bV5CSType = LCS_SRGB;
    hdr.bV5Intent = LCS_GM_IMAGES;
    // SAFETY: structure simple, lue octet par octet.
    let mut out = unsafe {
        std::slice::from_raw_parts(&hdr as *const BITMAPV5HEADER as *const u8, size_of::<BITMAPV5HEADER>()).to_vec()
    };
    out.reserve(rgba.len());
    for row in rgba.chunks_exact(w as usize * 4).rev() {
        for &[r, g, b, a] in row.as_chunks::<4>().0 {
            out.extend([b, g, r, a]);
        }
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Machine principale : capture
// ---------------------------------------------------------------------------

struct WinHost {
    layout: Layout,
    layout_dirty: bool,
    link: LinkHandle,
    config_path: PathBuf,
}

impl Host for WinHost {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn cursor(&self) -> Point {
        cursor_pos().unwrap_or_else(|| self.layout.bounds().center())
    }

    fn hide_cursor(&mut self, _at: Point) {
        // Traité juste après le retour du hook (pas d'appel bloquant dans un hook).
        post(WM_APP_HIDE, 0, 0);
    }

    fn show_cursor(&mut self, pos: Point) {
        post(WM_APP_SHOW, pos.x as isize as usize, pos.y as isize);
    }

    fn send(&mut self, to: PeerId, msg: Message) {
        self.link.send_input(to, msg);
    }

    fn keep_active(&mut self) {
        // Traité juste après le retour du hook (pas d'injection depuis un hook).
        post(WM_APP_ACTIVE, 0, 0);
    }

    fn save_layout(&mut self, machines: Vec<MachineCfg>) {
        app::save_layout(&self.config_path, machines);
    }

    fn arrived(&mut self, _pos: Point) {
        request_bubble();
    }
}

struct PrimaryState {
    core: PrimaryCore,
    host: WinHost,
    peer_rx: Receiver<PeerEvent>,
    /// Faux Ctrl d'AltGr : `Some(true)` = laissé au système local.
    fake_ctrl_local: Option<bool>,
    /// Horodatage du dernier Ctrl gauche envoyé (repli de détection AltGr).
    lctrl_down_time: Option<u32>,
    last_pt: Point,
}

struct CursorState {
    hider: HWND,
    saved_fg: HWND,
    hidden_at: Option<Point>,
    mouse_hook: HHOOK,
    kbd_hook: HHOOK,
    last_fg_fix: Option<Instant>,
}

thread_local! {
    static PRIMARY: RefCell<Option<PrimaryState>> = const { RefCell::new(None) };
    static CURSOR: RefCell<CursorState> = const {
        RefCell::new(CursorState {
            hider: null_mut(),
            saved_fg: null_mut(),
            hidden_at: None,
            mouse_hook: null_mut(),
            kbd_hook: null_mut(),
            last_fg_fix: None,
        })
    };
    static UI: RefCell<Option<Ui>> = const { RefCell::new(None) };
}

fn hiword_signed(v: u32) -> i32 {
    ((v >> 16) & 0xffff) as u16 as i16 as i32
}

/// Touches multimédia sans scancode (certains claviers / logiciels).
fn hid_from_vk(vk: u32) -> Option<u32> {
    let vk = vk as u16;
    Some(match vk {
        VK_VOLUME_MUTE => HID_VOLUME_MUTE,
        VK_VOLUME_DOWN => HID_VOLUME_DOWN,
        VK_VOLUME_UP => HID_VOLUME_UP,
        VK_MEDIA_NEXT_TRACK => HID_MEDIA_NEXT,
        VK_MEDIA_PREV_TRACK => HID_MEDIA_PREVIOUS,
        VK_MEDIA_STOP => HID_MEDIA_STOP,
        VK_MEDIA_PLAY_PAUSE => HID_MEDIA_PLAY_PAUSE,
        VK_BROWSER_BACK => HID_BROWSER_BACK,
        VK_BROWSER_FORWARD => HID_BROWSER_FORWARD,
        _ => return None,
    })
}

impl PrimaryState {
    fn refresh_layout(&mut self) {
        if self.host.layout_dirty {
            self.host.layout = monitors();
            self.host.layout_dirty = false;
            log::info!("écrans : {:?}", self.host.layout.displays);
        }
    }

    fn on_mouse(&mut self, msg: u32, info: &MSLLHOOKSTRUCT) -> Decision {
        self.refresh_layout();
        let remote = self.core.is_remote();
        let other = if remote { Decision::Swallow } else { Decision::Pass };
        let pt = Point::new(info.pt.x, info.pt.y);
        let host = &mut self.host;
        match msg {
            WM_MOUSEMOVE => {
                if remote {
                    self.core.remote_abs_motion(host, pt);
                    Decision::Swallow
                } else {
                    let cur = cursor_pos().unwrap_or(self.last_pt);
                    self.last_pt = pt;
                    self.core.local_motion(host, cur, pt)
                }
            }
            WM_LBUTTONDOWN => self.core.button(host, MouseButton::Left, true),
            WM_LBUTTONUP => self.core.button(host, MouseButton::Left, false),
            WM_RBUTTONDOWN => self.core.button(host, MouseButton::Right, true),
            WM_RBUTTONUP => self.core.button(host, MouseButton::Right, false),
            WM_MBUTTONDOWN => self.core.button(host, MouseButton::Middle, true),
            WM_MBUTTONUP => self.core.button(host, MouseButton::Middle, false),
            WM_XBUTTONDOWN | WM_XBUTTONUP => {
                let b = match (info.mouseData >> 16) & 0xffff {
                    1 => MouseButton::Back,
                    2 => MouseButton::Forward,
                    _ => return other,
                };
                self.core.button(host, b, msg == WM_XBUTTONDOWN)
            }
            WM_MOUSEWHEEL => self.core.wheel(host, 0, hiword_signed(info.mouseData)),
            WM_MOUSEHWHEEL => self.core.wheel(host, hiword_signed(info.mouseData), 0),
            _ => other,
        }
    }

    fn on_key(&mut self, info: &KBDLLHOOKSTRUCT) -> Decision {
        let down = info.flags & LLKHF_UP == 0;
        let remote = self.core.is_remote();

        // Faux Ctrl gauche généré par Windows quand on appuie sur AltGr :
        // ce n'est pas une touche physique, il n'est jamais transmis.
        if info.vkCode == VK_LCONTROL as u32 && info.scanCode & FAKE_LCTRL_BIT != 0 {
            let local = if down {
                *self.fake_ctrl_local.get_or_insert(!remote)
            } else {
                self.fake_ctrl_local.take().unwrap_or(!remote)
            };
            return if local { Decision::Pass } else { Decision::Swallow };
        }

        let mut sc = (info.scanCode & 0xff) as u16;
        if info.flags & LLKHF_EXTENDED != 0 {
            sc |= 0xe000;
        }
        let hid = if info.scanCode == 0 {
            hid_from_vk(info.vkCode)
        } else {
            keymap::hid_from_win(sc).or_else(|| hid_from_vk(info.vkCode))
        };
        let Some(hid) = hid else {
            log::debug!("touche ignorée : vk=0x{:02x} scancode=0x{sc:04x}", info.vkCode);
            return if remote { Decision::Swallow } else { Decision::Pass };
        };

        // Repli si le marqueur du faux Ctrl n'était pas présent : un Ctrl
        // gauche suivi d'AltGr avec le même horodatage était un faux Ctrl.
        if hid == HID_ALT_RIGHT && down && remote {
            if self.lctrl_down_time.take() == Some(info.time) {
                log::debug!("faux Ctrl d'AltGr annulé");
                self.core.key(&mut self.host, HID_CONTROL_LEFT, false);
            }
        }
        if hid == HID_CONTROL_LEFT && down && remote {
            self.lctrl_down_time = Some(info.time);
        } else if hid != HID_ALT_RIGHT {
            self.lctrl_down_time = None;
        }

        self.core.key(&mut self.host, hid, down)
    }
}

unsafe extern "system" fn mouse_proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: pour WH_MOUSE_LL, lParam pointe vers un MSLLHOOKSTRUCT.
    if code == HC_ACTION as i32 && unsafe { &*(l as *const MSLLHOOKSTRUCT) }.dwExtraInfo != BASKUL_INPUT {
        // SAFETY: idem.
        let info = unsafe { &*(l as *const MSLLHOOKSTRUCT) };
        let d = PRIMARY.with(|p| match p.try_borrow_mut() {
            Ok(mut g) => match g.as_mut() {
                Some(s) => s.on_mouse(w as u32, info),
                None => Decision::Pass,
            },
            Err(_) => Decision::Pass,
        });
        if d == Decision::Swallow {
            return 1;
        }
    }
    // SAFETY: transmission normale au hook suivant.
    unsafe { CallNextHookEx(null_mut(), code, w, l) }
}

unsafe extern "system" fn kbd_proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: pour WH_KEYBOARD_LL, lParam pointe vers un KBDLLHOOKSTRUCT.
        let info = unsafe { &*(l as *const KBDLLHOOKSTRUCT) };
        let d = PRIMARY.with(|p| match p.try_borrow_mut() {
            Ok(mut g) => match g.as_mut() {
                Some(s) => s.on_key(info),
                None => Decision::Pass,
            },
            Err(_) => Decision::Pass,
        });
        if d == Decision::Swallow {
            return 1;
        }
    }
    // SAFETY: transmission normale au hook suivant.
    unsafe { CallNextHookEx(null_mut(), code, w, l) }
}

fn install_hooks() {
    CURSOR.with(|c| {
        let mut c = c.borrow_mut();
        // SAFETY: handles de hooks obtenus par SetWindowsHookExW ou nuls.
        unsafe {
            if !c.mouse_hook.is_null() {
                UnhookWindowsHookEx(c.mouse_hook);
            }
            if !c.kbd_hook.is_null() {
                UnhookWindowsHookEx(c.kbd_hook);
            }
            let hmod: HINSTANCE = GetModuleHandleW(null());
            c.mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), hmod, 0);
            c.kbd_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kbd_proc), hmod, 0);
        }
        if c.mouse_hook.is_null() || c.kbd_hook.is_null() {
            log::error!("installation des hooks clavier/souris impossible");
            status::set_warning(Some(i18n::t("status.captureFailed")));
        }
    });
}

fn uninstall_hooks() {
    CURSOR.with(|c| {
        let mut c = c.borrow_mut();
        // SAFETY: handles valides ou nuls.
        unsafe {
            if !c.mouse_hook.is_null() {
                UnhookWindowsHookEx(c.mouse_hook);
            }
            if !c.kbd_hook.is_null() {
                UnhookWindowsHookEx(c.kbd_hook);
            }
        }
        c.mouse_hook = null_mut();
        c.kbd_hook = null_mut();
    });
}

fn with_primary<R>(f: impl FnOnce(&mut PrimaryState) -> R) -> Option<R> {
    PRIMARY.with(|p| match p.try_borrow_mut() {
        Ok(mut g) => g.as_mut().map(f),
        Err(_) => None,
    })
}

fn is_remote() -> bool {
    with_primary(|s| s.core.is_remote()).unwrap_or(false)
}

/// Met une fenêtre au premier plan, même si une autre application l'a.
fn force_foreground(hwnd: HWND) {
    // SAFETY: handles de fenêtres ; AttachThreadInput est détaché juste après.
    unsafe {
        let fg = GetForegroundWindow();
        if fg == hwnd {
            return;
        }
        let me = GetCurrentThreadId();
        let other = if fg.is_null() { 0 } else { GetWindowThreadProcessId(fg, null_mut()) };
        if other != 0 && other != me {
            AttachThreadInput(other, me, 1);
            SetForegroundWindow(hwnd);
            AttachThreadInput(other, me, 0);
        } else {
            SetForegroundWindow(hwnd);
        }
    }
}

/// Le curseur part sur l'autre machine : recentrage et masquage.
fn do_hide() {
    let Some((layout, cur)) = with_primary(|s| {
        if s.core.is_remote() { Some((s.host.layout.clone(), cursor_pos().unwrap_or(s.last_pt))) } else { None }
    })
    .flatten() else {
        return;
    };
    let center = layout.nearest_display(cur).center();
    let hider = CURSOR.with(|c| c.borrow().hider);
    // SAFETY: déplacement du curseur.
    unsafe { SetCursorPos(center.x, center.y) };
    // Le point de référence des déplacements doit changer AVANT tout appel
    // susceptible de traiter des messages (et donc d'appeler les hooks) :
    // sinon un déplacement mesuré depuis le bord de l'écran serait envoyé.
    with_primary(|s| s.core.on_warped(center));
    CURSOR.with(|c| c.borrow_mut().hidden_at = Some(center));
    // SAFETY: appels Win32 sur nos propres fenêtres.
    unsafe {
        let fg = GetForegroundWindow();
        if fg != hider {
            CURSOR.with(|c| c.borrow_mut().saved_fg = fg);
        }
        SetWindowPos(hider, HWND_TOPMOST, center.x - 16, center.y - 16, 32, 32, SWP_SHOWWINDOW);
    }
    // Peut traiter d'autres messages (et des appels de hooks) : aucun emprunt
    // n'est conservé pendant ces appels.
    force_foreground(hider);
    // SAFETY: masquage du curseur au-dessus de notre fenêtre.
    unsafe {
        SetCapture(hider);
        let mut n = 0;
        while ShowCursor(0) >= 0 && n < 50 {
            n += 1;
        }
        SetCursor(null_mut());
    }
}

/// Le curseur revient : réaffichage à la position demandée.
fn do_show(pos: Point) {
    if is_remote() {
        // Une nouvelle bascule a eu lieu entre-temps.
        return;
    }
    let (hider, saved) = CURSOR.with(|c| {
        let mut c = c.borrow_mut();
        c.hidden_at = None;
        (c.hider, std::mem::replace(&mut c.saved_fg, null_mut()))
    });
    // SAFETY: appels Win32 sur nos fenêtres ou des handles vérifiés par IsWindow.
    unsafe {
        ReleaseCapture();
        let mut n = 0;
        while ShowCursor(1) < 0 && n < 50 {
            n += 1;
        }
        ShowWindow(hider, SW_HIDE);
        if !saved.is_null() && IsWindow(saved) != 0 {
            force_foreground(saved);
        }
        SetCursorPos(pos.x, pos.y);
    }
}

/// Vérifications périodiques en mode distant.
fn watchdog() {
    if !is_remote() {
        return;
    }
    let (hidden_at, hider) = CURSOR.with(|c| {
        let c = c.borrow();
        (c.hidden_at, c.hider)
    });
    let Some(center) = hidden_at else { return };
    // Si le curseur réel a bougé, les hooks ne reçoivent plus les
    // événements (Windows peut les retirer sans prévenir) : on revient ici.
    if let Some(p) = cursor_pos() {
        if (p.x - center.x).abs() + (p.y - center.y).abs() > 200 {
            log::warn!("les hooks d'entrée ne répondent plus : retour sur ce PC et réinstallation");
            with_primary(|s| s.core.reset(&mut s.host));
            install_hooks();
            return;
        }
    }
    // Garder notre fenêtre au premier plan (sinon une fenêtre lancée en
    // administrateur pourrait recevoir les frappes).
    // SAFETY: lecture du premier plan.
    let fg = unsafe { GetForegroundWindow() };
    if fg != hider {
        let due = CURSOR.with(|c| c.borrow().last_fg_fix.is_none_or(|t| t.elapsed() > Duration::from_secs(2)));
        if due {
            CURSOR.with(|c| c.borrow_mut().last_fg_fix = Some(Instant::now()));
            force_foreground(hider);
        }
    }
}

fn process_peer_events() {
    let done = with_primary(|s| {
        while let Ok(ev) = s.peer_rx.try_recv() {
            s.core.peer_event(&mut s.host, ev);
        }
    });
    if done.is_none() && PRIMARY.with(|p| p.try_borrow().map(|g| g.is_some()).unwrap_or(true)) {
        // Occupé (réentrance) : on réessaie un peu plus tard.
        post(WM_APP_NET, 0, 0);
    }
}

fn reset_primary(reason: &str) {
    if with_primary(|s| s.core.is_remote() || s.core.is_connected()).unwrap_or(false) {
        log::info!("{reason} : tout est relâché");
    }
    with_primary(|s| s.core.reset(&mut s.host));
}

// ---------------------------------------------------------------------------
// Machine pilotée : émulation
// ---------------------------------------------------------------------------

struct WinEmulator {
    keys_down: HashSet<u32>,
    buttons: [bool; 5],
    scroll: ScrollCfg,
}

// SAFETY: aucune donnée liée à un thread.
unsafe impl Send for WinEmulator {}

fn send_input(input: INPUT) {
    // SAFETY: tableau d'un élément, taille correcte.
    unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) };
}

fn mouse_input(dx: i32, dy: i32, data: i32, flags: u32) {
    mouse_input_tagged(dx, dy, data, flags, 0);
}

fn mouse_input_tagged(dx: i32, dy: i32, data: i32, flags: u32, extra: usize) {
    // SAFETY: INPUT est une structure C initialisable à zéro.
    let mut input: INPUT = unsafe { zeroed() };
    input.r#type = INPUT_MOUSE;
    input.Anonymous.mi = MOUSEINPUT { dx, dy, mouseData: data as _, dwFlags: flags, time: 0, dwExtraInfo: extra };
    send_input(input);
}

/// L'utilisateur est actif sur l'autre machine : l'écran reste allumé et
/// Windows ne compte pas d'inactivité (économiseur, verrouillage
/// automatique, statut « absent » de Teams, Discord…).
fn keep_active() {
    // SAFETY: appel sans précondition.
    unsafe { SetThreadExecutionState(ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED) };
    // Déplacement nul : remet à zéro le délai d'inactivité de Windows.
    mouse_input_tagged(0, 0, 0, MOUSEEVENTF_MOVE, BASKUL_INPUT);
}

/// Temps écoulé depuis la dernière entrée de Windows.
fn idle_time() -> Option<Duration> {
    let mut info = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    // SAFETY: structure de la taille annoncée.
    if unsafe { GetLastInputInfo(&mut info) } == 0 {
        return None;
    }
    // SAFETY: appel sans précondition.
    let now = unsafe { GetTickCount() };
    Some(Duration::from_millis(u64::from(now.wrapping_sub(info.dwTime))))
}

fn key_input(code: u32, down: bool) {
    let mut ki = KEYBDINPUT { wVk: 0, wScan: 0, dwFlags: 0, time: 0, dwExtraInfo: 0 };
    let vk = match code {
        HID_PAUSE => Some(VK_PAUSE),
        HID_NUM_LOCK => Some(VK_NUMLOCK),
        HID_VOLUME_MUTE => Some(VK_VOLUME_MUTE),
        HID_VOLUME_UP => Some(VK_VOLUME_UP),
        HID_VOLUME_DOWN => Some(VK_VOLUME_DOWN),
        HID_MEDIA_NEXT => Some(VK_MEDIA_NEXT_TRACK),
        HID_MEDIA_PREVIOUS => Some(VK_MEDIA_PREV_TRACK),
        HID_MEDIA_STOP => Some(VK_MEDIA_STOP),
        HID_MEDIA_PLAY_PAUSE => Some(VK_MEDIA_PLAY_PAUSE),
        _ => None,
    };
    if let Some(vk) = vk {
        ki.wVk = vk;
        if code == HID_NUM_LOCK || keymap::page(code) == PAGE_CONSUMER {
            ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }
    } else {
        let Some(sc) = keymap::win_from_hid(code) else {
            log::debug!("touche sans équivalent Windows : {}", keymap::name_from_hid(code));
            return;
        };
        ki.wScan = sc & 0xff;
        ki.dwFlags = KEYEVENTF_SCANCODE;
        if sc & 0xff00 == 0xe000 {
            ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }
    }
    if !down {
        ki.dwFlags |= KEYEVENTF_KEYUP;
    }
    // SAFETY: INPUT est une structure C initialisable à zéro.
    let mut input: INPUT = unsafe { zeroed() };
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous.ki = ki;
    send_input(input);
}

impl WinEmulator {
    /// Bouton physique tel quel (l'échange 4/5 a déjà été appliqué).
    fn raw_button(&mut self, button: MouseButton, down: bool) {
        self.buttons[button.index()] = down;
        let (flags, data) = match (button, down) {
            (MouseButton::Left, true) => (MOUSEEVENTF_LEFTDOWN, 0),
            (MouseButton::Left, false) => (MOUSEEVENTF_LEFTUP, 0),
            (MouseButton::Right, true) => (MOUSEEVENTF_RIGHTDOWN, 0),
            (MouseButton::Right, false) => (MOUSEEVENTF_RIGHTUP, 0),
            (MouseButton::Middle, true) => (MOUSEEVENTF_MIDDLEDOWN, 0),
            (MouseButton::Middle, false) => (MOUSEEVENTF_MIDDLEUP, 0),
            (MouseButton::Back, true) => (MOUSEEVENTF_XDOWN, 1),
            (MouseButton::Back, false) => (MOUSEEVENTF_XUP, 1),
            (MouseButton::Forward, true) => (MOUSEEVENTF_XDOWN, 2),
            (MouseButton::Forward, false) => (MOUSEEVENTF_XUP, 2),
        };
        mouse_input(0, 0, data, flags);
    }
}

impl Emulator for WinEmulator {
    fn layout(&mut self) -> Layout {
        monitors()
    }

    fn move_to(&mut self, p: Point, _dx: i32, _dy: i32) {
        // SAFETY: lecture de métriques système.
        let (vx, vy, vw, vh) = unsafe {
            (
                GetSystemMetrics(SM_XVIRTUALSCREEN),
                GetSystemMetrics(SM_YVIRTUALSCREEN),
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            )
        };
        let nx = ((p.x - vx) as i64 * 65535 / (vw - 1).max(1) as i64) as i32;
        let ny = ((p.y - vy) as i64 * 65535 / (vh - 1).max(1) as i64) as i32;
        mouse_input(nx, ny, 0, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK);
    }

    fn button(&mut self, button: MouseButton, down: bool) {
        self.raw_button(button, down);
    }

    fn wheel(&mut self, dx: i32, dy: i32) {
        let dy = if self.scroll.invert_vertical { -dy } else { dy };
        let dx = if self.scroll.invert_horizontal { -dx } else { dx };
        if dy != 0 {
            mouse_input(0, 0, dy, MOUSEEVENTF_WHEEL);
        }
        if dx != 0 {
            mouse_input(0, 0, dx, MOUSEEVENTF_HWHEEL);
        }
    }

    fn key(&mut self, code: u32, down: bool, _repeat: bool) {
        if down {
            self.keys_down.insert(code);
        } else {
            self.keys_down.remove(&code);
        }
        key_input(code, down);
    }

    fn release_all(&mut self) {
        for code in std::mem::take(&mut self.keys_down) {
            key_input(code, false);
        }
        for b in MouseButton::ALL {
            if self.buttons[b.index()] {
                self.raw_button(b, false);
            }
        }
    }

    fn arrived(&mut self) {
        request_bubble();
    }

    fn cursor(&mut self) -> Option<Point> {
        cursor_pos()
    }

    fn keep_active(&mut self) {
        keep_active();
    }
}

// ---------------------------------------------------------------------------
// Icône de notification et menu
// ---------------------------------------------------------------------------

struct Ui {
    hwnd: HWND,
    tray_added: bool,
    icons: HashMap<(u8, bool, Style), HICON>,
    tray_icon: TrayIcon,
    config_path: PathBuf,
    args: Args,
    role: Role,
}

fn make_icon(size: i32, rgba: &[u8]) -> HICON {
    // SAFETY: création de bitmaps GDI de la taille demandée, puis copie de
    // `size*size` pixels dans le tampon alloué par CreateDIBSection.
    unsafe {
        let mut bi: BITMAPV5HEADER = zeroed();
        bi.bV5Size = size_of::<BITMAPV5HEADER>() as u32;
        bi.bV5Width = size;
        bi.bV5Height = -size;
        bi.bV5Planes = 1;
        bi.bV5BitCount = 32;
        bi.bV5Compression = BI_BITFIELDS;
        bi.bV5RedMask = 0x00ff_0000;
        bi.bV5GreenMask = 0x0000_ff00;
        bi.bV5BlueMask = 0x0000_00ff;
        bi.bV5AlphaMask = 0xff00_0000;
        let hdc = GetDC(null_mut());
        let mut bits: *mut c_void = null_mut();
        let color = CreateDIBSection(
            hdc,
            &bi as *const BITMAPV5HEADER as *const BITMAPINFO,
            DIB_RGB_COLORS,
            &mut bits,
            null_mut(),
            0,
        );
        ReleaseDC(null_mut(), hdc);
        if color.is_null() || bits.is_null() {
            return LoadIconW(null_mut(), IDI_APPLICATION);
        }
        let n = (size * size) as usize;
        let dst = std::slice::from_raw_parts_mut(bits as *mut u8, n * 4);
        for i in 0..n {
            dst[i * 4] = rgba[i * 4 + 2];
            dst[i * 4 + 1] = rgba[i * 4 + 1];
            dst[i * 4 + 2] = rgba[i * 4];
            dst[i * 4 + 3] = rgba[i * 4 + 3];
        }
        let mask = CreateBitmap(size, size, 1, 1, null());
        let ii = ICONINFO { fIcon: 1, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
        let icon = CreateIconIndirect(&ii);
        DeleteObject(color as _);
        DeleteObject(mask as _);
        if icon.is_null() { LoadIconW(null_mut(), IDI_APPLICATION) } else { icon }
    }
}

/// La barre des tâches est claire (thème Windows).
fn light_taskbar() -> bool {
    let k = wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let v = wide("SystemUsesLightTheme");
    let mut value = 0u32;
    let mut len = size_of::<u32>() as u32;
    // SAFETY: chaînes terminées par un zéro, tampon de la taille indiquée.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut value as *mut u32 as *mut c_void,
            &mut len,
        )
    };
    r == ERROR_SUCCESS && value != 0
}

impl Ui {
    fn icon(&mut self, tone: Tone, remote: bool) -> HICON {
        let style = match self.tray_icon {
            TrayIcon::Color => Style::Color,
            // Monochrome : noir sur une barre des tâches claire, blanc sinon.
            TrayIcon::Mono if light_taskbar() => Style::Mono([0, 0, 0]),
            TrayIcon::Mono => Style::Mono([255, 255, 255]),
        };
        let key = (tone as u8, remote, style);
        if let Some(i) = self.icons.get(&key) {
            return *i;
        }
        // SAFETY: lecture de métrique système.
        let size = unsafe { GetSystemMetrics(SM_CXSMICON) }.clamp(16, 64);
        let h = make_icon(size, &icon::render(size as u32, tone, remote, style));
        self.icons.insert(key, h);
        h
    }

    fn update_tray(&mut self) {
        let snap = status::snapshot();
        let tone = Tone::of(&snap.conn);
        let hicon = self.icon(tone, snap.remote_active);
        // SAFETY: structure initialisable à zéro.
        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
        nid.uCallbackMessage = WM_APP_TRAY;
        nid.hIcon = hicon;
        let tip: Vec<u16> = format!("Baskul — {}", snap.summary()).encode_utf16().take(127).collect();
        nid.szTip[..tip.len()].copy_from_slice(&tip);
        // SAFETY: structure correctement remplie.
        unsafe {
            if self.tray_added {
                if Shell_NotifyIconW(NIM_MODIFY, &nid) == 0 {
                    self.tray_added = Shell_NotifyIconW(NIM_ADD, &nid) != 0;
                }
            } else {
                self.tray_added = Shell_NotifyIconW(NIM_ADD, &nid) != 0;
            }
        }
    }

    fn remove_tray(&mut self) {
        if !self.tray_added {
            return;
        }
        // SAFETY: structure initialisable à zéro.
        let mut nid: NOTIFYICONDATAW = unsafe { zeroed() };
        nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        // SAFETY: structure correctement remplie.
        unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };
        self.tray_added = false;
    }
}

fn with_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    UI.with(|u| match u.try_borrow_mut() {
        Ok(mut g) => g.as_mut().map(f),
        Err(_) => None,
    })
}

fn autostart_command(args: &Args) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let mut s = format!("\"{}\"", exe.display());
    for a in args.to_vec() {
        if a == "--console" {
            continue;
        }
        s.push_str(&format!(" \"{a}\""));
    }
    Some(s)
}

fn autostart_enabled() -> bool {
    let k = wide(RUN_KEY);
    let v = wide(RUN_VALUE);
    // SAFETY: chaînes terminées par un zéro ; on ne lit que l'existence.
    unsafe {
        RegGetValueW(HKEY_CURRENT_USER, k.as_ptr(), v.as_ptr(), RRF_RT_REG_SZ, null_mut(), null_mut(), null_mut())
            == ERROR_SUCCESS
    }
}

/// Ancien nom (Bascule) : son lancement au démarrage passe à Baskul.
fn migrate_old_autostart(args: &Args) {
    let k = wide(RUN_KEY);
    let v = wide(crate::config::OLD_NAME);
    // SAFETY: chaînes terminées par un zéro.
    if unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, k.as_ptr(), v.as_ptr()) } == ERROR_SUCCESS {
        log::info!("lancement au démarrage repris de {}", crate::config::OLD_NAME);
        set_autostart(true, args);
    }
}

fn set_autostart(on: bool, args: &Args) {
    let k = wide(RUN_KEY);
    let v = wide(RUN_VALUE);
    // SAFETY: chaînes terminées par un zéro, taille en octets correcte.
    let r = unsafe {
        if on {
            let Some(cmd) = autostart_command(args) else { return };
            let data = wide(&cmd);
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                k.as_ptr(),
                v.as_ptr(),
                REG_SZ,
                data.as_ptr() as *const c_void,
                (data.len() * 2) as u32,
            )
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, k.as_ptr(), v.as_ptr())
        }
    };
    if r == ERROR_SUCCESS {
        log::info!("lancement au démarrage {}", if on { "activé" } else { "désactivé" });
    } else {
        log::warn!("modification du lancement au démarrage impossible (code {r})");
    }
}

fn append(menu: HMENU, flags: MENU_ITEM_FLAGS, id: usize, text: &str) {
    let t = wide(text);
    // SAFETY: menu valide, chaîne terminée par un zéro.
    unsafe { AppendMenuW(menu, flags, id, t.as_ptr()) };
}

fn show_menu(hwnd: HWND) {
    let Some((role, snap)) = with_ui(|ui| (ui.role, status::snapshot())) else { return };
    let locked = with_primary(|s| s.core.is_locked()).unwrap_or(false);
    let names: Vec<String> = snap.peers.iter().map(|p| p.name.clone()).collect();
    // SAFETY: création et destruction d'un menu local.
    unsafe {
        let menu = CreatePopupMenu();
        append(menu, MF_STRING | MF_GRAYED, 0, &snap.summary());
        if let Some(v) = crate::update::ready_version() {
            append(menu, MF_STRING, ID_UPDATE, &i18n::tf("menu.updateReady", &[("version", &v)]));
        }
        append(menu, MF_SEPARATOR, 0, "");
        if role == Role::Primary {
            append(menu, MF_STRING, ID_RETURN, &i18n::t("menu.returnHere"));
            for (i, name) in names.iter().enumerate() {
                let on = if snap.active_peer.as_deref() == Some(name.as_str()) { MF_CHECKED } else { 0 };
                append(menu, MF_STRING | on, ID_GOTO + i, &i18n::tf("menu.goTo", &[("name", name)]));
            }
            append(menu, MF_STRING | if locked { MF_CHECKED } else { 0 }, ID_LOCK, &i18n::t("menu.lock"));
            append(menu, MF_SEPARATOR, 0, "");
        }
        append(menu, MF_STRING, ID_SETTINGS, &i18n::t("menu.settings"));
        append(menu, MF_STRING, ID_LOG, &i18n::t("menu.openLog"));
        append(menu, MF_STRING, ID_FOLDER, &i18n::t("menu.openFolder"));
        append(menu, MF_SEPARATOR, 0, "");
        let auto = if autostart_enabled() { MF_CHECKED } else { 0 };
        append(menu, MF_STRING | auto, ID_AUTOSTART, &i18n::t("menu.startWithWindows"));
        append(menu, MF_SEPARATOR, 0, "");
        append(menu, MF_STRING, ID_RESTART, &i18n::t("menu.restart"));
        append(menu, MF_STRING, ID_QUIT, &i18n::t("menu.quit"));
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY, pt.x, pt.y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
        match (cmd as usize).checked_sub(ID_GOTO).and_then(|i| names.get(i)) {
            Some(name) => {
                with_primary(|s| s.core.switch_to(&mut s.host, None, SwitchTo::Client(name.clone())));
            }
            None => handle_command(cmd as usize),
        }
    }
}

fn handle_command(cmd: usize) {
    let Some((path, args)) = with_ui(|ui| (ui.config_path.clone(), ui.args.clone())) else { return };
    match cmd {
        ID_RETURN => {
            with_primary(|s| s.core.return_home(&mut s.host));
        }
        ID_LOCK => {
            with_primary(|s| s.core.toggle_lock());
        }
        ID_SETTINGS => super::open_settings(&path, false),
        ID_LOG => {
            if let Some(p) = crate::logging::log_path() {
                crate::logging::flush();
                shell_open("notepad.exe", Some(&format!("\"{}\"", p.display())));
            }
        }
        ID_FOLDER => {
            if let Some(dir) = path.parent() {
                shell_open(&dir.display().to_string(), None);
            }
        }
        ID_AUTOSTART => set_autostart(!autostart_enabled(), &args),
        ID_UPDATE => {
            if let Err(e) = crate::update::install(None) {
                log::warn!("{e}");
            }
        }
        ID_RESTART => {
            shutdown();
            super::relaunch(&args);
        }
        ID_QUIT => {
            shutdown();
            log::info!("arrêt demandé");
            crate::logging::flush();
            std::process::exit(0);
        }
        _ => {}
    }
}

/// Remet le système dans un état propre avant de quitter.
fn shutdown() {
    with_primary(|s| s.core.reset(&mut s.host));
    // Le message « tout relâcher » part avant la fermeture.
    std::thread::sleep(Duration::from_millis(150));
    // SAFETY: restauration de l'état du curseur.
    unsafe {
        ReleaseCapture();
        let mut n = 0;
        while ShowCursor(1) < 0 && n < 50 {
            n += 1;
        }
    }
    uninstall_hooks();
    with_ui(|ui| ui.remove_tray());
}

// ---------------------------------------------------------------------------
// Fenêtres et boucle principale
// ---------------------------------------------------------------------------

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_APP_NET => {
            process_peer_events();
            0
        }
        WM_APP_STATUS => {
            with_ui(|ui| ui.update_tray());
            0
        }
        WM_APP_HIDE => {
            do_hide();
            0
        }
        WM_APP_SHOW => {
            do_show(Point::new(w as isize as i32, l as i32));
            0
        }
        WM_APP_RESTART => {
            handle_command(ID_RESTART);
            0
        }
        WM_APP_ACTIVE => {
            keep_active();
            0
        }
        WM_APP_BUBBLE => {
            show_bubble();
            0
        }
        WM_APP_TRAY => {
            let ev = (l as u32) & 0xffff;
            if ev == WM_RBUTTONUP || ev == WM_LBUTTONUP || ev == WM_CONTEXTMENU {
                show_menu(hwnd);
            }
            0
        }
        WM_TIMER => {
            match w {
                TIMER_WATCHDOG => watchdog(),
                TIMER_BUBBLE => bubble_tick(),
                TIMER_REHOOK => {
                    // Windows peut retirer un hook sans prévenir : on le
                    // réinstalle régulièrement (jamais pendant le pilotage).
                    if PRIMARY.with(|p| p.try_borrow().map(|g| g.is_some()).unwrap_or(false)) && !is_remote() {
                        install_hooks();
                    }
                }
                TIMER_TRAY => {
                    let added = with_ui(|ui| {
                        ui.update_tray();
                        ui.tray_added
                    })
                    .unwrap_or(true);
                    if added {
                        // SAFETY: notre fenêtre.
                        unsafe { KillTimer(hwnd, TIMER_TRAY) };
                    }
                }
                _ => {}
            }
            0
        }
        // Thème de Windows changé : l'icône monochrome suit la barre des tâches.
        WM_SETTINGCHANGE => {
            with_ui(|ui| ui.update_tray());
            // SAFETY: traitement par défaut.
            unsafe { DefWindowProcW(hwnd, msg, w, l) }
        }
        WM_DISPLAYCHANGE => {
            with_primary(|s| s.host.layout_dirty = true);
            // SAFETY: traitement par défaut.
            unsafe { DefWindowProcW(hwnd, msg, w, l) }
        }
        WM_WTSSESSION_CHANGE => {
            if w as u32 == WTS_SESSION_LOCK {
                reset_primary("session verrouillée");
            }
            0
        }
        WM_POWERBROADCAST => {
            if w as u32 == PBT_APMSUSPEND {
                reset_primary("mise en veille");
            }
            1
        }
        WM_QUERYENDSESSION => 1,
        WM_ENDSESSION => {
            if w != 0 {
                shutdown();
            }
            0
        }
        WM_DESTROY => {
            // SAFETY: fin de la boucle de messages.
            unsafe { PostQuitMessage(0) };
            0
        }
        m if m != 0 && m == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            // L'Explorateur a redémarré : il faut recréer l'icône.
            with_ui(|ui| {
                ui.tray_added = false;
                ui.update_tray();
            });
            0
        }
        // SAFETY: traitement par défaut.
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
}

// ---------------------------------------------------------------------------
// Bulle d'arrivée
// ---------------------------------------------------------------------------

/// Nom affiché par la bulle (absent : bulle désactivée).
static BUBBLE_NAME: OnceLock<String> = OnceLock::new();
/// Durée d'affichage avant le fondu, puis durée du fondu.
const BUBBLE_SHOW: Duration = Duration::from_millis(900);
const BUBBLE_FADE: Duration = Duration::from_millis(350);

thread_local! {
    /// Fenêtre de la bulle (créée au premier affichage) et heure d'affichage.
    static BUBBLE: RefCell<(HWND, Option<Instant>)> = const { RefCell::new((null_mut(), None)) };
}

/// Le clavier et la souris arrivent sur cette machine : son nom s'affiche un
/// instant près du curseur. Utilisable depuis n'importe quel thread.
fn request_bubble() {
    if BUBBLE_NAME.get().is_some() {
        post(WM_APP_BUBBLE, 0, 0);
    }
}

unsafe extern "system" fn bubble_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_NCHITTEST {
        return HTTRANSPARENT as LRESULT;
    }
    // SAFETY: traitement par défaut.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

/// Fenêtre de la bulle : transparente aux clics, sans focus, au premier plan,
/// absente de la barre des tâches.
fn bubble_window() -> HWND {
    let existing = BUBBLE.with(|b| b.borrow().0);
    if !existing.is_null() {
        return existing;
    }
    let class = wide("BaskulBubble");
    let empty = wide("");
    // SAFETY: création d'une fenêtre sur le thread de l'interface.
    let hwnd = unsafe {
        let hinst: HINSTANCE = GetModuleHandleW(null());
        register_class(&class, Some(bubble_proc), hinst, false);
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class.as_ptr(),
            empty.as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        )
    };
    BUBBLE.with(|b| b.borrow_mut().0 = hwnd);
    hwnd
}

/// Dessine la bulle (pastille sombre, logo Windows, nom de la machine) près
/// du curseur, à l'échelle de l'écran où il se trouve.
fn show_bubble() {
    let Some(name) = BUBBLE_NAME.get() else { return };
    let Some(cur) = cursor_pos() else { return };
    let hwnd = bubble_window();
    if hwnd.is_null() {
        return;
    }
    let text: Vec<u16> = name.encode_utf16().collect();
    // SAFETY: objets GDI créés, utilisés puis libérés dans ce bloc ; pixels de
    // la section DIB de la taille demandée.
    unsafe {
        let pt = POINT { x: cur.x, y: cur.y };
        let monitor = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let (mut dpi, mut dpi_y) = (96u32, 96u32);
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi, &mut dpi_y);
        let px = |v: f32| (v * dpi as f32 / 96.0).round() as i32;
        let mut info: MONITORINFO = zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        GetMonitorInfoW(monitor, &mut info);
        let work = info.rcWork;

        let face = wide("Segoe UI Semibold");
        let font = CreateFontW(
            -px(15.0),
            0,
            0,
            0,
            FW_SEMIBOLD as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            ANTIALIASED_QUALITY as u32,
            DEFAULT_PITCH as u32,
            face.as_ptr(),
        );
        let screen = GetDC(null_mut());
        let mem = CreateCompatibleDC(screen);
        let old_font = SelectObject(mem, font);
        let mut extent = SIZE { cx: 0, cy: 0 };
        GetTextExtentPoint32W(mem, text.as_ptr(), text.len() as i32, &mut extent);
        let (pad, logo, gap) = (px(14.0), px(13.0), px(9.0));
        let h = extent.cy + px(14.0);
        let w = pad + logo + gap + extent.cx + pad;

        let mut bi: BITMAPINFO = zeroed();
        bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = w;
        bi.bmiHeader.biHeight = -h;
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;
        let mut bits: *mut c_void = null_mut();
        let dib = CreateDIBSection(mem, &bi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
        if dib.is_null() || bits.is_null() {
            SelectObject(mem, old_font);
            DeleteObject(font);
            DeleteDC(mem);
            ReleaseDC(null_mut(), screen);
            return;
        }
        let old_bmp = SelectObject(mem, dib);
        // Texte blanc sur fond noir : sa luminosité donne sa couverture.
        SetBkMode(mem, TRANSPARENT as i32);
        SetTextColor(mem, 0x00ff_ffff);
        TextOutW(mem, pad + logo + gap, (h - extent.cy) / 2, text.as_ptr(), text.len() as i32);
        GdiFlush();
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u8, (w * h * 4) as usize);
        paint_bubble(pixels, w, h, pad, logo);

        // En bas à droite du curseur, sans sortir de l'écran.
        let x = (cur.x + px(18.0)).min(work.right - w - px(6.0)).max(work.left + px(6.0));
        let y = (cur.y + px(22.0)).min(work.bottom - h - px(6.0)).max(work.top + px(6.0));
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        UpdateLayeredWindow(
            hwnd,
            screen,
            &POINT { x, y },
            &SIZE { cx: w, cy: h },
            mem,
            &POINT { x: 0, y: 0 },
            0,
            &blend,
            ULW_ALPHA,
        );
        SelectObject(mem, old_bmp);
        SelectObject(mem, old_font);
        DeleteObject(dib);
        DeleteObject(font);
        DeleteDC(mem);
        ReleaseDC(null_mut(), screen);
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW);
        SetTimer(MAIN_HWND.load(Ordering::Acquire), TIMER_BUBBLE, 30, None);
    }
    BUBBLE.with(|b| b.borrow_mut().1 = Some(Instant::now()));
}

/// Compose la bulle (pixels BGRA prémultipliés) : pastille aux bords
/// arrondis, logo Windows et texte déjà dessiné en blanc sur noir.
fn paint_bubble(pixels: &mut [u8], w: i32, h: i32, pad: i32, logo: i32) {
    const BG: [f32; 3] = [32.0, 32.0, 32.0];
    const BG_ALPHA: f32 = 0.88;
    const LOGO: [f32; 3] = [255.0, 180.0, 80.0]; // bleu (BGR)
    let r = h as f32 / 2.0;
    let logo_y = (h - logo) / 2;
    let tile_gap = (logo / 12).max(1);
    let tile = (logo - tile_gap) / 2;
    let in_tile = |v: i32| (0..tile).contains(&v) || (tile + tile_gap..2 * tile + tile_gap).contains(&v);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            // Pastille : rectangle aux extrémités en demi-cercle, bords lissés.
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let cx = fx.clamp(r, w as f32 - r);
            let d = ((fx - cx).powi(2) + (fy - r).powi(2)).sqrt();
            let shape = (r - d + 0.5).clamp(0.0, 1.0);
            let text = pixels[i].max(pixels[i + 1]).max(pixels[i + 2]) as f32 / 255.0;
            let on_logo = in_tile(x - pad) && in_tile(y - logo_y);
            let (fg, cover) = if on_logo { (LOGO, 1.0) } else { ([255.0; 3], text) };
            let alpha = shape * (BG_ALPHA + (1.0 - BG_ALPHA) * cover);
            for c in 0..3 {
                // Premier plan opaque sur le fond translucide, prémultiplié.
                let color = fg[c] * cover + BG[c] * BG_ALPHA * (1.0 - cover);
                pixels[i + c] = (shape * color).round().clamp(0.0, 255.0) as u8;
            }
            pixels[i + 3] = (alpha * 255.0).round() as u8;
        }
    }
}

/// Fondu, puis fenêtre masquée (une nouvelle arrivée repousse l'échéance).
fn bubble_tick() {
    let (hwnd, shown_at) = BUBBLE.with(|b| *b.borrow());
    let Some(elapsed) = shown_at.map(|t| t.elapsed()) else { return };
    if elapsed < BUBBLE_SHOW {
        return;
    }
    // SAFETY: fenêtre de la bulle et minuterie créées par ce thread.
    unsafe {
        if elapsed >= BUBBLE_SHOW + BUBBLE_FADE {
            ShowWindow(hwnd, SW_HIDE);
            KillTimer(MAIN_HWND.load(Ordering::Acquire), TIMER_BUBBLE);
            BUBBLE.with(|b| b.borrow_mut().1 = None);
            return;
        }
        let left = 1.0 - (elapsed - BUBBLE_SHOW).as_secs_f32() / BUBBLE_FADE.as_secs_f32();
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: (left * 255.0) as u8,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        UpdateLayeredWindow(hwnd, null_mut(), null(), null(), null_mut(), null(), 0, &blend, ULW_ALPHA);
    }
}

unsafe extern "system" fn hider_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_SETCURSOR {
        // SAFETY: curseur invisible au-dessus de notre fenêtre.
        unsafe { SetCursor(null_mut()) };
        return 1;
    }
    // SAFETY: traitement par défaut.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

fn register_class(name: &[u16], proc_: WNDPROC, hinst: HINSTANCE, with_cursor: bool) {
    // SAFETY: structure initialisable à zéro ; nom terminé par un zéro et
    // conservé pendant toute la durée du programme (constante).
    unsafe {
        let mut wc: WNDCLASSEXW = zeroed();
        wc.cbSize = size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = proc_;
        wc.hInstance = hinst;
        wc.lpszClassName = name.as_ptr();
        if with_cursor {
            wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        }
        RegisterClassExW(&wc);
    }
}

pub fn run(settings: Settings, config_path: PathBuf, args: Args) -> ! {
    migrate_old_autostart(&args);
    let main_class = wide("BaskulMain");
    let hider_class = wide("BaskulHider");
    let title = wide("Baskul");
    let empty = wide("");
    // SAFETY: création de nos fenêtres sur ce thread, qui exécute ensuite la
    // boucle de messages.
    let (hwnd, hider) = unsafe {
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
        let hinst: HINSTANCE = GetModuleHandleW(null());
        register_class(&main_class, Some(wndproc), hinst, true);
        register_class(&hider_class, Some(hider_proc), hinst, false);
        let hwnd = CreateWindowExW(
            0,
            main_class.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        let hider = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            hider_class.as_ptr(),
            empty.as_ptr(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        if hwnd.is_null() || hider.is_null() {
            fatal("impossible de créer les fenêtres de Baskul");
        }
        // Presque transparente (une opacité nulle la rendrait « traversable »).
        SetLayeredWindowAttributes(hider, 0, 1, LWA_ALPHA);
        let tc = wide("TaskbarCreated");
        TASKBAR_CREATED.store(RegisterWindowMessageW(tc.as_ptr()), Ordering::Relaxed);
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
        (hwnd, hider)
    };
    MAIN_HWND.store(hwnd, Ordering::Release);
    CURSOR.with(|c| c.borrow_mut().hider = hider);
    if settings.ui.arrival_bubble {
        let _ = BUBBLE_NAME.set(settings.name.clone());
    }
    status::on_change(|| post(WM_APP_STATUS, 0, 0));
    crate::api::set_restart_handler(|| post(WM_APP_RESTART, 0, 0));
    UI.with(|u| {
        *u.borrow_mut() = Some(Ui {
            hwnd,
            tray_added: false,
            icons: HashMap::new(),
            tray_icon: settings.ui.tray_icon,
            config_path: config_path.clone(),
            args,
            role: settings.role,
        })
    });

    let hooks = ClipboardHooks {
        sequence: clip_sequence,
        sensitive: clip_sensitive,
        read_images: clip_read_images,
        write: clip_write,
        formats: clip_formats,
    };
    let _services: Services = match settings.role {
        Role::Primary => {
            let (services, rx) = app::start_primary(&settings, hooks, || post(WM_APP_NET, 0, 0));
            let layout = monitors();
            log::info!("écrans : {:?}", layout.displays);
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
            PRIMARY.with(|p| {
                *p.borrow_mut() = Some(PrimaryState {
                    core: PrimaryCore::new(opts),
                    host: WinHost {
                        layout,
                        layout_dirty: false,
                        link: services.link.clone(),
                        config_path: config_path.clone(),
                    },
                    peer_rx: rx,
                    fake_ctrl_local: None,
                    lctrl_down_time: None,
                    last_pt: cursor_pos().unwrap_or_default(),
                })
            });
            install_hooks();
            // SAFETY: minuteries sur notre fenêtre.
            unsafe {
                SetTimer(hwnd, TIMER_WATCHDOG, 500, None);
                SetTimer(hwnd, TIMER_REHOOK, 60_000, None);
            }
            services
        }
        Role::Secondary => {
            let emu = WinEmulator { keys_down: HashSet::new(), buttons: [false; 5], scroll: settings.scroll.clone() };
            app::start_secondary(&settings, hooks, emu, idle_time)
        }
    };

    with_ui(|ui| ui.update_tray());
    // SAFETY: minuterie sur notre fenêtre (réessai de création de l'icône si
    // l'Explorateur n'est pas encore prêt, au démarrage de session).
    unsafe { SetTimer(hwnd, TIMER_TRAY, 2000, None) };

    // SAFETY: boucle de messages standard.
    unsafe {
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    shutdown();
    crate::logging::flush();
    std::process::exit(0)
}
