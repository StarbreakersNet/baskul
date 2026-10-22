//! Réglages de Bascule : interface Tauri + Vue 3. Le front (dossier `app/src`)
//! appelle ces commandes, qui relaient les demandes à l'API locale du daemon.

// Pas de fenêtre console sous Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod daemon;

use daemon::Error;
use serde_json::{Value, json};
use tauri::Manager;

/// Exécute un appel à l'API hors du thread de l'interface.
async fn call(method: &'static str, path: &'static str, body: Option<Value>) -> Result<Value, Error> {
    tauri::async_runtime::spawn_blocking(move || daemon::request(method, path, body.as_ref()))
        .await
        .map_err(|e| Error::Offline(e.to_string()))?
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

/// Enregistre la configuration ; le daemon redémarre ensuite.
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

/// Fin du journal de Bascule.
#[tauri::command]
async fn read_log() -> Result<Value, Error> {
    call("GET", "/api/log", None).await
}

#[tauri::command]
async fn open_log_file() -> Result<Value, Error> {
    call("POST", "/api/open-log", None).await
}

/// Saisie d'un raccourci par le crochet clavier du daemon : `start`, `poll`
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

#[tauri::command]
fn start_daemon() -> Result<(), Error> {
    daemon::start().map_err(Error::Refused)
}

/// Options de lancement : `--bienvenue` au premier lancement de Bascule.
#[tauri::command]
fn launch_options() -> Value {
    json!({ "welcome": std::env::args().any(|a| a == "--bienvenue") })
}

fn main() {
    tauri::Builder::default()
        // Une seule fenêtre de réglages : un second lancement la met en avant.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .setup(|app| {
            // Lancée par Bascule, la fenêtre doit passer au premier plan.
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_focus();
            }
            Ok(())
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
            start_daemon,
            launch_options
        ])
        .run(tauri::generate_context!())
        .expect("démarrage de l'app de réglages");
}
