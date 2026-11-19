//! Baskul sur le poste de travail : la couche de chaque système (capture et
//! injection du clavier et de la souris, icône, fenêtres), l'API locale, la
//! ligne de commande, les journaux et les mises à jour, autour du cœur
//! (`baskul-core`).
//!
//! L'app (`app/src-tauri`, Tauri) l'héberge : [`prepare`] avant de démarrer
//! Tauri (ligne de commande, instance unique, configuration, journaux), puis
//! [`start`] une fois Tauri prêt. La couche système tourne dans ses propres
//! threads (Windows, Linux) ou sur la boucle principale de macOS, et n'attend
//! jamais l'interface.

// La vérification croisée (feature `xcheck`) compile le code des deux
// plateformes sur Linux sans pouvoir l'exécuter.
#![cfg_attr(feature = "xcheck", allow(dead_code, unused_imports, unused_variables))]
// Sur un système non pris en charge (Linux : tests uniquement), le code
// utilisé seulement par les couches Windows/macOS paraît inutilisé.
#![cfg_attr(all(not(any(windows, target_os = "macos")), not(feature = "xcheck")), allow(dead_code))]
#![allow(clippy::collapsible_if, clippy::too_many_arguments)]

pub mod api;
mod cli;
mod history;
mod icon;
pub mod logging;
mod pair;
pub mod platform;
mod profiles;
mod programs;
mod transfer;
pub mod tray;
mod update;

// Le cœur (crate `baskul-core`) : ses modules sont accessibles ici sous les
// mêmes chemins qu'avant (`crate::config`…). Selon le système, la couche
// plateforme n'en utilise qu'une partie.
#[allow(unused_imports)]
use baskul_core::{
    capture, clipboard, config, crypto, discovery, gamepad, geometry, hotkey, iface, keymap, latency, net, primary,
    protocol, secondary, services, smooth, status,
};

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub use baskul_core::{VERSION, activity, i18n};

