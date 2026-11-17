//! Activité récente gardée dans `activity.json`, à côté du fichier de
//! configuration : elle survit aux redémarrages (enregistrement des réglages,
//! mise à jour).

use crate::activity::{self, Event};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

const FILE: &str = "activity.json";
/// Plusieurs événements rapprochés (bascule, verrouillage…) : une seule écriture.
const WRITE_DELAY: Duration = Duration::from_millis(500);

static WRITER: OnceLock<Sender<()>> = OnceLock::new();

pub fn file(config_path: &Path) -> PathBuf {
    config_path.parent().unwrap_or(Path::new(".")).join(FILE)
}

/// Reprend l'activité enregistrée, puis l'enregistre à chaque changement, hors
/// du thread qui produit l'événement.
pub fn start(config_path: &Path) {
    let path = file(config_path);
    activity::restore(read(&path));
    let (tx, rx) = mpsc::channel::<()>();
    let spawned = std::thread::Builder::new().name("baskul-activity".into()).spawn(move || {
        while rx.recv().is_ok() {
            std::thread::sleep(WRITE_DELAY);
            while rx.try_recv().is_ok() {}
            if let Err(e) = write(&path, &activity::list()) {
                log::warn!("{e}");
            }
        }
    });
    match spawned {
        Ok(_) => {
            let _ = WRITER.set(tx);
            activity::on_event(|_| save());
        }
        Err(e) => log::warn!("enregistrement de l'activité impossible : {e}"),
    }
}

pub fn clear() {
    activity::clear();
    save();
}

fn save() {
    if let Some(tx) = WRITER.get() {
        let _ = tx.send(());
    }
}

fn read(path: &Path) -> Vec<Event> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log::warn!("activité illisible ({}) : {e}", path.display());
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

fn write(path: &Path, events: &[Event]) -> Result<(), String> {
    let text = serde_json::to_string(events).map_err(|e| format!("activité : {e}"))?;
    let tmp = path.with_extension("json.tmp");
    let err = |e: std::io::Error| format!("écriture de {} impossible : {e}", path.display());
    std::fs::write(&tmp, text).map_err(err)?;
    std::fs::rename(&tmp, path).map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::Kind;

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir().join(format!("baskul-history-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE);
        let events = vec![
            Event { id: 1, at: 5, until: 5, count: 1, names: Vec::new(), kind: Kind::Started { restarted: false } },
            Event {
                id: 2,
                at: 6,
                until: 9,
                count: 3,
                names: vec!["Mac".into()],
                kind: Kind::SwitchedTo { peer: "Mac".into() },
            },
        ];
        write(&path, &events).unwrap();
        assert_eq!(read(&path), events);
        std::fs::write(&path, "pas du json").unwrap();
        assert!(read(&path).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
