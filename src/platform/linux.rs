//! Linux (et autres systèmes) : version minimale, sans partage du clavier et
//! de la souris. Connexion, état, presse-papiers et réglages fonctionnent ;
//! utile pour tester l'app de réglages et la liaison avec l'autre machine.

use crate::Args;
use crate::app::{self, ClipboardHooks};
use crate::config::Settings;
use crate::geometry::{Layout, Point, Rect};
use crate::protocol::{MouseButton, Role};
use crate::secondary::Emulator;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn early_init(_args: &Args) {}

pub fn fatal(msg: &str) -> ! {
    log::error!("{msg}");
    crate::logging::flush();
    eprintln!("Bascule : {msg}");
    std::process::exit(1)
}

pub fn first_run(path: &Path) {
    eprintln!("Configuration créée : {}", path.display());
    super::open_settings(path, true);
}

pub fn open_in_editor(path: &Path) {
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}

/// Machine pilotée sans injection : les événements reçus sont ignorés. Un
/// écran fictif permet au curseur d'entrer puis de repartir par le bord.
struct NullEmulator;

impl Emulator for NullEmulator {
    fn layout(&mut self) -> Layout {
        Layout::new(vec![Rect::new(0, 0, 1920, 1080)])
    }
    fn move_to(&mut self, _p: Point, _dx: i32, _dy: i32) {}
    fn button(&mut self, _button: MouseButton, _down: bool) {}
    fn wheel(&mut self, _dx: i32, _dy: i32) {}
    fn key(&mut self, _code: u32, _down: bool, _repeat: bool) {}
    fn release_all(&mut self) {}
}

pub fn run(settings: Settings, _config_path: PathBuf, args: Args) -> ! {
    log::warn!("Linux : partage du clavier et de la souris non pris en charge (connexion, presse-papiers et réglages)");
    crate::api::set_restart_handler(super::request_restart);
    let hooks = ClipboardHooks::NONE;
    let _services = match settings.role {
        Role::Primary => app::start_primary(&settings, hooks, || {}).0,
        Role::Secondary => app::start_secondary(&settings, hooks, NullEmulator, || None),
    };
    loop {
        if super::take_restart_request() {
            super::relaunch(&args);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
