//! Bascule — partage du clavier, de la souris et du presse-papiers entre un
//! PC Windows et un Mac.

// Pas de fenêtre console sous Windows (les journaux vont dans un fichier).
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]
// La vérification croisée (feature `xcheck`) compile le code des deux
// plateformes sur Linux sans pouvoir l'exécuter.
#![cfg_attr(feature = "xcheck", allow(dead_code, unused_imports, unused_variables))]
// Sur un système non pris en charge (Linux : tests uniquement), le code
// utilisé seulement par les couches Windows/macOS paraît inutilisé.
#![cfg_attr(all(not(any(windows, target_os = "macos")), not(feature = "xcheck")), allow(dead_code))]
#![allow(clippy::collapsible_if, clippy::too_many_arguments)]

mod api;
mod app;
mod capture;
mod clipboard;
mod config;
mod crypto;
mod geometry;
mod hotkey;
mod i18n;
mod icon;
mod keymap;
mod logging;
mod net;
mod platform;
mod primary;
mod protocol;
mod secondary;
#[cfg(any(target_os = "macos", test, feature = "xcheck"))]
mod smooth;
mod status;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Options de la ligne de commande.
#[derive(Debug, Default, Clone)]
pub struct Args {
    pub config: Option<PathBuf>,
    /// Journaux aussi sur la sortie d'erreur (Windows : console attachée).
    pub console: bool,
    /// Ouvre seulement l'app de réglages.
    pub settings: bool,
}

impl Args {
    fn parse() -> Result<Args, String> {
        let mut a = Args::default();
        let mut it = std::env::args().skip(1);
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--config" | "-c" => {
                    a.config = Some(PathBuf::from(it.next().ok_or("--config attend un chemin")?));
                }
                "--console" => a.console = true,
                "--settings" => a.settings = true,
                "--print-config" => {
                    print!("{}", config::default_config_text(config::default_role(), &config::generate_secret()));
                    std::process::exit(0);
                }
                "--version" | "-V" => {
                    println!("bascule {VERSION}");
                    std::process::exit(0);
                }
                "--help" | "-h" => {
                    println!(
                        "bascule {VERSION}\n\nUsage : bascule [--config FICHIER] [--console] [--settings]\n\n  \
                         --config FICHIER  fichier de configuration (défaut : {})\n  \
                         --console         affiche aussi le journal dans le terminal\n  \
                         --settings        ouvre l'app de réglages\n  \
                         --print-config    affiche une configuration par défaut",
                        config::default_config_path().display()
                    );
                    std::process::exit(0);
                }
                // macOS ajoute parfois -psn_… quand l'app est lancée depuis le Finder.
                s if s.starts_with("-psn_") => {}
                other => return Err(format!("option inconnue : {other}")),
            }
        }
        Ok(a)
    }

    /// Arguments à repasser lors d'un redémarrage.
    pub fn to_vec(&self) -> Vec<String> {
        let mut v = vec![];
        if let Some(c) = &self.config {
            v.push("--config".into());
            v.push(c.display().to_string());
        }
        if self.console {
            v.push("--console".into());
        }
        v
    }
}

enum LockError {
    /// Bascule tourne déjà.
    Running,
    Other(String),
}

/// Empêche deux instances simultanées. Un redémarrage attend que l'instance
/// précédente se termine ; un lancement par l'utilisateur s'arrête aussitôt.
fn acquire_instance_lock() -> Result<File, LockError> {
    let dir = config::config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("bascule.lock");
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| LockError::Other(format!("impossible de créer {} : {e}", path.display())))?;
    let relaunched = std::env::var_os(platform::RELAUNCH_ENV).is_some();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) => {
                if !relaunched {
                    return Err(LockError::Running);
                }
                if Instant::now() > deadline {
                    return Err(LockError::Other("Bascule est déjà en cours d'exécution sur cette machine.".into()));
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(std::fs::TryLockError::Error(e)) => {
                log::warn!("verrou d'instance indisponible ({e}) : on continue");
                return Ok(file);
            }
        }
    }
}

fn main() {
    let args = match Args::parse() {
        Ok(a) => a,
        Err(e) => platform::fatal(&e),
    };
    platform::early_init(&args);

    let path = args.config.clone().unwrap_or_else(config::default_config_path);
    if args.settings {
        platform::open_settings(&path, false);
        std::process::exit(0);
    }
    let _lock = match acquire_instance_lock() {
        Ok(f) => f,
        // Bascule tourne déjà : on ouvre ses réglages.
        Err(LockError::Running) => {
            platform::open_settings(&path, false);
            std::process::exit(0);
        }
        Err(LockError::Other(e)) => platform::fatal(&e),
    };

    let loaded = config::load_or_create(&path);
    let level = match &loaded {
        Ok((cfg, _)) => cfg.validate().map_or(log::LevelFilter::Info, |s| s.log_level),
        Err(_) => log::LevelFilter::Info,
    };
    logging::init(config::log_dir().join("bascule.log"), level, args.console);
    log::info!("Bascule {VERSION} ({}) — configuration : {}", std::env::consts::OS, path.display());
    if let Err(e) = api::start(&path) {
        log::error!("{e}");
    }

    let (cfg, created) = match loaded {
        Ok(x) => x,
        Err(e) => config_error(&path, &args, &e),
    };
    let settings = match cfg.validate() {
        Ok(s) => s,
        Err(e) => config_error(&path, &args, &e),
    };
    i18n::init(settings.ui.language);
    log::info!("rôle : machine {} ; réseau : {:?} ; nom : {}", settings.role.label(), settings.endpoint, settings.name);

    if created {
        log::info!("configuration créée : {}", path.display());
        platform::first_run(&path);
    }

    platform::run(settings, path, args);
}

/// Configuration illisible ou invalide : l'app de réglages s'ouvre pour la
/// corriger, puis Bascule redémarre. Sans elle, le fichier s'ouvre dans
/// l'éditeur de texte.
fn config_error(path: &Path, args: &Args, e: &str) -> ! {
    log::error!("{e}");
    i18n::init(config::Language::System);
    status::set_conn(status::Conn::Error(i18n::tf("status.configInvalid", &[("error", e)])));
    api::set_restart_handler(platform::request_restart);
    match api::launch_settings_app(path, false) {
        Ok(mut child) => {
            let launched = Instant::now();
            loop {
                if platform::take_restart_request() {
                    platform::relaunch(args);
                }
                // Réglages fermés sans correction. (Une sortie immédiate
                // signifie qu'une fenêtre de réglages était déjà ouverte.)
                if !matches!(child.try_wait(), Ok(None)) && launched.elapsed() > Duration::from_secs(3) {
                    log::info!("configuration toujours invalide : arrêt");
                    logging::flush();
                    std::process::exit(1);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        Err(err) => {
            log::warn!("{err}");
            platform::open_in_editor(path);
            platform::fatal(&format!(
                "Erreur dans la configuration :\n{e}\n\nFichier : {}\n\nCorrigez-la puis relancez Bascule.",
                path.display()
            ));
        }
    }
}
