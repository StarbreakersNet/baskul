//! Saisie d'un raccourci depuis l'app de réglages : sur la machine
//! principale, la prochaine combinaison tapée est relevée par le crochet
//! clavier de Bascule. Elle ne déclenche alors aucun raccourci et n'arrive pas
//! aux applications, et les touches que la fenêtre de réglages ne reçoit pas
//! (F11, F12…) fonctionnent aussi.

#[cfg(not(test))]
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Abandon automatique d'une saisie restée sans réponse.
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Idle,
    Waiting,
    /// Combinaison relevée, au format de la configuration (« Shift+F11 »).
    Done(String),
}

#[derive(Debug)]
struct Inner {
    supported: bool,
    state: State,
    deadline: Option<Instant>,
}

const IDLE: Inner = Inner { supported: false, state: State::Idle, deadline: None };

#[cfg(not(test))]
static INNER: Mutex<Inner> = Mutex::new(IDLE);

#[cfg(not(test))]
fn with<R>(f: impl FnOnce(&mut Inner) -> R) -> R {
    f(&mut INNER.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Tests : un état par thread, les tests du clavier tournant en parallèle.
#[cfg(test)]
fn with<R>(f: impl FnOnce(&mut Inner) -> R) -> R {
    thread_local!(static INNER: std::cell::RefCell<Inner> = const { std::cell::RefCell::new(IDLE) });
    INNER.with(|i| f(&mut i.borrow_mut()))
}

/// Le crochet clavier de la machine principale est actif.
pub fn set_supported(on: bool) {
    with(|i| i.supported = on);
}

/// Commence une saisie. `false` si cette machine ne peut pas la faire.
pub fn start() -> bool {
    with(|i| {
        if i.supported {
            i.state = State::Waiting;
            i.deadline = Some(Instant::now() + TIMEOUT);
        }
        i.supported
    })
}

pub fn cancel() {
    with(|i| {
        i.state = State::Idle;
        i.deadline = None;
    });
}

fn expire(i: &mut Inner) {
    if i.state == State::Waiting && i.deadline.is_some_and(|d| Instant::now() > d) {
        i.state = State::Idle;
    }
}

/// Une saisie attend une combinaison (appelé à chaque touche enfoncée).
pub fn is_waiting() -> bool {
    with(|i| {
        expire(i);
        i.state == State::Waiting
    })
}

/// Combinaison relevée par le crochet clavier.
pub fn finish(hotkey: String) {
    with(|i| {
        if i.state == State::Waiting {
            i.state = State::Done(hotkey);
        }
    });
}

/// État de la saisie ; un résultat n'est rendu qu'une fois.
pub fn poll() -> State {
    with(|i| {
        expire(i);
        let state = i.state.clone();
        if matches!(state, State::Done(_)) {
            i.state = State::Idle;
        }
        state
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_cycle() {
        set_supported(false);
        assert!(!start());
        set_supported(true);
        assert!(start());
        assert!(is_waiting());
        assert_eq!(poll(), State::Waiting);
        finish("Shift+F11".into());
        assert!(!is_waiting());
        assert_eq!(poll(), State::Done("Shift+F11".into()));
        assert_eq!(poll(), State::Idle);
        // Un résultat sans saisie en cours est ignoré.
        finish("F1".into());
        assert_eq!(poll(), State::Idle);
        assert!(start());
        cancel();
        assert_eq!(poll(), State::Idle);
    }
}
