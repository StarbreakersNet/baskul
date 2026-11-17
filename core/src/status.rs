//! État partagé affiché par l'icône (barre des tâches / barre des menus), le
//! menu et la fenêtre de réglages.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, PartialEq)]
pub enum Conn {
    /// En attente (écoute, recherche ou tentative de connexion).
    Waiting(String),
    /// Au moins une autre machine connectée (résumé : leurs noms).
    Connected(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub conn: Conn,
    /// Machine principale : le clavier et la souris agissent sur une pilotée.
    /// Machine pilotée : ils agissent sur cette machine.
    pub remote_active: bool,
    /// Machine principale : pilotée où agissent le clavier et la souris.
    pub active_peer: Option<String>,
    pub locked: bool,
    /// Avertissement persistant (permissions manquantes…).
    pub warning: Option<String>,
    /// Autres machines connectées, dans l'ordre de connexion.
    pub peers: Vec<Peer>,
    /// Programme de la liste `[screen] pause_edges_for` au premier plan : la
    /// bascule par les bords est suspendue.
    pub edges_paused_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Peer {
    pub name: String,
    /// Système de l'autre machine (`macos`, `windows`, `linux`).
    pub os: String,
    /// Début de la connexion (ms depuis 1970).
    pub since: u64,
}

impl Snapshot {
    /// Résumé d'une ligne pour l'info-bulle / le menu.
    pub fn summary(&self) -> String {
        use crate::i18n::{t, tf};
        let mut s = match &self.conn {
            Conn::Waiting(w) => w.clone(),
            Conn::Connected(peers) => tf("status.connected", &[("peer", peers)]),
            Conn::Error(e) => tf("status.error", &[("error", e)]),
        };
        if self.locked {
            s.push_str(" — ");
            s.push_str(&t("status.locked"));
        }
        if let Some(program) = &self.edges_paused_by {
            s.push_str(" — ");
            s.push_str(&tf("status.edgesPaused", &[("program", program)]));
        }
        if let Some(w) = &self.warning {
            s.push_str(" — ");
            s.push_str(w);
        }
        s
    }

    pub fn is_connected(&self) -> bool {
        !self.peers.is_empty()
    }
}

struct State {
    snap: Snapshot,
    idle: Conn,
}

static STATE: Mutex<State> = Mutex::new(State {
    snap: Snapshot {
        conn: Conn::Waiting(String::new()),
        remote_active: false,
        active_peer: None,
        locked: false,
        warning: None,
        peers: Vec::new(),
        edges_paused_by: None,
    },
    idle: Conn::Waiting(String::new()),
});

type Listener = Box<dyn Fn() + Send + Sync>;
static LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());

fn update(f: impl FnOnce(&mut State)) {
    let (before, after) = {
        let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let before = s.snap.clone();
        f(&mut s);
        s.snap.conn = if s.snap.peers.is_empty() {
            s.idle.clone()
        } else {
            Conn::Connected(s.snap.peers.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "))
        };
        (before, s.snap.clone())
    };
    if before != after {
        for kind in crate::activity::changes(&before, &after) {
            crate::activity::record(kind);
        }
        for l in LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).iter() {
            l();
        }
    }
}

/// État hors connexion (attente, erreur), affiché tant qu'aucune machine
/// n'est connectée.
pub fn set_conn(c: Conn) {
    update(|s| s.idle = c);
}

pub fn add_peer(name: &str, os: &str) {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
    update(|s| {
        s.snap.peers.retain(|p| p.name != name);
        s.snap.peers.push(Peer { name: name.to_string(), os: os.to_string(), since });
    });
}

pub fn remove_peer(name: &str) {
    update(|s| s.snap.peers.retain(|p| p.name != name));
}

pub fn set_remote_active(active: bool) {
    update(|s| s.snap.remote_active = active);
}

pub fn set_active_peer(name: Option<&str>) {
    update(|s| {
        s.snap.remote_active = name.is_some();
        s.snap.active_peer = name.map(str::to_string);
    });
}

pub fn set_locked(locked: bool) {
    update(|s| s.snap.locked = locked);
}

pub fn set_warning(w: Option<String>) {
    update(|s| s.snap.warning = w);
}

/// Lu à chaque passage contre un bord : sans verrou.
static EDGES_PAUSED: AtomicBool = AtomicBool::new(false);

/// Programme au premier plan qui suspend la bascule par les bords (`None` : elle reprend).
pub fn set_edges_paused_by(program: Option<String>) {
    EDGES_PAUSED.store(program.is_some(), Ordering::Release);
    update(|s| s.snap.edges_paused_by = program);
}

pub fn edges_paused() -> bool {
    EDGES_PAUSED.load(Ordering::Acquire)
}

pub fn snapshot() -> Snapshot {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).snap.clone()
}

pub fn on_change(f: impl Fn() + Send + Sync + 'static) {
    LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(f));
}
