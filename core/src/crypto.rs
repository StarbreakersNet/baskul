//! Chiffrement et authentification : protocole Noise `NNpsk0` avec une clé
//! dérivée du secret partagé. Une machine qui ne connaît pas le secret ne
//! peut ni se connecter, ni lire, ni injecter de frappes.

use blake2::{Blake2s256, Digest};
use snow::{Builder, HandshakeState, StatelessTransportState};

pub const NOISE_PARAMS: &str = "Noise_NNpsk0_25519_ChaChaPoly_BLAKE2s";
/// Taille maximale d'un message Noise.
pub const MAX_MSG: usize = 65535;
pub const TAG_LEN: usize = 16;

/// Dérive la clé pré-partagée (32 octets) du secret de la configuration.
pub fn derive_psk(secret: &str) -> [u8; 32] {
    let mut h = Blake2s256::new();
    h.update(b"bascule-psk-v1\0");
    h.update(secret.trim().as_bytes());
    let out = h.finalize();
    let mut psk = [0u8; 32];
    psk.copy_from_slice(&out);
    psk
}

pub fn new_handshake(psk: &[u8; 32], initiator: bool) -> Result<HandshakeState, snow::Error> {
    let params = NOISE_PARAMS.parse().expect("paramètres Noise valides");
    let b = Builder::new(params).psk(0, psk)?;
    if initiator { b.build_initiator() } else { b.build_responder() }
}

/// Moitié « émission » d'un canal chiffré (un seul thread l'utilise).
pub struct Sealer {
    state: std::sync::Arc<StatelessTransportState>,
    nonce: u64,
    buf: Vec<u8>,
}

/// Moitié « réception » d'un canal chiffré (un seul thread l'utilise).
pub struct Opener {
    state: std::sync::Arc<StatelessTransportState>,
    nonce: u64,
    buf: Vec<u8>,
}

pub fn split(state: StatelessTransportState) -> (Sealer, Opener) {
    let state = std::sync::Arc::new(state);
    (
        Sealer { state: state.clone(), nonce: 0, buf: vec![0u8; MAX_MSG] },
        Opener { state, nonce: 0, buf: vec![0u8; MAX_MSG] },
    )
}

impl Sealer {
    /// Chiffre `plain` ; renvoie le message chiffré.
    pub fn seal(&mut self, plain: &[u8]) -> Result<&[u8], snow::Error> {
        let n = self.state.write_message(self.nonce, plain, &mut self.buf)?;
        self.nonce += 1;
        Ok(&self.buf[..n])
    }
}

impl Opener {
    /// Déchiffre `cipher` ; renvoie le texte clair.
    pub fn open(&mut self, cipher: &[u8]) -> Result<&[u8], snow::Error> {
        let n = self.state.read_message(self.nonce, cipher, &mut self.buf)?;
        self.nonce += 1;
        Ok(&self.buf[..n])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Pair = ((Sealer, Opener), (Sealer, Opener));

    fn pair(a: &str, b: &str) -> Result<Pair, snow::Error> {
        let mut i = new_handshake(&derive_psk(a), true)?;
        let mut r = new_handshake(&derive_psk(b), false)?;
        let mut m1 = vec![0u8; MAX_MSG];
        let mut p = vec![0u8; MAX_MSG];
        let n = i.write_message(&[], &mut m1)?;
        r.read_message(&m1[..n], &mut p)?;
        let n = r.write_message(&[], &mut m1)?;
        i.read_message(&m1[..n], &mut p)?;
        Ok((split(i.into_stateless_transport_mode()?), split(r.into_stateless_transport_mode()?)))
    }

    #[test]
    fn same_secret_works() {
        let ((mut is, mut io), (mut rs, mut ro)) = pair("abc-def-ghi-jkl", " abc-def-ghi-jkl ").unwrap();
        for i in 0..5u8 {
            let c = is.seal(&[i; 10]).unwrap().to_vec();
            assert_eq!(ro.open(&c).unwrap(), &[i; 10]);
            let c = rs.seal(&[i; 3]).unwrap().to_vec();
            assert_eq!(io.open(&c).unwrap(), &[i; 3]);
        }
    }

    #[test]
    fn different_secret_fails() {
        assert!(pair("abc-def-ghi-jkl", "abc-def-ghi-jkm").is_err());
    }

    #[test]
    fn tampering_is_detected() {
        let ((mut is, _), (_, mut ro)) = pair("secret-secret", "secret-secret").unwrap();
        let mut c = is.seal(b"hello").unwrap().to_vec();
        c[0] ^= 1;
        assert!(ro.open(&c).is_err());
    }
}
