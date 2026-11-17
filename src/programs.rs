//! Programmes qui suspendent la bascule par les bords tant qu'ils sont au
//! premier plan (`[screen] pause_edges_for`) : jeux qui capturent la souris…

use crate::{platform, status};
use std::sync::Mutex;
use std::time::Duration;

const POLL: Duration = Duration::from_millis(500);

/// Dernier programme vu au premier plan (hors Baskul) : la fenêtre de réglages
/// propose de l'ajouter à la liste.
static LAST_FOREGROUND: Mutex<Option<String>> = Mutex::new(None);

pub fn last_foreground() -> Option<String> {
    LAST_FOREGROUND.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn start(list: Vec<String>) {
    let spawned = std::thread::Builder::new().name("baskul-programs".into()).spawn(move || {
        let mut paused_by: Option<String> = None;
        loop {
            let names = platform::foreground_programs();
            if let Some(first) = names.first() {
                *LAST_FOREGROUND.lock().unwrap_or_else(|e| e.into_inner()) = Some(first.clone());
            }
            let hit = matching(&list, &names);
            if hit != paused_by {
                match &hit {
                    Some(program) => log::info!("bascule par les bords suspendue : {program} au premier plan"),
                    None => log::info!("bascule par les bords rétablie"),
                }
                status::set_edges_paused_by(hit.clone());
                paused_by = hit;
            }
            std::thread::sleep(POLL);
        }
    });
    if let Err(e) = spawned {
        log::warn!("surveillance des programmes au premier plan impossible : {e}");
    }
}

/// Entrée de la liste qui désigne l'un des noms du programme au premier plan.
fn matching(list: &[String], names: &[String]) -> Option<String> {
    list.iter().find(|entry| names.iter().any(|name| normalize(entry) == normalize(name))).cloned()
}

/// Sans majuscules ni extension d'exécutable : "Wow", "wow.exe" et "WoW.EXE" se valent.
fn normalize(name: &str) -> String {
    let name = name.trim().to_lowercase();
    [".exe", ".app"].iter().find_map(|ext| name.strip_suffix(ext)).map_or(name.clone(), str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_without_case_or_extension() {
        let list = vec!["Wow".to_string(), "Moon Sweeper".to_string()];
        assert_eq!(matching(&list, &["WoW.exe".into()]), Some("Wow".into()));
        assert_eq!(matching(&list, &["World of Warcraft".into(), "moon sweeper".into()]), Some("Moon Sweeper".into()));
        assert_eq!(matching(&list, &["notepad.exe".into()]), None);
        assert_eq!(matching(&list, &[]), None);
    }
}
