//! Activité récente (connexions, bascules, verrouillage…), déduite des
//! changements d'état ([`crate::status`]) : pour la fenêtre de réglages et
//! les notifications du système.

use crate::status::Snapshot;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

pub const MAX_EVENTS: usize = 100;
/// Au-delà de ce silence, une suite d'événements semblables est close.
const RUN_GAP_MS: u64 = 30 * 60 * 1000;
/// Une reconnexion aussi rapide remplace la déconnexion.
const FLAP_MS: u64 = 30 * 1000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Kind {
    Started {
        /// Redémarrage (réglages enregistrés, profil, mise à jour) plutôt que lancement.
        #[serde(default)]
        restarted: bool,
    },
    Connected {
        peer: String,
    },
    Disconnected {
        peer: String,
    },
    /// Déconnexion suivie d'une reconnexion en moins de [`FLAP_MS`].
    Reconnected {
        peer: String,
        /// Durée de la coupure (ms).
        gap: u64,
    },
    SwitchedTo {
        peer: String,
    },
    SwitchedHere,
    Locked,
    Unlocked,
    Warning {
        text: String,
    },
}

impl Kind {
    /// Événements qui se regroupent quand ils se suivent.
    fn run_key(&self) -> Option<String> {
        Some(match self {
            Kind::Started { .. } => "started".into(),
            Kind::SwitchedTo { .. } | Kind::SwitchedHere => "switch".into(),
            Kind::Locked | Kind::Unlocked => "lock".into(),
            Kind::Reconnected { peer, .. } => format!("reconnected:{peer}"),
            Kind::Warning { text } => format!("warning:{text}"),
            Kind::Connected { .. } | Kind::Disconnected { .. } => return None,
        })
    }

    /// Machine concernée, gardée dans [`Event::names`] d'une suite.
    fn name(&self) -> Option<&str> {
        match self {
            Kind::SwitchedTo { peer } | Kind::Reconnected { peer, .. } => Some(peer),
            _ => None,
        }
    }
}

/// Un événement, ou une suite d'événements semblables regroupés.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    /// Première occurrence (ms depuis 1970).
    pub at: u64,
    /// Dernière occurrence (ms depuis 1970).
    #[serde(default)]
    pub until: u64,
    #[serde(default = "one")]
    pub count: u32,
    /// Machines concernées par une suite (bascules, reconnexions).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<String>,
    /// Le plus récent de la suite.
    #[serde(flatten)]
    pub kind: Kind,
}

fn one() -> u32 {
    1
}

struct State {
    events: VecDeque<Event>,
    next_id: u64,
}

static STATE: Mutex<State> = Mutex::new(State { events: VecDeque::new(), next_id: 1 });
static REVISION: AtomicU64 = AtomicU64::new(0);

type Listener = Box<dyn Fn(&Kind) + Send + Sync>;
static LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Ajoute `kind` à l'activité, en le regroupant si possible (voir [`add`]).
pub fn record(kind: Kind) {
    {
        let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let State { events, next_id } = &mut *s;
        add(events, next_id, kind.clone(), now_ms());
    }
    REVISION.fetch_add(1, Ordering::AcqRel);
    for listener in LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        listener(&kind);
    }
}

/// Une reconnexion rapide remplace la déconnexion ; un événement semblable au
/// dernier enregistré (même suite, moins de [`RUN_GAP_MS`] après lui) le complète
/// au lieu de s'ajouter.
fn add(events: &mut VecDeque<Event>, next_id: &mut u64, kind: Kind, now: u64) {
    let kind = match kind {
        Kind::Connected { peer } => match recent_disconnection(events, &peer, now) {
            Some(index) => {
                let gap = events.remove(index).map_or(0, |e| now.saturating_sub(e.at));
                Kind::Reconnected { peer, gap }
            }
            None => Kind::Connected { peer },
        },
        other => other,
    };
    if let Some(last) = events.back_mut() {
        let same_run = kind.run_key().is_some() && kind.run_key() == last.kind.run_key();
        if same_run && now.saturating_sub(last.until.max(last.at)) < RUN_GAP_MS {
            last.count += 1;
            last.until = now;
            if let Some(name) = kind.name() {
                if !last.names.iter().any(|n| n == name) {
                    last.names.push(name.to_string());
                }
            }
            last.kind = kind;
            return;
        }
    }
    let names = kind.name().map(|n| vec![n.to_string()]).unwrap_or_default();
    events.push_back(Event { id: *next_id, at: now, until: now, count: 1, names, kind });
    *next_id += 1;
    while events.len() > MAX_EVENTS {
        events.pop_front();
    }
}

