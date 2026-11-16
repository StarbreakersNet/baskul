//! Code propre à chaque système.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

#[cfg(any(windows, all(feature = "xcheck", target_os = "linux")))]
pub mod windows;

#[cfg(any(target_os = "macos", all(feature = "xcheck", target_os = "linux")))]
pub mod macos;

#[cfg(not(any(windows, target_os = "macos")))]
mod linux;

#[cfg(windows)]
use self::windows::run;
#[cfg(windows)]
pub use self::windows::{early_init, fatal, first_run, open_in_editor};

#[cfg(target_os = "macos")]
pub use self::macos::{early_init, fatal, first_run, open_in_editor};

#[cfg(not(any(windows, target_os = "macos")))]
use self::linux::run;
#[cfg(not(any(windows, target_os = "macos")))]
pub use self::linux::{early_init, fatal, first_run, open_in_editor};

use crate::Args;
use baskul_core::config::Settings;

/// Démarre la couche système. Windows et Linux : dans son propre thread, avec
/// sa propre boucle de messages (les crochets clavier et souris ne dépendent
/// jamais de l'interface). macOS : sur la boucle principale, celle de l'app
/// (à appeler depuis le thread principal).
pub fn start(settings: Settings, config_path: PathBuf, args: Args) {
    #[cfg(target_os = "macos")]
    self::macos::start(settings, config_path, args);
    #[cfg(not(target_os = "macos"))]
    {
        let spawned =
            std::thread::Builder::new().name("baskul-system".into()).spawn(move || run(settings, config_path, args));
        if let Err(e) = spawned {
            fatal(&format!("démarrage de Baskul impossible : {e}"));
        }
    }
}

/// Fonction qui ouvre la fenêtre de réglages (fournie par l'app) ;
/// l'argument indique un premier lancement.
type SettingsOpener = Box<dyn Fn(bool) + Send + Sync>;

static SETTINGS_OPENER: OnceLock<SettingsOpener> = OnceLock::new();

/// La fenêtre de réglages est ouverte : elle se rouvre après un redémarrage.
static SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);

/// Donne la fonction qui ouvre la fenêtre de réglages (appelable depuis
/// n'importe quel thread).
pub fn set_settings_opener(f: impl Fn(bool) + Send + Sync + 'static) {
    let _ = SETTINGS_OPENER.set(Box::new(f));
}

/// L'app signale l'ouverture ou la fermeture de la fenêtre de réglages.
pub fn set_settings_window_open(open: bool) {
    SETTINGS_OPEN.store(open, Ordering::Release);
}

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

/// Le programme `exe` n'est plus celui du lancement précédent : première
/// exécution d'une nouvelle version (ou toute première exécution). Sa taille
/// et sa date sont notées dans `marker` pour la fois suivante.
#[cfg(any(target_os = "macos", test, feature = "xcheck"))]
pub fn program_changed(marker: &Path, exe: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(exe) else { return false };
    let modified =
        meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    let id = format!("{}:{modified}", meta.len());
    if std::fs::read_to_string(marker).is_ok_and(|old| old.trim() == id) {
        return false;
    }
    if let Err(e) = std::fs::write(marker, &id) {
        log::warn!("écriture de {} impossible : {e}", marker.display());
    }
    true
}

/// Demande de redémarrage, traitée par la boucle principale avec
/// [`take_restart_request`] (macOS, Linux, configuration invalide).
pub fn request_restart() {
    RESTART_REQUESTED.store(true, Ordering::Release);
}

pub fn take_restart_request() -> bool {
    RESTART_REQUESTED.swap(false, Ordering::AcqRel)
}

/// Ouvre la fenêtre de réglages ; à défaut, le fichier de configuration
/// dans l'éditeur de texte (et renvoie `false`). `welcome` : premier
/// lancement.
pub fn open_settings(path: &Path, welcome: bool) -> bool {
    match SETTINGS_OPENER.get() {
        Some(open) => {
            open(welcome);
            true
        }
        None => {
            log::warn!("fenêtre de réglages indisponible : ouverture du fichier de configuration");
            open_in_editor(path);
            false
        }
    }
}

/// Relance Baskul avec les mêmes arguments puis quitte ce processus.
pub fn relaunch(args: &crate::Args) -> ! {
    let mut argv = args.to_vec();
    argv.push(RELAUNCH_ARG.into());
    // La fenêtre de réglages ouverte se rouvre dans la nouvelle instance.
    if SETTINGS_OPEN.load(Ordering::Acquire) {
        argv.push("--settings".into());
    }
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

    #[test]
    fn program_change_is_seen_once_per_version() {
        let dir = std::env::temp_dir().join(format!("baskul-program-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (marker, exe) = (dir.join("program-id"), dir.join("baskul"));
        std::fs::write(&exe, "version 1").unwrap();
        // Première exécution, puis lancements suivants du même programme.
        assert!(program_changed(&marker, &exe));
        assert!(!program_changed(&marker, &exe));
        assert!(!program_changed(&marker, &exe));
        // Mise à jour : le programme est remplacé.
        std::fs::write(&exe, "version 2, plus longue").unwrap();
        assert!(program_changed(&marker, &exe));
        assert!(!program_changed(&marker, &exe));
        // Programme introuvable : rien à signaler.
        assert!(!program_changed(&marker, &dir.join("absent")));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
