//! Position et taille de la fenêtre de réglages, retrouvées d'une ouverture
//! et d'un lancement à l'autre (greffon `window-state` de Tauri).
//!
//! Le greffon n'écrit son fichier qu'à la fin normale de l'app, or Baskul se
//! termine par `std::process::exit` (redémarrage, mise à jour, "Quitter") :
//! la fenêtre est donc enregistrée ici dès qu'elle cesse de bouger, et à sa
//! fermeture.

use std::sync::OnceLock;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

/// Calme après le dernier déplacement avant d'enregistrer.
const QUIET: Duration = Duration::from_millis(500);

static CHANGES: OnceLock<Sender<()>> = OnceLock::new();

fn flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_window_state::Builder::default().with_state_flags(flags()).build()
}

/// Lance l'enregistrement de la fenêtre, à chaque [`changed`].
pub fn start(app: &AppHandle) {
    let (tx, rx) = mpsc::channel::<()>();
    if CHANGES.set(tx).is_err() {
        return;
    }
    let app = app.clone();
    let spawned = std::thread::Builder::new().name("baskul-window-state".into()).spawn(move || {
        while rx.recv().is_ok() {
            if !wait_for_quiet(&rx) {
                return;
            }
            if let Err(e) = app.save_window_state(flags()) {
                log::warn!("enregistrement de la position de la fenêtre impossible : {e}");
            }
        }
    });
    if let Err(e) = spawned {
        log::warn!("enregistrement de la position de la fenêtre impossible : {e}");
    }
}

/// Attend que la fenêtre ne bouge plus ; `false` si l'app se ferme.
fn wait_for_quiet(rx: &mpsc::Receiver<()>) -> bool {
    loop {
        match rx.recv_timeout(QUIET) {
            Ok(()) => {}
            Err(RecvTimeoutError::Timeout) => return true,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

/// La fenêtre a bougé, changé de taille ou s'est fermée.
pub fn changed() {
    if let Some(tx) = CHANGES.get() {
        let _ = tx.send(());
    }
}
