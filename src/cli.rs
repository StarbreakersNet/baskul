//! Commandes pour d'autres applications (Keyboard Maestro, Raccourcis,
//! AutoHotkey, scripts…). Elles s'adressent au Baskul en cours d'exécution
//! par son API locale :
//!
//! - `baskul switch CIBLE` : le clavier et la souris passent sur la machine
//!   principale (`primary`), sur la dernière pilotée utilisée (`secondary`),
//!   sur la machine où la commande est lancée (`here`), sur une autre
//!   (`other`), sur celle où ils ne sont pas (`toggle`), sur la suivante
//!   (`next`), ou sur une pilotée donnée par son nom ;
//! - `baskul status` : état de Baskul, une information par ligne.
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

/// La ligne de commande demande une commande (et non le lancement de Baskul).
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
    eprintln!("baskul : {msg}");
    1
}

/// `baskul status` : clés et valeurs en anglais, faciles à lire dans un
/// script (`baskul status | grep active`). `active` : machine où agissent le
/// clavier et la souris (`primary`, ou le nom d'une pilotée ; sur une
/// pilotée : `here` ou `elsewhere`).
fn status_text(v: &Value) -> String {
    let role = v["role"].as_str().unwrap_or("unknown");
    let peers: Vec<&str> = v["peers"].as_array().into_iter().flatten().filter_map(|p| p["name"].as_str()).collect();
    let remote = v["remote_active"].as_bool().unwrap_or(false);
    let active = match (role, v["active_peer"].as_str()) {
        ("secondary", _) => {
            if remote {
                "here"
            } else {
                "elsewhere"
            }
        }
        (_, Some(name)) => name,
        _ => "primary",
    };
    format!(
        "role: {role}\nconnected: {}\npeers: {}\nactive: {active}\nlocked: {}\n",
        yes(!peers.is_empty()),
        peers.join(", "),
        yes(v["locked"].as_bool().unwrap_or(false))
    )
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_is_easy_to_parse() {
        let v = json!({
            "role": "primary",
            "peers": [{ "name": "Atelier" }, { "name": "Salon" }],
            "remote_active": true,
            "active_peer": "Salon",
            "locked": false
        });
        assert_eq!(
            status_text(&v),
            "role: primary\nconnected: yes\npeers: Atelier, Salon\nactive: Salon\nlocked: no\n"
        );
        let v = json!({ "role": "secondary", "peers": [], "remote_active": false });
        assert_eq!(status_text(&v), "role: secondary\nconnected: no\npeers: \nactive: elsewhere\nlocked: no\n");
    }
}
