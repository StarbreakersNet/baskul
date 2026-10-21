//! Code propre à chaque système.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(any(windows, all(feature = "xcheck", target_os = "linux")))]
pub mod windows;

#[cfg(any(target_os = "macos", all(feature = "xcheck", target_os = "linux")))]
pub mod macos;

#[cfg(not(any(windows, target_os = "macos")))]
mod linux;

#[cfg(windows)]
pub use self::windows::{early_init, fatal, first_run, open_in_editor, run};

#[cfg(target_os = "macos")]
pub use self::macos::{early_init, fatal, first_run, open_in_editor, run};

#[cfg(not(any(windows, target_os = "macos")))]
pub use self::linux::{early_init, fatal, first_run, open_in_editor, run};

/// Variable d'environnement posée lors d'un redémarrage : la nouvelle
/// instance attend alors la fin de l'ancienne.
pub const RELAUNCH_ENV: &str = "BASCULE_RELAUNCH";

static RESTART_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Demande de redémarrage, traitée par la boucle principale avec
/// [`take_restart_request`] (macOS, Linux, configuration invalide).
pub fn request_restart() {
    RESTART_REQUESTED.store(true, Ordering::Release);
}

pub fn take_restart_request() -> bool {
    RESTART_REQUESTED.swap(false, Ordering::AcqRel)
}

/// Ouvre l'app de réglages ; à défaut, le fichier de configuration dans
/// l'éditeur de texte. `welcome` : premier lancement.
pub fn open_settings(path: &Path, welcome: bool) {
    match crate::api::launch_settings_app(path, welcome) {
        Ok(mut child) => {
            // On attend sa fin pour ne pas laisser de processus zombie.
            let _ = std::thread::Builder::new().name("bascule-settings-wait".into()).spawn(move || child.wait());
        }
        Err(e) => {
            log::warn!("{e} : ouverture du fichier de configuration");
            open_in_editor(path);
        }
    }
}

/// Relance Bascule avec les mêmes arguments puis quitte ce processus.
#[cfg(not(target_os = "macos"))]
pub fn relaunch(args: &crate::Args) -> ! {
    match std::env::current_exe() {
        Ok(exe) => match std::process::Command::new(&exe).args(args.to_vec()).env(RELAUNCH_ENV, "1").spawn() {
            Ok(_) => log::info!("redémarrage…"),
            Err(e) => log::error!("redémarrage impossible : {e}"),
        },
        Err(e) => log::error!("redémarrage impossible : {e}"),
    }
    crate::logging::flush();
    std::process::exit(0)
}

/// Relance Bascule (même processus : les autorisations macOS sont conservées).
#[cfg(target_os = "macos")]
pub fn relaunch(args: &crate::Args) -> ! {
    use std::os::unix::process::CommandExt;
    crate::logging::flush();
    match std::env::current_exe() {
        Ok(exe) => {
            let err = std::process::Command::new(&exe).args(args.to_vec()).env(RELAUNCH_ENV, "1").exec();
            log::error!("redémarrage impossible : {err}");
        }
        Err(e) => log::error!("redémarrage impossible : {e}"),
    }
    crate::logging::flush();
    std::process::exit(1)
}
