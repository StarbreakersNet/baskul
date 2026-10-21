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

mod app;
mod clipboard;
mod config;
mod crypto;
mod geometry;
mod hotkey;
mod icon;
mod keymap;
mod logging;
mod net;
mod platform;
mod primary;
mod protocol;
mod secondary;
mod status;

use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Options de la ligne de commande.
#[derive(Debug, Default, Clone)]
pub struct Args {
    pub config: Option<PathBuf>,
    /// Journaux aussi sur la sortie d'erreur (Windows : console attachée).
    pub console: bool,
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
                        "bascule {VERSION}\n\nUsage : bascule [--config FICHIER] [--console]\n\n  \
                         --config FICHIER  fichier de configuration (défaut : {})\n  \
                         --console         affiche aussi le journal dans le terminal\n  \
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

/// Empêche deux instances simultanées (un redémarrage attend que
/// l'instance précédente se termine).
fn acquire_instance_lock() -> Result<File, String> {
    let dir = config::config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("bascule.lock");
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| format!("impossible de créer {} : {e}", path.display()))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) => {
                if Instant::now() > deadline {
                    return Err("Bascule est déjà en cours d'exécution sur cette machine.".into());
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
    let (cfg, created) = match config::load_or_create(&path) {
        Ok(x) => x,
        Err(e) => platform::fatal(&format!("{e}\n\nFichier : {}", path.display())),
    };
    let level = match cfg.validate() {
        Ok(s) => s.log_level,
        Err(_) => log::LevelFilter::Info,
    };
    logging::init(config::log_dir().join("bascule.log"), level, args.console);
    log::info!("Bascule {VERSION} ({}) — configuration : {}", std::env::consts::OS, path.display());

    let settings = match cfg.validate() {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            platform::open_in_editor(&path);
            platform::fatal(&format!(
                "Erreur dans la configuration :\n{e}\n\nFichier : {}\n\nCorrigez-la puis relancez Bascule.",
                path.display()
            ));
        }
    };
    log::info!("rôle : machine {} ; réseau : {:?} ; nom : {}", settings.role.label(), settings.endpoint, settings.name);

    let _lock = match acquire_instance_lock() {
        Ok(f) => f,
        Err(e) => platform::fatal(&e),
    };

    if created {
        log::info!("configuration créée : {}", path.display());
        platform::first_run(&path);
    }

    platform::run(settings, path, args);
}
