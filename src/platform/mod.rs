//! Code propre à chaque système.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

/// Option ajoutée lors d'un redémarrage : la nouvelle instance attend alors
/// la fin de l'ancienne. (Une option plutôt qu'une variable d'environnement :
/// `open` ne transmet pas l'environnement.)
pub const RELAUNCH_ARG: &str = "--relaunched";

static RESTART_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Programme à relancer au redémarrage, quand ce n'est plus celui qui tourne :
/// après une mise à jour, le programme en cours a été renommé ou déplacé, et
/// son chemin d'origine désigne la nouvelle version.
static RELAUNCH_EXE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_relaunch_exe(exe: PathBuf) {
    *RELAUNCH_EXE.lock().unwrap_or_else(|e| e.into_inner()) = Some(exe);
}

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
            let _ = std::thread::Builder::new().name("baskul-settings-wait".into()).spawn(move || child.wait());
        }
        Err(e) => {
            log::warn!("{e} : ouverture du fichier de configuration");
            open_in_editor(path);
        }
    }
}

/// Relance Baskul avec les mêmes arguments puis quitte ce processus.
pub fn relaunch(args: &crate::Args) -> ! {
    let mut argv = args.to_vec();
    argv.push(RELAUNCH_ARG.into());
    match launch_again(&argv) {
        Ok(()) => log::info!("redémarrage…"),
        Err(e) => log::error!("redémarrage impossible : {e}"),
    }
    crate::logging::flush();
    std::process::exit(0)
}

fn launch_again(argv: &[String]) -> std::io::Result<()> {
    let chosen = RELAUNCH_EXE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let exe = match chosen {
        Some(exe) => exe,
        None => std::env::current_exe()?,
    };
    // macOS, dans Baskul.app : relance par LaunchServices, comme depuis le
    // Finder. Un nouveau processus, et non le même relancé par `exec` : avec
    // `exec`, l'ancienne icône laissait un emplacement vide dans la barre des
    // menus, et la nouvelle restait invisible.
    if cfg!(target_os = "macos")
        && let Some(app) = app_bundle(&exe)
    {
        let status =
            std::process::Command::new("/usr/bin/open").arg("-n").arg(&app).arg("--args").args(argv).status()?;
        if status.success() {
            return Ok(());
        }
        log::warn!("relance par open impossible ({status}) : lancement direct");
    }
    std::process::Command::new(&exe).args(argv).spawn().map(drop)
}

/// Bundle `.app` dont `exe` est l'exécutable principal
/// (`Baskul.app/Contents/MacOS/baskul`).
pub fn app_bundle(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let app = contents.parent()?;
    let ok = macos.file_name()? == "MacOS" && contents.file_name()? == "Contents" && app.extension()? == "app";
    ok.then(|| app.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_bundle_of_the_main_executable() {
        assert_eq!(
            app_bundle(Path::new("/Applications/Baskul.app/Contents/MacOS/baskul")),
            Some(PathBuf::from("/Applications/Baskul.app"))
        );
        assert_eq!(app_bundle(Path::new("/Users/moi/baskul/target/release/baskul")), None);
        assert_eq!(app_bundle(Path::new("/Applications/Baskul.app/Contents/Resources/baskul")), None);
        assert_eq!(app_bundle(Path::new("baskul")), None);
    }
}
