//! Ajout d'une machine par code court : la principale affiche six chiffres,
//! la pilotée les saisit et reçoit le secret partagé.
//!
//! Sécurité : SPAKE2 transforme le code en une clé forte sans le laisser
//! deviner hors ligne (un espion du réseau n'apprend rien d'utile) ; un
//! attaquant actif n'a droit qu'à [`MAX_FAILURES`] essais en ligne, pendant
//! [`WINDOW`]. La clé obtenue sert de PSK à un échange Noise, comme une
//! connexion ordinaire (`crypto.rs`) : un mauvais code fait échouer cet
//! échange, et le secret ne circule que chiffré.
//!
//! Échange, sur le port d'écoute de la principale + 1 (trames : longueur sur
//! 2 octets, big-endian) : [`MAGIC`], message SPAKE2 de la pilotée, message
//! SPAKE2 de la principale, poignée de main Noise, nom de la pilotée, puis
//! invitation (secret et nom de la principale).

use crate::crypto::{self, MAX_MSG};
use blake2::{Blake2s256, Digest};
use serde::{Deserialize, Serialize};
use spake2::{Ed25519Group, Identity, Password, Spake2};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const CODE_LEN: usize = 6;
/// Durée de validité d'un code.
pub const WINDOW: Duration = Duration::from_secs(120);
/// Mauvais codes acceptés avant de fermer l'appairage.
pub const MAX_FAILURES: u32 = 3;

const MAGIC: &[u8; 8] = b"BSKPAIR1";
const ID_SECONDARY: &[u8] = b"baskul-pilotee";
const ID_PRIMARY: &[u8] = b"baskul-principale";
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(100);

/// Ce que la principale transmet à la pilotée.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invitation {
    pub secret: String,
    /// Nom de la machine principale.
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct Hello {
    name: String,
}

/// Port de l'appairage, à côté du port d'écoute de la principale.
pub fn port(listen_port: u16) -> u16 {
    listen_port.wrapping_add(1)
}

/// Six chiffres tirés uniformément (sans biais de modulo).
pub fn new_code() -> Result<String, String> {
    loop {
        let mut bytes = [0u8; 4];
        getrandom::fill(&mut bytes).map_err(|e| format!("générateur aléatoire : {e}"))?;
        let n = u32::from_le_bytes(bytes);
        // Plus grand multiple de 10^6 sous 2^32 : au-delà, on retire
        if n < 4_294_000_000 {
            return Ok(format!("{:0width$}", n % 1_000_000, width = CODE_LEN));
        }
    }
}

/// Garde les chiffres d'un code saisi ("123 456", "123-456").
pub fn normalize_code(code: &str) -> Result<String, String> {
    let digits: String = code.chars().filter(char::is_ascii_digit).collect();
    if digits.len() == CODE_LEN { Ok(digits) } else { Err(format!("le code a {CODE_LEN} chiffres")) }
}

fn psk_from(key: &[u8]) -> [u8; 32] {
    let mut h = Blake2s256::new();
    h.update(b"baskul-pairing-v1\0");
    h.update(key);
    let mut psk = [0u8; 32];
    psk.copy_from_slice(&h.finalize());
    psk
}

fn write_frame(stream: &mut TcpStream, data: &[u8]) -> Result<(), String> {
    let len = u16::try_from(data.len()).map_err(|_| "message trop long".to_string())?;
    stream.write_all(&len.to_be_bytes()).and_then(|()| stream.write_all(data)).map_err(|e| e.to_string())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut len = [0u8; 2];
    stream.read_exact(&mut len).map_err(|e| e.to_string())?;
    let mut data = vec![0u8; u16::from_be_bytes(len) as usize];
    stream.read_exact(&mut data).map_err(|e| e.to_string())?;
    Ok(data)
}

/// Poignée de main Noise avec la clé SPAKE2, puis envoi et réception d'un message chiffré.
fn exchange(stream: &mut TcpStream, key: &[u8], initiator: bool, outgoing: &[u8]) -> Result<Vec<u8>, String> {
    let bad_code = |_| "code incorrect".to_string();
    let mut hs = crypto::new_handshake(&psk_from(key), initiator).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; MAX_MSG];
    let mut plain = vec![0u8; MAX_MSG];
    if initiator {
        let n = hs.write_message(&[], &mut buf).map_err(|e| e.to_string())?;
        write_frame(stream, &buf[..n])?;
        hs.read_message(&read_frame(stream)?, &mut plain).map_err(bad_code)?;
    } else {
        hs.read_message(&read_frame(stream)?, &mut plain).map_err(bad_code)?;
        let n = hs.write_message(&[], &mut buf).map_err(|e| e.to_string())?;
        write_frame(stream, &buf[..n])?;
    }
    let (mut sealer, mut opener) = crypto::split(hs.into_stateless_transport_mode().map_err(|e| e.to_string())?);
    if initiator {
        write_frame(stream, sealer.seal(outgoing).map_err(|e| e.to_string())?)?;
        opener.open(&read_frame(stream)?).map(<[u8]>::to_vec).map_err(bad_code)
    } else {
        let incoming = opener.open(&read_frame(stream)?).map(<[u8]>::to_vec).map_err(bad_code)?;
        write_frame(stream, sealer.seal(outgoing).map_err(|e| e.to_string())?)?;
        Ok(incoming)
    }
}

