//! Découverte automatique (Bonjour / mDNS) : la machine principale s'annonce
//! sur le réseau local, les pilotées la trouvent sans adresse à saisir. Le
//! secret partagé reste vérifié à la connexion.

use crate::protocol::PROTOCOL_VERSION;
use mdns_sd::{IfKind, ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Mutex, OnceLock};

/// Type de service annoncé.
pub const SERVICE: &str = "_baskul._tcp.local.";

/// Machine principale trouvée sur le réseau.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Found {
    pub name: String,
    /// Adresses où la joindre (IPv4 d'abord).
    pub addresses: Vec<SocketAddr>,
    pub os: String,
    /// Version du protocole : une version différente ne peut pas se connecter.
    pub protocol: u32,
}

/// Machines principales trouvées, par nom complet du service.
static FOUND: Mutex<BTreeMap<String, Found>> = Mutex::new(BTreeMap::new());
/// Service mDNS (annonce ou recherche), gardé pour toute la durée du programme.
static DAEMON: OnceLock<ServiceDaemon> = OnceLock::new();

/// Machines principales trouvées sur le réseau local.
pub fn primaries() -> Vec<Found> {
    FOUND.lock().unwrap_or_else(|e| e.into_inner()).values().cloned().collect()
}

fn daemon(interface: Option<&str>) -> Option<&'static ServiceDaemon> {
    if let Some(d) = DAEMON.get() {
        return Some(d);
    }
    let d = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log::warn!("découverte automatique indisponible : {e}");
            return None;
        }
    };
    if let Some(name) = interface {
        // Seulement sur la carte réseau choisie.
        let _ = d.disable_interface(IfKind::All);
        let _ = d.enable_interface(IfKind::Name(name.to_string()));
    }
    Some(DAEMON.get_or_init(|| d))
}

/// Machine principale : annonce `name` sur le port `port`.
pub fn advertise(name: &str, port: u16, interface: Option<&str>) {
    let Some(d) = daemon(interface) else { return };
    let host = format!("{}.local.", host_label(&crate::config::hostname()));
    let props = [
        ("name", name.to_string()),
        ("os", std::env::consts::OS.to_string()),
        ("protocol", PROTOCOL_VERSION.to_string()),
        ("version", crate::VERSION.to_string()),
    ];
    let info = match ServiceInfo::new(SERVICE, &host_label(name), &host, "", port, &props[..]) {
        Ok(i) => i.enable_addr_auto(),
        Err(e) => {
            log::warn!("annonce sur le réseau impossible : {e}");
            return;
        }
    };
    match d.register(info) {
        Ok(()) => log::info!("annoncée sur le réseau local ({SERVICE})"),
        Err(e) => log::warn!("annonce sur le réseau impossible : {e}"),
    }
}

/// Machine pilotée : recherche les machines principales.
pub fn browse(interface: Option<&str>) {
    let Some(d) = daemon(interface) else { return };
    let rx = match d.browse(SERVICE) {
        Ok(rx) => rx,
        Err(e) => {
            log::warn!("recherche sur le réseau impossible : {e}");
            return;
        }
    };
    let _ = std::thread::Builder::new().name("baskul-discovery".into()).spawn(move || {
        while let Ok(ev) = rx.recv() {
            match ev {
                ServiceEvent::ServiceResolved(s) => {
                    let mut addresses: Vec<SocketAddr> =
                        s.addresses.iter().map(|a| SocketAddr::new(a.to_ip_addr(), s.port)).collect();
                    addresses.sort_by_key(|a| (!a.is_ipv4(), a.ip().is_loopback()));
                    let prop = |k: &str| s.get_property_val_str(k).unwrap_or_default().to_string();
                    let found = Found {
                        name: Some(prop("name")).filter(|n| !n.is_empty()).unwrap_or_else(|| s.host.clone()),
                        addresses,
                        os: prop("os"),
                        protocol: prop("protocol").parse().unwrap_or(0),
                    };
                    let mut all = FOUND.lock().unwrap_or_else(|e| e.into_inner());
                    if all.get(&s.fullname) != Some(&found) {
                        log::info!("machine principale trouvée : « {} » {:?}", found.name, found.addresses);
                        all.insert(s.fullname.clone(), found);
                    }
                }
                ServiceEvent::ServiceRemoved(_, fullname) => {
                    if let Some(f) = FOUND.lock().unwrap_or_else(|e| e.into_inner()).remove(&fullname) {
                        log::info!("machine principale disparue du réseau : « {} »", f.name);
                    }
                }
                _ => {}
            }
        }
    });
}

/// Nom utilisable dans un nom d'hôte ou d'instance mDNS.
fn host_label(name: &str) -> String {
    let s: String =
        name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' }).take(63).collect();
    let s = s.trim_matches('-').to_string();
    if s.is_empty() { "baskul".into() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_dns_safe() {
        assert_eq!(host_label("Bureau d'été"), "Bureau-d--t");
        assert_eq!(host_label("PC-BUREAU"), "PC-BUREAU");
        assert_eq!(host_label("  "), "baskul");
    }
}
