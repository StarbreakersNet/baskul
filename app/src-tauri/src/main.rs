//! Baskul : un clavier, une souris et un presse-papiers partagés entre
//! plusieurs ordinateurs.
//!
//! Un seul exécutable. Avant de démarrer Tauri, `baskul_desktop::prepare`
//! traite la ligne de commande (`baskul switch …`, `baskul status`),
//! l'instance unique et la configuration. Une fois Tauri prêt, Baskul démarre
//! (`baskul_desktop::start`) : sa couche système tourne dans ses propres
//! threads (sur la boucle principale pour macOS) et n'attend jamais la
//! fenêtre. Son icône (zone de notification, barre des menus) est dessinée
//! ici, avec Tauri (`tray.rs`). La fenêtre de réglages (Vue 3, dans `app/src`) n'existe que
//! lorsqu'elle est ouverte ; ses commandes appellent l'API de Baskul dans le
//! même processus.

// Pas de fenêtre console sous Windows (les journaux vont dans un fichier ;
// `--console` et les commandes s'attachent à la console qui les a lancées).
#![cfg_attr(windows, windows_subsystem = "windows")]

mod tray;

use baskul_desktop::{api, platform};
use serde::Serialize;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, RunEvent, WebviewWindowBuilder, WindowEvent};

/// Libellé de la fenêtre de réglages (voir tauri.conf.json).
const SETTINGS: &str = "main";

/// Premier lancement : la fenêtre de réglages s'ouvre sur l'accueil.
static WELCOME: AtomicBool = AtomicBool::new(false);

/// Configuration invalide : l'app se ferme avec la fenêtre de réglages.
static CONFIG_ERROR: AtomicBool = AtomicBool::new(false);

/// Erreur transmise au front : `{ kind, message }`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "lowercase")]
enum Error {
    /// Baskul ne répond pas.
    #[allow(dead_code)]
    Offline(String),
    /// Baskul a refusé la demande (réglage invalide…).
    Refused(String),
}

/// Appel de l'API de Baskul, hors du thread de l'interface.
async fn call(method: &'static str, path: &'static str, body: Option<Value>) -> Result<Value, Error> {
    tauri::async_runtime::spawn_blocking(move || api::call_local(method, path, body.as_ref()))
        .await
        .map_err(|e| Error::Refused(e.to_string()))?
        .map_err(Error::Refused)
}

/// État de la connexion (interrogé régulièrement).
#[tauri::command]
async fn daemon_status() -> Result<Value, Error> {
    call("GET", "/api/status", None).await
}

/// Configuration et informations sur la machine.
#[tauri::command]
async fn load_settings() -> Result<Value, Error> {
    call("GET", "/api/config", None).await
}

/// Enregistre la configuration ; Baskul redémarre ensuite (la fenêtre se
/// rouvre).
#[tauri::command]
async fn save_settings(config: Value) -> Result<Value, Error> {
    call("POST", "/api/config", Some(config)).await
}

#[tauri::command]
async fn restart_daemon() -> Result<Value, Error> {
    call("POST", "/api/restart", None).await
}

#[tauri::command]
async fn open_config_file() -> Result<Value, Error> {
    call("POST", "/api/open-file", None).await
}

/// Fin du journal de Baskul.
#[tauri::command]
async fn read_log() -> Result<Value, Error> {
    call("GET", "/api/log", None).await
}

#[tauri::command]
async fn open_log_file() -> Result<Value, Error> {
    call("POST", "/api/open-log", None).await
}

/// Saisie d'un raccourci par le crochet clavier de Baskul : `start`, `poll`
/// ou `cancel`.
#[tauri::command]
async fn capture_hotkey(action: String) -> Result<Value, Error> {
    let method = match action.as_str() {
        "start" => "POST",
        "poll" => "GET",
        "cancel" => "DELETE",
        _ => return Err(Error::Refused(format!("action inconnue : {action}"))),
    };
    call(method, "/api/hotkey/capture", None).await
}

/// Cartes réseau et machines principales trouvées sur le réseau local.
#[tauri::command]
async fn network_info() -> Result<Value, Error> {
    call("GET", "/api/network", None).await
}

/// Le clavier et la souris passent sur une machine (`primary` ou un nom).
#[tauri::command]
async fn switch_to(to: String) -> Result<Value, Error> {
    call("POST", "/api/switch", Some(json!({ "to": to }))).await
}

/// Mises à jour : `check` ou `install` (installation puis redémarrage).
#[tauri::command]
async fn update_action(action: String) -> Result<Value, Error> {
    let path = match action.as_str() {
        "check" => "/api/update/check",
        "install" => "/api/update/install",
        _ => return Err(Error::Refused(format!("action inconnue : {action}"))),
    };
    call("POST", path, None).await
}

/// Baskul tourne dans ce processus : rien à démarrer.
#[tauri::command]
fn start_daemon() {}

/// Options d'ouverture de la fenêtre : `welcome` au premier lancement.
#[tauri::command]
fn launch_options() -> Value {
    json!({ "welcome": WELCOME.swap(false, Ordering::AcqRel) })
}

/// Ouvre la fenêtre de réglages, ou la met au premier plan. Appelable depuis
/// n'importe quel thread (menu de l'icône, API…).
fn show_settings(app: &AppHandle, welcome: bool) {
    if welcome {
        WELCOME.store(true, Ordering::Release);
    }
    let handle = app.clone();
    let shown = app.run_on_main_thread(move || {
        if let Some(w) = handle.get_webview_window(SETTINGS) {
            let _ = w.unminimize();
            let _ = w.show();
            let _ = w.set_focus();
        } else {
            let Some(config) = handle.config().app.windows.iter().find(|w| w.label == SETTINGS).cloned() else {
                log::error!("fenêtre de réglages absente de la configuration de l'app");
                return;
            };
            match WebviewWindowBuilder::from_config(&handle, &config).and_then(|b| b.build()) {
                Ok(w) => {
                    let _ = w.set_focus();
                }
                Err(e) => {
                    log::error!("ouverture de la fenêtre de réglages impossible : {e}");
                    return;
                }
            }
        }
        platform::set_settings_window_open(true);
    });
    if let Err(e) = shown {
        log::error!("ouverture de la fenêtre de réglages impossible : {e}");
    }
}

fn main() {
    let prepared = baskul_desktop::prepare();
    CONFIG_ERROR.store(prepared.config_error(), Ordering::Release);

    let app = tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            platform::set_settings_opener(move |welcome| show_settings(&handle, welcome));
            baskul_desktop::start(prepared);
            tray::install(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == SETTINGS && matches!(event, WindowEvent::Destroyed) {
                platform::set_settings_window_open(false);
            }
        })
        .invoke_handler(tauri::generate_handler![
            daemon_status,
            load_settings,
            save_settings,
            restart_daemon,
            open_config_file,
            read_log,
            open_log_file,
            capture_hotkey,
            network_info,
            switch_to,
            update_action,
            start_daemon,
            launch_options
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| platform::fatal(&format!("démarrage de l'interface impossible : {e}")));

    app.run(|app, event| match event {
        // Fenêtre de réglages fermée : Baskul continue (icône), sauf si la
        // configuration est invalide (il ne tourne pas).
        RunEvent::ExitRequested { code: None, api, .. } if !CONFIG_ERROR.load(Ordering::Acquire) => {
            api.prevent_exit();
        }
        // macOS : double-clic sur Baskul.app déjà lancée.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => show_settings(app, false),
        _ => {
            let _ = app;
        }
    });
}
