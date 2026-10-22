//! État partagé affiché par l'icône (barre des tâches / barre des menus).

use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq)]
pub enum Conn {
    /// En attente d'une connexion (écoute ou tentative de connexion).
    Waiting(String),
    Connected(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub conn: Conn,
    /// Le curseur est sur l'autre machine (machine principale) ou sur cette
    /// machine (machine pilotée).
    pub remote_active: bool,
    pub locked: bool,
    /// Avertissement persistant (permissions manquantes…).
    pub warning: Option<String>,
    /// Autre machine, quand la connexion est établie.
    pub peer: Option<Peer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Peer {
    /// Système de l'autre machine (`macos`, `windows`, `linux`).
    pub os: String,
    /// Début de la connexion (ms depuis 1970).
    pub since: u64,
}

impl Snapshot {
    /// Résumé d'une ligne pour l'info-bulle / le menu.
    pub fn summary(&self) -> String {
        let mut s = match &self.conn {
            Conn::Waiting(w) => w.clone(),
            Conn::Connected(peer) => format!("Connecté à {peer}"),
            Conn::Error(e) => format!("Erreur : {e}"),
        };
        if self.locked {
            s.push_str(" — curseur verrouillé");
        }
        if let Some(w) = &self.warning {
            s.push_str(" — ");
            s.push_str(w);
        }
        s
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.conn, Conn::Connected(_))
    }
}

static STATE: Mutex<Snapshot> = Mutex::new(Snapshot {
    conn: Conn::Waiting(String::new()),
    remote_active: false,
    locked: false,
    warning: None,
    peer: None,
});

type Listener = Box<dyn Fn() + Send + Sync>;
static LISTENER: OnceLock<Listener> = OnceLock::new();

fn update(f: impl FnOnce(&mut Snapshot)) {
    let changed = {
        let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let before = s.clone();
        f(&mut s);
        *s != before
    };
    if changed {
        if let Some(l) = LISTENER.get() {
            l();
        }
    }
}

/// État de la connexion (hors connexion établie : voir `set_connected`).
pub fn set_conn(c: Conn) {
    update(|s| {
        if !matches!(c, Conn::Connected(_)) {
            s.peer = None;
        }
        s.conn = c;
    });
}

/// Connexion établie avec l'autre machine.
pub fn set_connected(name: &str, os: &str) {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
    update(|s| {
        s.conn = Conn::Connected(name.to_string());
        s.peer = Some(Peer { os: os.to_string(), since });
    });
}

pub fn set_remote_active(active: bool) {
    update(|s| s.remote_active = active);
}

pub fn set_locked(locked: bool) {
    update(|s| s.locked = locked);
}

pub fn set_warning(w: Option<String>) {
    update(|s| s.warning = w);
}

pub fn snapshot() -> Snapshot {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Enregistre la fonction appelée à chaque changement (une seule fois).
pub fn on_change(f: impl Fn() + Send + Sync + 'static) {
    let _ = LISTENER.set(Box::new(f));
}
