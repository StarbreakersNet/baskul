//! Accès à l'API locale du daemon Bascule (voir `src/api.rs` à la racine).
//!
//! Le daemon publie son port et un jeton dans `api.json`. Le fichier est relu
//! à chaque appel : l'app suit ainsi les redémarrages du daemon.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Erreur transmise au front : `{ kind, message }`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "lowercase")]
pub enum Error {
    /// Bascule ne tourne pas, ou ne répond pas.
    Offline(String),
    /// Le daemon a refusé la demande (réglage invalide…).
    Refused(String),
}

#[derive(Deserialize)]
struct ApiFile {
    port: u16,
    token: String,
}

/// `api.json` : indiqué par le daemon quand il lance l'app, sinon à
/// l'emplacement par défaut de la configuration.
fn api_file() -> PathBuf {
    if let Some(p) = std::env::var_os("BASCULE_API_FILE") {
        return p.into();
    }
    dirs::config_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from(".")).join("Bascule").join("api.json")
}

fn offline() -> Error {
    Error::Offline("Bascule n'est pas lancé.".into())
}

/// Appel synchrone de l'API ; renvoie le JSON de la réponse.
pub fn request(method: &str, path: &str, body: Option<&Value>) -> Result<Value, Error> {
    let text = std::fs::read_to_string(api_file()).map_err(|_| offline())?;
    let api: ApiFile = serde_json::from_str(&text).map_err(|_| offline())?;
    let addr = SocketAddr::from(([127, 0, 0, 1], api.port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).map_err(|_| offline())?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let body = body.map(Value::to_string).unwrap_or_default();
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Bascule-Token: {}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        api.port,
        api.token,
        body.len()
    );
    let mut resp = Vec::new();
    stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body.as_bytes()))
        .and_then(|_| stream.read_to_end(&mut resp))
        .map_err(|_| offline())?;
    let resp = String::from_utf8_lossy(&resp);
    let (head, body) = resp.split_once("\r\n\r\n").ok_or_else(offline)?;
    let status: u16 = head.get(9..12).and_then(|c| c.parse().ok()).ok_or_else(offline)?;
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    match status {
        200 => Ok(value),
        // Jeton périmé : le daemon vient de redémarrer.
        403 => Err(offline()),
        _ => Err(Error::Refused(value["error"].as_str().unwrap_or("erreur inconnue").to_string())),
    }
}

/// Lance Bascule, installé avec l'app de réglages.
pub fn start() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // macOS : Bascule.app/Contents/Helpers/BasculeSettings.app/Contents/MacOS/…
    if cfg!(target_os = "macos")
        && let Some(app) = exe.ancestors().filter(|p| p.extension().is_some_and(|e| e == "app")).nth(1)
    {
        return Command::new("/usr/bin/open").arg(app).spawn().map(drop).map_err(|e| e.to_string());
    }
    let daemon = exe.with_file_name(if cfg!(windows) { "bascule.exe" } else { "bascule" });
    if !daemon.is_file() {
        return Err(format!("{} introuvable", daemon.display()));
    }
    let mut cmd = Command::new(&daemon);
    // Même fichier de configuration que l'instance qui a lancé l'app.
    if let Some(config) = std::env::var_os("BASCULE_CONFIG") {
        cmd.arg("--config").arg(config);
    }
    let mut child = cmd
        .env_remove("BASCULE_RELAUNCH")
        .spawn()
        .map_err(|e| format!("lancement de {} impossible : {e}", daemon.display()))?;
    // On attend sa fin pour ne pas laisser de processus zombie.
    std::thread::spawn(move || child.wait());
    Ok(())
}
