//! Export et import des réglages dans un fichier JSON, pour les reprendre sur
//! une autre machine ou après une réinstallation.

use crate::config::Config;
use crate::profiles::{self, Profile};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Export {
    /// Version de Baskul qui a écrit le fichier.
    baskul: String,
    config: Config,
    #[serde(default)]
    profiles: Vec<Profile>,
}

pub fn export(config_path: &Path, out: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(config_path)
        .map_err(|e| format!("impossible de lire {} : {e}", config_path.display()))?;
    let export = Export {
        baskul: crate::VERSION.to_string(),
        config: Config::parse(&text)?,
        profiles: profiles::read(config_path),
    };
    let json = serde_json::to_string_pretty(&export).map_err(|e| format!("export : {e}"))?;
    std::fs::write(out, json).map_err(|e| format!("écriture de {} impossible : {e}", out.display()))?;
    log::info!("réglages exportés dans {}", out.display());
    Ok(())
}

/// Ce que la machine garde du sien : son rôle, son nom, ses adresses, sa carte réseau et
/// sa grille. Tout le reste vient du fichier, secret partagé compris.
fn merged(current: &Config, imported: Config) -> Config {
    let mut cfg = imported;
    cfg.role = current.role;
    cfg.name = current.name.clone();
    cfg.network.connect = current.network.connect.clone();
    cfg.network.listen = current.network.listen.clone();
    cfg.network.interface = current.network.interface.clone();
    cfg.layout = current.layout.clone();
    cfg
}

/// Écrit la configuration et les profils du fichier ; Baskul doit ensuite redémarrer.
pub fn import(config_path: &Path, file: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("impossible de lire {} : {e}", file.display()))?;
    let export: Export =
        serde_json::from_str(&text).map_err(|e| format!("{} n'est pas un export de Baskul : {e}", file.display()))?;
    let current = Config::parse(
        &std::fs::read_to_string(config_path)
            .map_err(|e| format!("impossible de lire {} : {e}", config_path.display()))?,
    )?;
    crate::config::write_config(config_path, &merged(&current, export.config))?;
    let mut list = profiles::read(config_path);
    for profile in export.profiles {
        list.retain(|p| p.name != profile.name);
        list.push(profile);
    }
    profiles::write(config_path, &list)?;
    log::info!("réglages importés de {} (exportés par Baskul {})", file.display(), export.baskul);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_then_import_keeps_machine_identity() {
        let dir = std::env::temp_dir().join(format!("baskul-transfer-{}", std::process::id()));
        let (a, b) = (dir.join("a"), dir.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let source = a.join("config.toml");
        let target = b.join("config.toml");
        std::fs::write(&source, crate::config::default_config_text("primary", "secret-de-la-premiere-machine"))
            .unwrap();
        std::fs::write(&target, crate::config::default_config_text("secondary", "autre-secret-assez-long")).unwrap();

        let mut cfg = Config::parse(&std::fs::read_to_string(&source).unwrap()).unwrap();
        cfg.mouse.speed = 3.0;
        cfg.name = "Principale".into();
        crate::config::write_config(&source, &cfg).unwrap();
        profiles::save_current(&source, "Jeu").unwrap();

        let file = dir.join("baskul.json");
        export(&source, &file).unwrap();
        import(&target, &file).unwrap();

        let imported = Config::parse(&std::fs::read_to_string(&target).unwrap()).unwrap();
        assert_eq!(imported.mouse.speed, 3.0);
        assert_eq!(imported.network.secret, "secret-de-la-premiere-machine");
        assert_eq!(imported.role, crate::config::RoleCfg::Secondary);
        assert_ne!(imported.name, "Principale");
        assert_eq!(profiles::read(&target).len(), 1);
        assert!(import(&target, &source).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
