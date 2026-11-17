//! Profils de réglages, gardés dans `profiles.json` à côté de `config.toml` :
//! les réglages d'usage (souris, défilement, clavier, presse-papiers, bords
//! de l'écran…) sous un nom, pour passer de l'un à l'autre d'un clic. Le
//! rôle, le nom, la connexion, la grille et les raccourcis restent ceux de
//! la configuration.

use crate::config::Config;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const FILE: &str = "profiles.json";

/// Noms des profils et profil actif, pour le menu de l'icône (relus après chaque changement).
static MENU: Mutex<(Vec<String>, Option<String>)> = Mutex::new((Vec::new(), None));

pub fn menu() -> (Vec<String>, Option<String>) {
    MENU.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Relit les profils pour le menu de l'icône.
pub fn refresh_menu(config_path: &Path) {
    let profiles = read(config_path);
    let active = load_config(config_path).ok().and_then(|cfg| active(&profiles, &cfg));
    *MENU.lock().unwrap_or_else(|e| e.into_inner()) = (profiles.into_iter().map(|p| p.name).collect(), active);
    crate::tray::changed();
}

/// Sections de la configuration qu'un profil retient.
pub const SECTIONS: [&str; 7] = ["screen", "mouse", "scroll", "buttons", "keyboard", "clipboard", "activity"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    /// Sections de [`SECTIONS`], telles qu'enregistrées dans la configuration.
    pub settings: Map<String, Value>,
}

pub fn file(config_path: &Path) -> PathBuf {
    config_path.parent().unwrap_or(Path::new(".")).join(FILE)
}

pub fn read(config_path: &Path) -> Vec<Profile> {
    let path = file(config_path);
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log::warn!("profils illisibles ({}) : {e}", path.display());
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

fn write(config_path: &Path, profiles: &[Profile]) -> Result<(), String> {
    let path = file(config_path);
    let text = serde_json::to_string_pretty(profiles).map_err(|e| format!("profils : {e}"))?;
    let tmp = path.with_extension("json.tmp");
    let err = |e: std::io::Error| format!("écriture de {} impossible : {e}", path.display());
    std::fs::write(&tmp, text).map_err(err)?;
    std::fs::rename(&tmp, &path).map_err(err)
}

fn load_config(config_path: &Path) -> Result<Config, String> {
    let text = std::fs::read_to_string(config_path)
        .map_err(|e| format!("impossible de lire {} : {e}", config_path.display()))?;
    Config::parse(&text)
}

/// Sections de [`SECTIONS`] d'une configuration.
pub fn settings_of(cfg: &Config) -> Result<Map<String, Value>, String> {
    let Value::Object(all) = serde_json::to_value(cfg).map_err(|e| e.to_string())? else {
        return Err("configuration : objet attendu".into());
    };
    Ok(all.into_iter().filter(|(k, _)| SECTIONS.contains(&k.as_str())).collect())
}

/// `cfg` avec les sections du profil.
pub fn with_settings(cfg: &Config, settings: &Map<String, Value>) -> Result<Config, String> {
    let Value::Object(mut all) = serde_json::to_value(cfg).map_err(|e| e.to_string())? else {
        return Err("configuration : objet attendu".into());
    };
    for (key, value) in settings {
        if SECTIONS.contains(&key.as_str()) {
            all.insert(key.clone(), value.clone());
        }
    }
    serde_json::from_value(Value::Object(all)).map_err(|e| format!("profil invalide : {e}"))
}

/// Nom du profil identique aux réglages actuels, s'il y en a un.
pub fn active(profiles: &[Profile], cfg: &Config) -> Option<String> {
    let current = settings_of(cfg).ok()?;
    profiles.iter().find(|p| p.settings == current).map(|p| p.name.clone())
}

/// Liste des profils et profil actif.
pub fn info(config_path: &Path) -> Value {
    let profiles = read(config_path);
    let active = load_config(config_path).ok().and_then(|cfg| active(&profiles, &cfg));
    serde_json::json!({
        "profiles": profiles.iter().map(|p| &p.name).collect::<Vec<_>>(),
        "active": active,
    })
}

/// Enregistre les réglages actuels sous `name` (remplace un profil du même nom).
pub fn save_current(config_path: &Path, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("nom de profil vide".into());
    }
    let settings = settings_of(&load_config(config_path)?)?;
    let mut profiles = read(config_path);
    match profiles.iter_mut().find(|p| p.name == name) {
        Some(p) => p.settings = settings,
        None => profiles.push(Profile { name: name.to_string(), settings }),
    }
    write(config_path, &profiles)?;
    log::info!("profil enregistré : {name}");
    refresh_menu(config_path);
    Ok(())
}

pub fn delete(config_path: &Path, name: &str) -> Result<(), String> {
    let mut profiles = read(config_path);
    profiles.retain(|p| p.name != name);
    write(config_path, &profiles)?;
    refresh_menu(config_path);
    Ok(())
}

/// Écrit les réglages du profil dans la configuration ; Baskul doit ensuite
/// redémarrer pour les appliquer.
pub fn apply(config_path: &Path, name: &str) -> Result<(), String> {
    let profiles = read(config_path);
    let profile = profiles.iter().find(|p| p.name == name).ok_or_else(|| format!("profil inconnu : {name}"))?;
    let cfg = with_settings(&load_config(config_path)?, &profile.settings)?;
    crate::config::write_config(config_path, &cfg)?;
    log::info!("profil appliqué : {name}");
    Ok(())
}

/// Choix du menu de l'icône : applique le profil puis redémarre.
pub fn apply_and_restart(config_path: &Path, name: &str) {
    match apply(config_path, name) {
        Ok(()) => crate::api::request_restart(),
        Err(e) => log::warn!("{e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::parse(&crate::config::default_config_text("primary", "secret-assez-long-pour-valider")).unwrap()
    }

    #[test]
    fn profile_keeps_only_its_sections() {
        let mut cfg = config();
        cfg.mouse.speed = 2.5;
        let settings = settings_of(&cfg).unwrap();
        assert!(settings.contains_key("mouse"));
        assert!(!settings.contains_key("network"));
        assert!(!settings.contains_key("hotkeys"));

        let mut other = config();
        other.name = "Atelier".into();
        let applied = with_settings(&other, &settings).unwrap();
        assert_eq!(applied.mouse.speed, 2.5);
        assert_eq!(applied.name, "Atelier");
    }

    #[test]
    fn active_profile_matches_current_settings() {
        let cfg = config();
        let mut slow = cfg.clone();
        slow.mouse.speed = 0.5;
        let profiles = vec![
            Profile { name: "Lent".into(), settings: settings_of(&slow).unwrap() },
            Profile { name: "Normal".into(), settings: settings_of(&cfg).unwrap() },
        ];
        assert_eq!(active(&profiles, &cfg), Some("Normal".into()));
        assert_eq!(active(&profiles, &slow), Some("Lent".into()));
    }

    #[test]
    fn save_apply_delete() {
        let dir = std::env::temp_dir().join(format!("baskul-profiles-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, crate::config::default_config_text("primary", "secret-assez-long-pour-valider")).unwrap();
        save_current(&path, "Bureau").unwrap();
        let mut cfg = load_config(&path).unwrap();
        cfg.screen.edge_switching = false;
        crate::config::write_config(&path, &cfg).unwrap();
        save_current(&path, "Jeu").unwrap();
        assert_eq!(info(&path)["active"], "Jeu");
        apply(&path, "Bureau").unwrap();
        assert!(load_config(&path).unwrap().screen.edge_switching);
        assert_eq!(info(&path)["active"], "Bureau");
        delete(&path, "Jeu").unwrap();
        assert_eq!(info(&path)["profiles"], serde_json::json!(["Bureau"]));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
