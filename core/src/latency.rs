//! Temps d'aller-retour mesurés avec chaque machine connectée (`Probe` toutes
//! les deux secondes, voir `net.rs`), pour le diagnostic réseau de la fenêtre.

use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Trois minutes de mesures, à une toutes les deux secondes.
pub const MAX_SAMPLES: usize = 90;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Sample {
    /// Date (ms depuis 1970).
    pub at: u64,
    /// Aller-retour, en microsecondes.
    pub rtt_us: u64,
}

static SAMPLES: Mutex<Option<HashMap<String, VecDeque<Sample>>>> = Mutex::new(None);
static START: OnceLock<Instant> = OnceLock::new();

/// Horloge monotone du processus, en microsecondes : seule la machine qui
/// envoie un `Probe` la lit, l'autre la lui renvoie telle quelle.
pub fn clock_us() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_micros() as u64
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Réponse à un `Probe` envoyé à `sent` (voir [`clock_us`]).
pub fn record(peer: &str, sent: u64) {
    let sample = Sample { at: now_ms(), rtt_us: clock_us().saturating_sub(sent) };
    let mut guard = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
    let samples = guard.get_or_insert_with(HashMap::new).entry(peer.to_string()).or_default();
    samples.push_back(sample);
    while samples.len() > MAX_SAMPLES {
        samples.pop_front();
    }
}

pub fn forget(peer: &str) {
    if let Some(map) = SAMPLES.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        map.remove(peer);
    }
}

/// Mesures de chaque machine, de la plus ancienne à la plus récente.
pub fn snapshot() -> Vec<(String, Vec<Sample>)> {
    let guard = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
    let mut list: Vec<_> = guard
        .iter()
        .flat_map(|map| map.iter())
        .map(|(name, samples)| (name.clone(), samples.iter().copied().collect()))
        .collect();
    list.sort_by(|a, b| a.0.cmp(&b.0));
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_last_samples() {
        let peer = "latence-test";
        for _ in 0..MAX_SAMPLES + 5 {
            record(peer, clock_us());
        }
        let samples = snapshot().into_iter().find(|(n, _)| n == peer).unwrap().1;
        assert_eq!(samples.len(), MAX_SAMPLES);
        forget(peer);
        assert!(snapshot().iter().all(|(n, _)| n != peer));
    }
}
