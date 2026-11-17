//! Activité récente (connexions, bascules, verrouillage…), déduite des
//! changements d'état ([`crate::status`]) : pour la fenêtre de réglages et
//! les notifications du système.

use crate::status::Snapshot;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

pub const MAX_EVENTS: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Kind {
    Started,
    Connected { peer: String },
    Disconnected { peer: String },
    SwitchedTo { peer: String },
    SwitchedHere,
    Locked,
    Unlocked,
    Warning { text: String },
    EdgesPaused { program: String },
    EdgesResumed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    /// Date (ms depuis 1970).
    pub at: u64,
    #[serde(flatten)]
    pub kind: Kind,
}

struct State {
    events: VecDeque<Event>,
    next_id: u64,
}

static STATE: Mutex<State> = Mutex::new(State { events: VecDeque::new(), next_id: 1 });

type Listener = Box<dyn Fn(&Event) + Send + Sync>;
static LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

pub fn record(kind: Kind) {
    let event = {
        let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let event = Event { id: s.next_id, at: now_ms(), kind };
        s.next_id += 1;
        s.events.push_back(event.clone());
        while s.events.len() > MAX_EVENTS {
            s.events.pop_front();
        }
        event
    };
    for listener in LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        listener(&event);
    }
}

/// Du plus ancien au plus récent.
pub fn list() -> Vec<Event> {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).events.iter().cloned().collect()
}

/// Identifiant du dernier événement (0 s'il n'y en a pas) : la fenêtre ne
/// relit l'activité que s'il change.
pub fn last_id() -> u64 {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).events.back().map_or(0, |e| e.id)
}

/// Reprend l'activité gardée avant un redémarrage, sans prévenir personne.
pub fn restore(events: Vec<Event>) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let skip = events.len().saturating_sub(MAX_EVENTS);
    s.events = events.into_iter().skip(skip).collect();
    s.next_id = s.events.back().map_or(1, |e| e.id + 1);
}

/// Efface l'activité ; les identifiants continuent de croître.
pub fn clear() {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).events.clear();
}

/// Appelée à chaque nouvel événement, sur le thread qui l'a produit (parfois
/// celui des crochets clavier et souris) : elle doit rendre la main vite.
pub fn on_event(f: impl Fn(&Event) + Send + Sync + 'static) {
    LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(f));
}

/// Événements entre deux états.
pub fn changes(before: &Snapshot, after: &Snapshot) -> Vec<Kind> {
    let mut kinds = Vec::new();
    for peer in &after.peers {
        if !before.peers.iter().any(|p| p.name == peer.name) {
            kinds.push(Kind::Connected { peer: peer.name.clone() });
        }
    }
    for peer in &before.peers {
        if !after.peers.iter().any(|p| p.name == peer.name) {
            kinds.push(Kind::Disconnected { peer: peer.name.clone() });
        }
    }
    if before.active_peer != after.active_peer {
        kinds.push(match &after.active_peer {
            Some(peer) => Kind::SwitchedTo { peer: peer.clone() },
            None => Kind::SwitchedHere,
        });
    } else if before.remote_active != after.remote_active && after.active_peer.is_none() {
        // Machine pilotée : `remote_active` dit que le clavier et la souris sont ici
        if after.remote_active {
            kinds.push(Kind::SwitchedHere);
        } else if let Some(primary) = after.peers.first() {
            kinds.push(Kind::SwitchedTo { peer: primary.name.clone() });
        }
    }
    if before.locked != after.locked {
        kinds.push(if after.locked { Kind::Locked } else { Kind::Unlocked });
    }
    if before.edges_paused_by != after.edges_paused_by {
        kinds.push(match &after.edges_paused_by {
            Some(program) => Kind::EdgesPaused { program: program.clone() },
            None => Kind::EdgesResumed,
        });
    }
    if let Some(text) = &after.warning {
        if before.warning.as_ref() != Some(text) {
            kinds.push(Kind::Warning { text: text.clone() });
        }
    }
    kinds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::{Conn, Peer};

    fn snap() -> Snapshot {
        Snapshot {
            conn: Conn::Waiting(String::new()),
            remote_active: false,
            active_peer: None,
            locked: false,
            warning: None,
            peers: vec![Peer { name: "Mac".into(), os: "macos".into(), since: 0 }],
            edges_paused_by: None,
        }
    }

    #[test]
    fn primary_changes() {
        let before = snap();
        let mut after = snap();
        after.peers.push(Peer { name: "Salon".into(), os: "windows".into(), since: 1 });
        after.active_peer = Some("Salon".into());
        after.remote_active = true;
        after.locked = true;
        assert_eq!(
            changes(&before, &after),
            vec![Kind::Connected { peer: "Salon".into() }, Kind::SwitchedTo { peer: "Salon".into() }, Kind::Locked]
        );
        assert_eq!(
            changes(&after, &before),
            vec![Kind::Disconnected { peer: "Salon".into() }, Kind::SwitchedHere, Kind::Unlocked]
        );
    }

    #[test]
    fn secondary_changes() {
        let before = snap();
        let mut after = snap();
        after.remote_active = true;
        assert_eq!(changes(&before, &after), vec![Kind::SwitchedHere]);
        assert_eq!(changes(&after, &before), vec![Kind::SwitchedTo { peer: "Mac".into() }]);
        // Connexion perdue : pas de bascule vers une machine qui n'est plus là
        let mut gone = before.clone();
        gone.peers.clear();
        assert_eq!(changes(&after, &gone), vec![Kind::Disconnected { peer: "Mac".into() }]);
    }

    #[test]
    fn warning_only_when_new() {
        let mut before = snap();
        before.warning = Some("a".into());
        let after = before.clone();
        assert!(changes(&before, &after).is_empty());
        let mut other = before.clone();
        other.warning = Some("b".into());
        assert_eq!(changes(&before, &other), vec![Kind::Warning { text: "b".into() }]);
    }

    #[test]
    fn edges_paused() {
        let before = snap();
        let mut after = snap();
        after.edges_paused_by = Some("Wow".into());
        assert_eq!(changes(&before, &after), vec![Kind::EdgesPaused { program: "Wow".into() }]);
        assert_eq!(changes(&after, &before), vec![Kind::EdgesResumed]);
    }

    #[test]
    fn serialized_flat() {
        let e = Event { id: 3, at: 10, kind: Kind::SwitchedTo { peer: "Mac".into() } };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json, serde_json::json!({ "id": 3, "at": 10, "kind": "switchedTo", "peer": "Mac" }));
        let back: Event = serde_json::from_value(json).unwrap();
        assert_eq!(back, e);
    }
}
