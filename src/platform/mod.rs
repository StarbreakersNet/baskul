//! Code propre à chaque système.

#[cfg(any(windows, all(feature = "xcheck", target_os = "linux")))]
pub mod windows;

#[cfg(any(target_os = "macos", all(feature = "xcheck", target_os = "linux")))]
pub mod macos;

#[cfg(windows)]
pub use self::windows::{early_init, fatal, first_run, open_in_editor, open_url, run};

#[cfg(target_os = "macos")]
pub use self::macos::{early_init, fatal, first_run, open_in_editor, open_url, run};

#[cfg(not(any(windows, target_os = "macos")))]
pub use self::unsupported::{early_init, fatal, first_run, open_in_editor, open_url, run};

#[cfg(not(any(windows, target_os = "macos")))]
mod unsupported {
    use crate::Args;
    use crate::config::Settings;
    use std::path::{Path, PathBuf};

    pub fn early_init(_args: &Args) {}

    pub fn fatal(msg: &str) -> ! {
        log::error!("{msg}");
        crate::logging::flush();
        eprintln!("Bascule : {msg}");
        std::process::exit(1)
    }

    pub fn first_run(path: &Path) {
        eprintln!("Configuration créée : {}", path.display());
    }

    pub fn open_in_editor(_path: &Path) {}

    pub fn open_url(_url: &str) {}

    pub fn run(_settings: Settings, _path: PathBuf, _args: Args) -> ! {
        fatal("ce système n'est pas pris en charge (Windows et macOS uniquement)")
    }
}

/// Ouvre la page de réglages dans le navigateur ; à défaut, le fichier de
/// configuration dans l'éditeur de texte. `welcome` : premier lancement.
pub fn open_settings(path: &std::path::Path, welcome: bool) {
    match crate::settings_ui::url(path, welcome) {
        Ok(url) => open_url(&url),
        Err(e) => {
            log::error!("page de réglages indisponible : {e}");
            open_in_editor(path);
        }
    }
}

/// Relance Bascule avec les mêmes arguments puis quitte ce processus.
#[cfg(not(target_os = "macos"))]
pub fn relaunch(args: &crate::Args) -> ! {
    match std::env::current_exe() {
        Ok(exe) => match std::process::Command::new(&exe).args(args.to_vec()).spawn() {
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
            let err = std::process::Command::new(&exe).args(args.to_vec()).exec();
            log::error!("redémarrage impossible : {err}");
        }
        Err(e) => log::error!("redémarrage impossible : {e}"),
    }
    crate::logging::flush();
    std::process::exit(1)
}
