//! Commandes pour d'autres applications (Keyboard Maestro, Raccourcis,
//! AutoHotkey, scripts…). Elles s'adressent au Bascule en cours d'exécution
//! par son API locale :
//!
//! - `bascule switch primary|secondary|toggle|here|other` : le clavier et la
//!   souris passent sur la machine principale, sur la machine pilotée, sur
//!   l'autre que maintenant, sur celle où la commande est lancée, ou sur
//!   l'autre ;
//! - `bascule status` : état de Bascule, une information par ligne.
//!
//! Code de sortie : 0 si tout va bien, 1 sinon (message sur la sortie
//! d'erreur).

use crate::api;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Switch(String),
    Status,
}

/// Noms des commandes (premier argument).
pub const NAMES: [&str; 2] = ["switch", "status"];

/// La ligne de commande demande une commande (et non le lancement de Bascule).
pub fn requested() -> bool {
    std::env::args().skip(1).any(|a| NAMES.contains(&a.as_str()))
}

/// Exécute la commande ; renvoie le code de sortie du processus.
pub fn run(cmd: &Command, config_path: &Path) -> i32 {
    let result = match cmd {
        // Rien n'est affiché en cas de succès (pas de fenêtre de résultat
        // dans Keyboard Maestro).
        Command::Switch(to) => api::call(config_path, "POST", "/api/switch", Some(&json!({ "to": to }))).map(drop),
        Command::Status => api::call(config_path, "GET", "/api/status", None).map(|v| print!("{}", status_text(&v))),
    };
    match result {
        Ok(()) => 0,
        Err(e) => fail(&e),
    }
}

/// Erreur d'une commande : message sur la sortie d'erreur, code 1.
pub fn fail(msg: &str) -> i32 {
    eprintln!("bascule : {msg}");
    1
}

/// `bascule status` : clés et valeurs en anglais, faciles à lire dans un
/// script (`bascule status | grep active`).
fn status_text(v: &Value) -> String {
    let connected = v["state"] == "connected";
    let active = if v["remote_active"].as_bool().unwrap_or(false) { "secondary" } else { "primary" };
    let mut out = format!("role: {}\nconnected: {}\n", v["role"].as_str().unwrap_or("unknown"), yes(connected));
    if connected {
        out.push_str(&format!("peer: {}\n", v["detail"].as_str().unwrap_or_default()));
    }
    out.push_str(&format!("active: {active}\nlocked: {}\n", yes(v["locked"].as_bool().unwrap_or(false))));
    out
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_is_easy_to_parse() {
        let v = json!({ "state": "connected", "detail": "Mac mini", "role": "primary", "remote_active": true, "locked": false });
        assert_eq!(status_text(&v), "role: primary\nconnected: yes\npeer: Mac mini\nactive: secondary\nlocked: no\n");
        let v = json!({ "state": "waiting", "detail": "attente", "role": "secondary", "remote_active": false });
        assert_eq!(status_text(&v), "role: secondary\nconnected: no\nactive: primary\nlocked: no\n");
    }
}
