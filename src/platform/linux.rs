//! Linux (et autres systèmes) : version minimale, sans partage du clavier et
//! de la souris. Connexion, état, presse-papiers et réglages fonctionnent ;
//! utile pour tester les réglages et la liaison avec l'autre machine. L'icône
//! est dessinée par l'app (Tauri), à partir de `crate::tray`.

use crate::Args;
use crate::config::Settings;
use crate::geometry::{Layout, Point, Rect};
use crate::icon::Style;
use crate::protocol::{MouseButton, Role};
use crate::secondary::Emulator;
use crate::services::{self, ClipboardHooks};
use crate::tray::{self, Action, IconStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn early_init(_args: &Args) {}

pub fn fatal(msg: &str) -> ! {
    log::error!("{msg}");
    crate::logging::flush();
    eprintln!("Baskul : {msg}");
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

/// Ce que l'icône (dessinée par l'app) demande au système.
struct LinuxTray {
    role: Role,
    config_path: PathBuf,
    args: Args,
}

impl tray::Backend for LinuxTray {
    fn role(&self) -> Role {
        self.role
    }

    fn icon_style(&self) -> IconStyle {
        IconStyle { style: Style::Color, size: 32, template: false }
    }

    fn login_item(&self) -> Option<(String, bool)> {
        None
    }

    fn perform(&self, action: Action) {
        match action {
            Action::Settings => {
                super::open_settings(&self.config_path, false);
            }
            Action::OpenLog => {
                if let Some(p) = crate::logging::log_path() {
                    crate::logging::flush();
                    open_in_editor(&p);
                }
            }
            Action::OpenFolder => {
                if let Some(dir) = self.config_path.parent() {
                    open_in_editor(dir);
                }
            }
            Action::InstallUpdate => {
                if let Err(e) = crate::update::install(None) {
                    log::warn!("{e}");
                }
            }
            Action::Restart => super::relaunch(&self.args),
            Action::Quit => {
                log::info!("arrêt demandé");
                crate::logging::flush();
                std::process::exit(0);
            }
            // Sans partage du clavier et de la souris.
            Action::ReturnHere
            | Action::GoTo(_)
            | Action::ToggleLock
            | Action::ToggleLogin
            | Action::AllowAccessibility => {}
        }
    }
}

/// L'icône de l'app peut s'afficher.
pub fn set_tray_backend(settings: &Settings, config_path: &Path, args: &Args) {
    tray::set_backend(LinuxTray { role: settings.role, config_path: config_path.to_path_buf(), args: args.clone() });
}

pub fn run(settings: Settings, _config_path: PathBuf, args: Args) -> ! {
    log::warn!("Linux : partage du clavier et de la souris non pris en charge (connexion, presse-papiers et réglages)");
    crate::api::set_restart_handler(super::request_restart);
    let hooks = ClipboardHooks::NONE;
    let _services = match settings.role {
        Role::Primary => services::start_primary(&settings, hooks, || {}).0,
        Role::Secondary => services::start_secondary(&settings, hooks, NullEmulator, || None),
    };
    loop {
        if super::take_restart_request() {
            super::relaunch(&args);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