#[derive(Debug, Default, Clone)]
pub struct Args {
    pub config: Option<PathBuf>,
    /// Journaux aussi sur la sortie d'erreur (Windows : console attachée).
    pub console: bool,
    /// Ouvre la fenêtre de réglages au démarrage (ou celle de l'instance
    /// déjà lancée).
    pub settings: bool,
    /// Lancé par un redémarrage : attend la fin de l'instance précédente.
    pub relaunched: bool,
    /// Commande pour Baskul en cours d'exécution (`switch`, `status`).
    pub command: Option<cli::Command>,
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
                "switch" => {
                    let to = it
                        .next()
                        .ok_or("switch attend une cible : primary, secondary, toggle, here, other, next ou un nom")?;
                    a.command = Some(cli::Command::Switch(to));
                }
                "status" => a.command = Some(cli::Command::Status),
                platform::RELAUNCH_ARG => a.relaunched = true,
                "--print-config" => {
                    print!("{}", config::default_config_text(config::default_role(), &config::generate_secret()));
                    std::process::exit(0);
                }
                "--version" | "-V" => {
                    println!("baskul {VERSION}");
                    std::process::exit(0);
                }
                "--help" | "-h" => {
                    println!(
                        "baskul {VERSION}\n\nUsage : baskul [--config FICHIER] [--console] [--settings]\n       \
                         baskul [--config FICHIER] switch CIBLE\n       \
                         baskul [--config FICHIER] status\n\n  \
                         --config FICHIER  fichier de configuration (défaut : {})\n  \
                         --console         affiche aussi le journal dans le terminal\n  \
                         --settings        ouvre la fenêtre de réglages\n  \
                         --print-config    affiche une configuration par défaut\n  \
                         switch CIBLE      le clavier et la souris passent sur la machine principale\n                    \
                         (primary), la dernière pilotée (secondary), celle-ci (here),\n                    \
                         une autre (other), celle où ils ne sont pas (toggle), la\n                    \
                         suivante (next) ou une pilotée donnée par son nom\n  \
                         status            état de Baskul (rôle, machines connectées, machine active)",
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
    /// Baskul tourne déjà.
    Running,
    Other(String),
}

/// Empêche deux instances simultanées. Un redémarrage attend que l'instance
/// précédente se termine ; un lancement par l'utilisateur s'arrête aussitôt.
fn acquire_instance_lock(relaunched: bool) -> Result<File, LockError> {
    let dir = config::config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("baskul.lock");
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| LockError::Other(format!("impossible de créer {} : {e}", path.display())))?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) => {
                if !relaunched {
                    return Err(LockError::Running);
                }
                if Instant::now() > deadline {
                    return Err(LockError::Other("Baskul est déjà en cours d'exécution sur cette machine.".into()));
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

/// Verrou d'instance, gardé jusqu'à la fin du processus.
static INSTANCE_LOCK: OnceLock<File> = OnceLock::new();

/// Baskul prêt à démarrer, une fois [`prepare`] passé.
pub struct Prepared {
    path: PathBuf,
    args: Args,
    state: State,
}

enum State {
    Ready {
        settings: Box<config::Settings>,
        created: bool,
        reopen_settings: bool,
    },
    /// Configuration illisible ou invalide : seule la fenêtre de réglages
    /// s'ouvre, pour la corriger.
    ConfigError(String),
}

impl Prepared {
    /// Configuration invalide : Baskul ne tourne pas, l'app doit se fermer
    /// avec la fenêtre de réglages.
    pub fn config_error(&self) -> bool {
        matches!(self.state, State::ConfigError(_))
    }

    /// Notifications du système à la connexion et à la déconnexion d'une machine.
    pub fn notifications(&self) -> bool {
        matches!(&self.state, State::Ready { settings, .. } if settings.ui.notifications)
    }
}

/// Avant de démarrer l'interface : ligne de commande (les commandes
/// `switch` et `status` se terminent ici), instance unique (une instance déjà
/// lancée ouvre sa fenêtre de réglages, et ce processus se termine),
/// configuration, journaux, API locale.
pub fn prepare() -> Prepared {
    let args = match Args::parse() {
        Ok(a) => a,
        // Commande lancée par une autre application : erreur dans le
        // terminal seulement.
        Err(e) if cli::requested() => std::process::exit(cli::fail(&e)),
        Err(e) => platform::fatal(&e),
    };
    platform::early_init(&args);

    let path = args.config.clone().unwrap_or_else(config::default_config_path);
    if let Some(cmd) = &args.command {
        std::process::exit(cli::run(cmd, &path));
    }
    // Ancien nom (Bascule) : ses réglages sont repris.
    match config::migrate_old_config_dir() {
        Ok(true) => eprintln!("Baskul : réglages de {} repris", config::OLD_NAME),
        Ok(false) => {}
        Err(e) => platform::fatal(&e),
    }
    match acquire_instance_lock(args.relaunched) {
        Ok(f) => {
            let _ = INSTANCE_LOCK.set(f);
        }
        // Baskul tourne déjà : il ouvre sa fenêtre de réglages.
        Err(LockError::Running) => {
            if let Err(e) = api::call(&path, "POST", "/api/show-settings", None) {
                eprintln!("Baskul : {e}");
            }
            std::process::exit(0);
        }
        Err(LockError::Other(e)) => platform::fatal(&e),
    }

    let loaded = config::load_or_create(&path);
    let level = match &loaded {
        Ok((cfg, _)) => cfg.validate().map_or(log::LevelFilter::Info, |s| s.log_level),
        Err(_) => log::LevelFilter::Info,
    };
    logging::init(config::log_dir().join("baskul.log"), level, args.console);
    log::info!("Baskul {VERSION} ({}) — configuration : {}", std::env::consts::OS, path.display());
    if let Err(e) = api::start(&path) {
        log::error!("{e}");
    }
    history::start(&path);
    activity::record(activity::Kind::Started { restarted: args.relaunched });

    let checked = loaded.and_then(|(cfg, created)| cfg.validate().map(|s| (s, created)));
    let (settings, created) = match checked {
        Ok(x) => x,
        Err(e) => return Prepared { path, args, state: State::ConfigError(e) },
    };
    i18n::init(settings.ui.language);
    update::cleanup();
    if settings.check_updates {
        update::start_auto_check();
    }
    log::info!(
        "rôle : machine {} ; nom : {} ; réseau : {}",
        settings.role.label(),
        settings.name,
        match (settings.role, &settings.connect) {
            (protocol::Role::Primary, _) => format!("écoute sur {}", settings.listen),
            (_, Some(addr)) => format!("connexion à {addr}"),
            (_, None) => "recherche de la machine principale".into(),
        }
    );
    let reopen_settings = update::take_reopen_settings();
    Prepared { path, args, state: State::Ready { settings: Box::new(settings), created, reopen_settings } }
}

/// Démarre Baskul, une fois l'interface prête (et la fonction d'ouverture de
/// la fenêtre de réglages donnée à [`platform::set_settings_opener`]). Sur
/// macOS, à appeler sur le thread principal : la couche système s'installe
/// sur sa boucle. Ailleurs, elle tourne dans son propre thread.
pub fn start(prepared: Prepared) {
    let Prepared { path, args, state } = prepared;
    let (settings, created, reopen_settings) = match state {
        State::Ready { settings, created, reopen_settings } => (settings, created, reopen_settings),
        State::ConfigError(e) => return start_config_error(&path, args, &e),
    };
    if created {
        log::info!("configuration créée : {}", path.display());
        platform::first_run(&path);
    } else if reopen_settings {
        log::info!("mise à jour installée : réouverture des réglages");
        platform::open_settings(&path, false);
    } else if args.settings {
        platform::open_settings(&path, false);
    }
    programs::start(settings.pause_edges_for.clone());
    profiles::refresh_menu(&path);
    platform::start(*settings, path, args);
}

/// Configuration illisible ou invalide : la fenêtre de réglages s'ouvre pour
/// la corriger, puis Baskul redémarre.
fn start_config_error(path: &Path, args: Args, e: &str) {
    log::error!("{e}");
    i18n::init(config::Language::System);
    status::set_conn(status::Conn::Error(i18n::tf("status.configInvalid", &[("error", e)])));
    api::set_restart_handler(platform::request_restart);
    let spawned = std::thread::Builder::new().name("baskul-restart".into()).spawn(move || {
        loop {
            if platform::take_restart_request() {
                platform::relaunch(&args);
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    });
    if let Err(err) = spawned {
        log::error!("surveillance du redémarrage impossible : {err}");
    }
    if !platform::open_settings(path, false) {
        platform::fatal(&format!(
            "Erreur dans la configuration :\n{e}\n\nFichier : {}\n\nCorrigez-la puis relancez Baskul.",
            path.display()
        ));
    }
}
