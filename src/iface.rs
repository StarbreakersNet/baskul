//! Cartes réseau de la machine, pour choisir celle qu'utilise Baskul.

use serde::Serialize;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NetInterface {
    /// Nom donné par le système (`en0`, `Ethernet`, `Wi-Fi`…) : c'est lui qui
    /// est enregistré dans la configuration.
    pub name: String,
    /// Nom lisible (macOS : « Wi-Fi », « Ethernet »… au lieu de `en0`).
    pub label: String,
    /// Adresses IPv4.
    pub addresses: Vec<IpAddr>,
}

/// Cartes actives qui ont au moins une adresse IPv4 utilisable (sans la
/// boucle locale ni les adresses automatiques 169.254.x.x).
pub fn list() -> Vec<NetInterface> {
    let labels = hardware_labels();
    let mut out: Vec<NetInterface> = Vec::new();
    for i in if_addrs::get_if_addrs().unwrap_or_default() {
        let ip = i.ip();
        if !ip.is_ipv4() || i.is_loopback() || i.is_link_local() || i.oper_status == if_addrs::IfOperStatus::Down {
            continue;
        }
        match out.iter_mut().find(|n| n.name == i.name) {
            Some(n) => n.addresses.push(ip),
            None => {
                let label = labels.iter().find(|(dev, _)| *dev == i.name).map_or(i.name.clone(), |(_, l)| l.clone());
                out.push(NetInterface { name: i.name.clone(), label, addresses: vec![ip] });
            }
        }
    }
    out
}

/// Adresse IPv4 de la carte `name`.
pub fn address_of(name: &str) -> Option<IpAddr> {
    list().into_iter().find(|i| i.name == name).and_then(|i| i.addresses.first().copied())
}

/// macOS : noms lisibles des cartes (`networksetup -listallhardwareports`).
fn hardware_labels() -> Vec<(String, String)> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }
    let Ok(out) = std::process::Command::new("/usr/sbin/networksetup").arg("-listallhardwareports").output() else {
        return Vec::new();
    };
    parse_hardware_ports(&String::from_utf8_lossy(&out.stdout))
}

fn parse_hardware_ports(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut port: Option<&str> = None;
    for line in text.lines() {
        if let Some(p) = line.strip_prefix("Hardware Port: ") {
            port = Some(p.trim());
        } else if let (Some(dev), Some(p)) = (line.strip_prefix("Device: "), port.take()) {
            out.push((dev.trim().to_string(), p.to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_ports_give_readable_names() {
        let text =
            "\nHardware Port: Ethernet\nDevice: en0\nEthernet Address: aa\n\nHardware Port: Wi-Fi\nDevice: en1\n";
        assert_eq!(parse_hardware_ports(text), vec![("en0".into(), "Ethernet".into()), ("en1".into(), "Wi-Fi".into())]);
    }

    #[test]
    fn interfaces_have_ipv4_addresses() {
        for i in list() {
            assert!(!i.addresses.is_empty() && i.addresses.iter().all(|a| a.is_ipv4() && !a.is_loopback()));
        }
    }
}
