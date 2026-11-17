//! Ajout d'une machine par code court (voir `baskul_core::pairing`) : la
//! principale ouvre l'appairage et affiche le code ; la pilotée le saisit,
//! reçoit le secret partagé, l'écrit dans sa configuration et redémarre.

use crate::activity;
use crate::config::{Config, RoleCfg, Settings};
use baskul_core::pairing::{self, Invitation};
use serde::Serialize;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum State {
    Idle,
    Waiting {
        code: String,
        /// Fin de validité du code (ms depuis 1970).
        expires: u64,
    },
    Paired {
        name: String,
    },
    Failed {
        reason: String,
    },
}

static STATE: Mutex<State> = Mutex::new(State::Idle);
static CANCEL: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

fn set_state(state: State) {
    *STATE.lock().unwrap_or_else(|e| e.into_inner()) = state;
}

pub fn state() -> State {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn settings(config_path: &Path) -> Result<(Config, Settings), String> {
    let text = std::fs::read_to_string(config_path)
        .map_err(|e| format!("impossible de lire {} : {e}", config_path.display()))?;
    let cfg = Config::parse(&text)?;
    let settings = cfg.validate()?;
    Ok((cfg, settings))
}

pub fn cancel() {
    if let Some(flag) = CANCEL.lock().unwrap_or_else(|e| e.into_inner()).take() {
        flag.store(true, Ordering::Release);
    }
    set_state(State::Idle);
}

/// Principale : nouveau code, valable deux minutes.
pub fn start(config_path: &Path) -> Result<State, String> {
    cancel();
    let (cfg, settings) = settings(config_path)?;
    if cfg.role != RoleCfg::Primary {
        return Err("seule la machine principale ajoute des machines".into());
    }
    let listen: SocketAddr =
        settings.listen.parse().map_err(|e| format!("adresse d'écoute invalide ({}) : {e}", settings.listen))?;
    let addr = SocketAddr::new(listen.ip(), pairing::port(listen.port()));
    let listener = TcpListener::bind(addr).map_err(|e| format!("appairage impossible sur {addr} : {e}"))?;
    let code = pairing::new_code()?;
    let invitation = Invitation { secret: settings.secret.clone(), name: settings.name.clone() };
    let flag = Arc::new(AtomicBool::new(false));
    *CANCEL.lock().unwrap_or_else(|e| e.into_inner()) = Some(flag.clone());
    let expires = now_ms() + pairing::WINDOW.as_millis() as u64;
    let waiting = State::Waiting { code: code.clone(), expires };
    set_state(waiting.clone());
    std::thread::Builder::new()
        .name("baskul-pairing".into())
        .spawn(move || {
            let result = pairing::serve(&listener, &code, &invitation, &flag);
            if !flag.load(Ordering::Acquire) {
                match result {
                    Ok(name) => {
                        activity::record(activity::Kind::Paired { peer: name.clone() });
                        set_state(State::Paired { name });
                    }
                    Err(reason) => set_state(State::Failed { reason }),
                }
            }
        })
        .map_err(|e| format!("appairage : {e}"))?;
    log::info!("appairage ouvert sur {addr} pendant {} s", pairing::WINDOW.as_secs());
    Ok(waiting)
}

/// Adresse d'appairage d'une principale donnée par son adresse (`ip` ou `ip:port`, port d'écoute).
fn pairing_address(address: &str) -> Result<SocketAddr, String> {
    let address = address.trim();
    let with_port = if address.parse::<SocketAddr>().is_ok()
        || address.rsplit_once(':').is_some_and(|(h, p)| !h.contains(':') && p.parse::<u16>().is_ok())
    {
        address.to_string()
    } else {
        format!("{address}:{}", crate::protocol::DEFAULT_PORT)
    };
    let main = with_port
        .to_socket_addrs()
        .map_err(|e| format!("adresse invalide ({address}) : {e}"))?
        .next()
        .ok_or_else(|| format!("adresse introuvable : {address}"))?;
    Ok(SocketAddr::new(main.ip(), pairing::port(main.port())))
}

/// Pilotée : rejoint la principale avec le code et écrit le secret reçu.
/// Renvoie le nom de la principale ; Baskul doit ensuite redémarrer.
pub fn join(config_path: &Path, address: &str, code: &str) -> Result<String, String> {
    let (mut cfg, settings) = settings(config_path)?;
    if cfg.role != RoleCfg::Secondary {
        return Err("seule une machine pilotée rejoint une principale".into());
    }
    let invitation = pairing::join(pairing_address(address)?, code, &settings.name)?;
    cfg.network.secret = invitation.secret;
    if !cfg.network.discovery {
        cfg.network.connect = address.trim().to_string();
    }
    crate::config::write_config(config_path, &cfg)?;
    log::info!("appairage réussi avec la principale {:?}", invitation.name);
    Ok(invitation.name)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_port_follows_the_listening_port() {
        assert_eq!(pairing_address("127.0.0.1:9000").unwrap().port(), 9001);
        assert_eq!(pairing_address("127.0.0.1").unwrap().port(), crate::protocol::DEFAULT_PORT + 1);
    }

    #[test]
    fn a_secondary_joins_its_primary() {
        let dir = std::env::temp_dir().join(format!("baskul-pair-{}", std::process::id()));
        let (a, b) = (dir.join("a"), dir.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let primary = a.join("config.toml");
        let secondary = b.join("config.toml");
        let port = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        let text = crate::config::default_config_text("primary", "secret-de-la-principale").replace(
            &format!("listen = \"0.0.0.0:{}\"", crate::protocol::DEFAULT_PORT),
            &format!("listen = \"127.0.0.1:{}\"", port - 1),
        );
        std::fs::write(&primary, text).unwrap();
        std::fs::write(&secondary, crate::config::default_config_text("secondary", "ancien-secret-pilotee")).unwrap();

        let State::Waiting { code, .. } = start(&primary).unwrap() else { panic!("pas de code") };
        join(&secondary, &format!("127.0.0.1:{}", port - 1), &code).unwrap();
        let cfg = Config::parse(&std::fs::read_to_string(&secondary).unwrap()).unwrap();
        assert_eq!(cfg.network.secret, "secret-de-la-principale");
        cancel();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
