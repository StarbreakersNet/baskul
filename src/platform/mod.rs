//! Code propre à chaque système.

#[cfg(any(windows, all(feature = "xcheck", target_os = "linux")))]
pub mod windows;

#[cfg(any(target_os = "macos", all(feature = "xcheck", target_os = "linux")))]
pub mod macos;

#[cfg(windows)]
pub use self::windows::{early_init, fatal, first_run, open_in_editor, run};

#[cfg(target_os = "macos")]
pub use self::macos::{early_init, fatal, first_run, open_in_editor, run};

#[cfg(not(any(windows, target_os = "macos")))]
pub use self::unsupported::{early_init, fatal, first_run, open_in_editor, run};

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

    pub fn run(_settings: Settings, _path: PathBuf, _args: Args) -> ! {
        fatal("ce système n'est pas pris en charge (Windows et macOS uniquement)")
    }
}

/// Texte du message affiché au premier lancement.
pub fn first_run_text(path: &std::path::Path) -> String {
    format!(
        "Bienvenue dans Bascule !\n\n\
         Un fichier de configuration vient d'être créé :\n{}\n\n\
         1. Copiez la ligne « secret = … » de cette machine dans la configuration de l'autre machine (le secret doit être identique).\n\
         2. Sur UNE des deux machines, indiquez l'adresse de l'autre dans « connect ».\n\
         3. Vérifiez « role » (primary = la machine qui a le clavier et la souris).\n\
         4. Menu Bascule > Redémarrer.\n\n\
         Le fichier va s'ouvrir dans l'éditeur de texte.",
        path.display()
    )
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