/// Pilotée : rejoint la principale à `addr` (son port d'appairage) avec le code.
pub fn join(addr: SocketAddr, code: &str, name: &str) -> Result<Invitation, String> {
    let code = normalize_code(code)?;
    let mut stream = TcpStream::connect_timeout(&addr, IO_TIMEOUT)
        .map_err(|e| format!("machine principale injoignable ({addr}) : {e}"))?;
    stream.set_read_timeout(Some(IO_TIMEOUT)).and_then(|()| stream.set_write_timeout(Some(IO_TIMEOUT))).ok();
    stream.write_all(MAGIC).map_err(|e| e.to_string())?;
    let (spake, outbound) = Spake2::<Ed25519Group>::start_a(
        &Password::new(code.as_bytes()),
        &Identity::new(ID_SECONDARY),
        &Identity::new(ID_PRIMARY),
    );
    write_frame(&mut stream, &outbound)?;
    let key = spake.finish(&read_frame(&mut stream)?).map_err(|e| format!("appairage : {e:?}"))?;
    let hello = serde_json::to_vec(&Hello { name: name.to_string() }).map_err(|e| e.to_string())?;
    let reply = exchange(&mut stream, &key, true, &hello)?;
    serde_json::from_slice(&reply).map_err(|e| format!("réponse de la principale illisible : {e}"))
}

/// Une tentative de la pilotée connectée sur `stream` ; renvoie son nom si le code est bon.
fn answer(stream: &mut TcpStream, code: &str, invitation: &Invitation) -> Result<String, String> {
    stream.set_read_timeout(Some(IO_TIMEOUT)).and_then(|()| stream.set_write_timeout(Some(IO_TIMEOUT))).ok();
    let mut magic = [0u8; 8];
    stream.read_exact(&mut magic).map_err(|e| e.to_string())?;
    if &magic != MAGIC {
        return Err("connexion qui n'est pas un appairage de Baskul".into());
    }
    let (spake, outbound) = Spake2::<Ed25519Group>::start_b(
        &Password::new(code.as_bytes()),
        &Identity::new(ID_SECONDARY),
        &Identity::new(ID_PRIMARY),
    );
    let inbound = read_frame(stream)?;
    write_frame(stream, &outbound)?;
    let key = spake.finish(&inbound).map_err(|e| format!("appairage : {e:?}"))?;
    let invitation = serde_json::to_vec(invitation).map_err(|e| e.to_string())?;
    let hello: Hello = serde_json::from_slice(&exchange(stream, &key, false, &invitation)?)
        .map_err(|e| format!("présentation de la pilotée illisible : {e}"))?;
    Ok(hello.name)
}

/// Principale : attend sur `listener` une pilotée qui présente `code`, jusqu'à
/// l'expiration, l'annulation (`cancel`) ou [`MAX_FAILURES`] mauvais codes.
/// Renvoie le nom de la pilotée.
pub fn serve(
    listener: &TcpListener,
    code: &str,
    invitation: &Invitation,
    cancel: &AtomicBool,
) -> Result<String, String> {
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + WINDOW;
    let mut failures = 0;
    loop {
        if cancel.load(Ordering::Acquire) {
            return Err("appairage annulé".into());
        }
        if Instant::now() >= deadline {
            return Err("code expiré".into());
        }
        match listener.accept() {
            Ok((mut stream, from)) => {
                let _ = stream.set_nonblocking(false);
                match answer(&mut stream, code, invitation) {
                    Ok(name) => {
                        log::info!("appairage réussi avec {name:?} ({from})");
                        return Ok(name);
                    }
                    Err(e) => {
                        failures += 1;
                        log::warn!("appairage refusé ({from}) : {e}");
                        if failures >= MAX_FAILURES {
                            return Err("trop de codes incorrects : appairage fermé".into());
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(POLL),
            Err(e) => return Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn invitation() -> Invitation {
        Invitation { secret: "le-secret-partage".into(), name: "Principale".into() }
    }

    fn server(code: &str) -> (SocketAddr, Arc<AtomicBool>, std::thread::JoinHandle<Result<String, String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let (code, stop) = (code.to_string(), cancel.clone());
        let handle = std::thread::spawn(move || serve(&listener, &code, &invitation(), &stop));
        (addr, cancel, handle)
    }

    #[test]
    fn the_right_code_gives_the_secret() {
        let (addr, _, handle) = server("123456");
        assert_eq!(join(addr, "123 456", "Pilotée").unwrap(), invitation());
        assert_eq!(handle.join().unwrap(), Ok("Pilotée".into()));
    }

    #[test]
    fn wrong_codes_close_the_pairing() {
        let (addr, _, handle) = server("123456");
        for _ in 0..MAX_FAILURES {
            assert!(join(addr, "654321", "Intrus").is_err());
        }
        assert!(handle.join().unwrap().is_err());
    }

    #[test]
    fn cancel_stops_waiting() {
        let (_, cancel, handle) = server("123456");
        cancel.store(true, Ordering::Release);
        assert_eq!(handle.join().unwrap(), Err("appairage annulé".into()));
    }

    #[test]
    fn codes_have_six_digits() {
        for _ in 0..50 {
            let code = new_code().unwrap();
            assert_eq!(code.len(), CODE_LEN);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
        assert!(normalize_code("12-34").is_err());
        assert_eq!(normalize_code(" 987 654 ").unwrap(), "987654");
    }
}