/// Déconnexion de `peer` il y a moins de [`FLAP_MS`].
fn recent_disconnection(events: &VecDeque<Event>, peer: &str, now: u64) -> Option<usize> {
    events
        .iter()
        .rposition(|e| matches!(&e.kind, Kind::Disconnected { peer: p } if p == peer))
        .filter(|&i| now.saturating_sub(events[i].at) < FLAP_MS)
}

/// Du plus ancien au plus récent.
pub fn list() -> Vec<Event> {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).events.iter().cloned().collect()
}

/// Change à chaque modification de l'activité : la fenêtre ne la relit que s'il change.
pub fn revision() -> u64 {
    REVISION.load(Ordering::Acquire)
}

/// Reprend l'activité gardée avant un redémarrage, sans prévenir personne.
pub fn restore(events: Vec<Event>) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let skip = events.len().saturating_sub(MAX_EVENTS);
    s.events = events.into_iter().skip(skip).collect();
    s.next_id = s.events.iter().map(|e| e.id).max().map_or(1, |id| id + 1);
    REVISION.fetch_add(1, Ordering::AcqRel);
}

/// Efface l'activité ; les identifiants continuent de croître.
pub fn clear() {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).events.clear();
    REVISION.fetch_add(1, Ordering::AcqRel);
}

/// Appelée à chaque événement, avant regroupement (une connexion reste une
/// connexion), sur le thread qui l'a produit (parfois celui des crochets
/// clavier et souris) : elle doit rendre la main vite.
pub fn on_event(f: impl Fn(&Kind) + Send + Sync + 'static) {
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

    fn feed(kinds: &[(Kind, u64)]) -> Vec<Event> {
        let mut events = VecDeque::new();
        let mut next_id = 1;
        for (kind, at) in kinds {
            add(&mut events, &mut next_id, kind.clone(), *at);
        }
        events.into()
    }

    fn to(peer: &str) -> Kind {
        Kind::SwitchedTo { peer: peer.into() }
    }

    #[test]
    fn switches_in_a_row_are_one_entry() {
        let events = feed(&[(to("Mac"), 0), (Kind::SwitchedHere, 1_000), (to("Salon"), 2_000), (to("Mac"), 3_000)]);
        assert_eq!(events.len(), 1);
        assert_eq!((events[0].count, events[0].at, events[0].until), (4, 0, 3_000));
        assert_eq!(events[0].names, ["Mac", "Salon"]);
    }

    #[test]
    fn a_run_ends_with_another_event_or_a_long_silence() {
        let events = feed(&[
            (to("Mac"), 0),
            (Kind::Locked, 1_000),
            (Kind::Unlocked, 2_000),
            (to("Mac"), 3_000),
            (to("Mac"), 3_000 + RUN_GAP_MS),
        ]);
        let counts: Vec<u32> = events.iter().map(|e| e.count).collect();
        assert_eq!(counts, [1, 2, 1, 1]);
    }

    #[test]
    fn a_quick_reconnection_replaces_the_disconnection() {
        let mac = || Kind::Disconnected { peer: "Mac".into() };
        let back = || Kind::Connected { peer: "Mac".into() };
        let events = feed(&[(mac(), 0), (Kind::SwitchedHere, 10), (back(), 4_000), (mac(), 60_000), (back(), 62_000)]);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, Kind::SwitchedHere);
        assert_eq!(events[1].kind, Kind::Reconnected { peer: "Mac".into(), gap: 2_000 });
        assert_eq!(events[1].count, 2);
        // Coupure trop longue : déconnexion et connexion restent
        assert_eq!(feed(&[(mac(), 0), (back(), FLAP_MS)]).len(), 2);
    }

    #[test]
    fn restarts_in_a_row_are_one_entry() {
        let start = |restarted| Kind::Started { restarted };
        let events = feed(&[(start(false), 0), (start(true), 5_000), (start(true), 9_000)]);
        assert_eq!((events.len(), events[0].count), (1, 3));
    }

    #[test]
    fn serialized_flat_and_compatible() {
        let e = Event { id: 3, at: 10, until: 10, count: 1, names: Vec::new(), kind: to("Mac") };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "id": 3, "at": 10, "until": 10, "count": 1, "kind": "switchedTo", "peer": "Mac" })
        );
        assert_eq!(serde_json::from_value::<Event>(json).unwrap(), e);
        // activity.json écrit avant les regroupements
        let old: Event = serde_json::from_str(r#"{ "id": 1, "at": 5, "kind": "started" }"#).unwrap();
        assert_eq!((old.count, old.kind), (1, Kind::Started { restarted: false }));
    }
}
