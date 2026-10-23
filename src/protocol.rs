//! Messages échangés entre les deux machines.
//!
//! Sérialisation : postcard (binaire compact), puis chiffrement Noise (voir
//! `crypto.rs`) et encadrement par une longueur sur 2 octets (voir `net.rs`).

use serde::{Deserialize, Serialize};

/// Incrémenté à chaque changement incompatible du protocole.
pub const PROTOCOL_VERSION: u32 = 3;

/// Port TCP par défaut.
pub const DEFAULT_PORT: u16 = 24870;

/// Taille maximale d'un morceau de presse-papiers dans un message.
/// (Un message Noise fait au plus 65535 octets, tag compris.)
pub const CLIP_CHUNK: usize = 48 * 1024;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Machine qui possède le clavier et la souris physiques.
    Primary,
    /// Machine pilotée à distance.
    Secondary,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Role::Primary => "principale",
            Role::Secondary => "secondaire",
        }
    }
}

/// Bord d'un écran.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    pub fn opposite(self) -> Edge {
        match self {
            Edge::Left => Edge::Right,
            Edge::Right => Edge::Left,
            Edge::Top => Edge::Bottom,
            Edge::Bottom => Edge::Top,
        }
    }

    pub fn is_horizontal_axis(self) -> bool {
        matches!(self, Edge::Left | Edge::Right)
    }
}

/// Machine où le clavier et la souris doivent agir (commande `bascule switch`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchTo {
    Primary,
    Secondary,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    /// Bouton 4 (« précédent »).
    Back,
    /// Bouton 5 (« suivant »).
    Forward,
}

impl MouseButton {
    pub const ALL: [MouseButton; 5] =
        [MouseButton::Left, MouseButton::Right, MouseButton::Middle, MouseButton::Back, MouseButton::Forward];

    pub fn index(self) -> usize {
        match self {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Message {
    /// Premier message envoyé par chaque côté après la poignée de main.
    Hello {
        version: u32,
        role: Role,
        name: String,
        os: String,
    },
    /// Maintien de connexion (envoyé quand rien d'autre ne passe).
    Ping,

    // --- Principale → secondaire ---
    /// Le curseur entre sur l'écran secondaire par le bord `edge`, à la
    /// position relative `ratio` (0 = début du bord, 1 = fin).
    Enter {
        edge: Edge,
        ratio: f32,
    },
    /// Bascule par raccourci clavier : le curseur de la machine secondaire
    /// reprend là où il était. `exit` : bord par lequel il pourra ressortir
    /// (`None` : bascule par les bords désactivée).
    Jump {
        exit: Option<Edge>,
    },
    /// Déplacement relatif, en pixels de la machine principale (accélération
    /// du système principal déjà appliquée).
    Motion {
        dx: i32,
        dy: i32,
    },
    Button {
        button: MouseButton,
        down: bool,
    },
    /// Molette, en unités Windows (120 = un cran). `dy > 0` : vers le haut
    /// (molette poussée vers l'avant). `dx > 0` : vers la droite.
    Wheel {
        dx: i32,
        dy: i32,
    },
    /// Touche physique, code USB HID.
    Key {
        code: u32,
        down: bool,
        repeat: bool,
    },
    /// Relâche toutes les touches et tous les boutons enfoncés.
    ReleaseAll,

    // --- Secondaire → principale ---
    /// Le curseur ressort par le bord d'entrée, à la position relative `ratio`.
    Leave {
        ratio: f32,
    },

    // --- Dans les deux sens ---
    /// Début d'un contenu de presse-papiers de `total` octets.
    ClipStart {
        id: u32,
        total: u32,
    },
    ClipChunk {
        id: u32,
        data: Vec<u8>,
    },

    // --- Secondaire → principale ---
    /// Commande `bascule switch` lancée sur la machine pilotée : c'est la
    /// machine principale qui décide où agissent le clavier et la souris.
    Switch {
        to: SwitchTo,
    },

    // --- Dans les deux sens ---
    /// L'utilisateur est actif sur la machine qui envoie ce message (au plus
    /// un message toutes les quelques secondes) : celle qui le reçoit reste
    /// active elle aussi (veille de l'écran, économiseur, statut « absent »).
    UserActive,
}

impl Message {
    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Message::Enter { .. }
                | Message::Jump { .. }
                | Message::Motion { .. }
                | Message::Button { .. }
                | Message::Wheel { .. }
                | Message::Key { .. }
                | Message::ReleaseAll
                | Message::Leave { .. }
        )
    }

    pub fn encode(&self) -> Vec<u8> {
        // postcard ne peut échouer que sur des types non sérialisables.
        postcard::to_stdvec(self).expect("sérialisation du message")
    }

    pub fn decode(bytes: &[u8]) -> Result<Message, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

/// Contenu de presse-papiers transmis (sérialisé puis découpé en morceaux).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ClipPayload {
    pub text: Option<String>,
    pub html: Option<String>,
    /// Image encodée en PNG.
    pub png: Option<Vec<u8>>,
}

impl ClipPayload {
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.html.is_none() && self.png.is_none()
    }

    pub fn encode(&self) -> Vec<u8> {
        postcard::to_stdvec(self).expect("sérialisation du presse-papiers")
    }

    pub fn decode(bytes: &[u8]) -> Result<ClipPayload, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let msgs = vec![
            Message::Hello { version: PROTOCOL_VERSION, role: Role::Primary, name: "PC".into(), os: "windows".into() },
            Message::Ping,
            Message::Enter { edge: Edge::Left, ratio: 0.25 },
            Message::Jump { exit: Some(Edge::Right) },
            Message::Jump { exit: None },
            Message::Motion { dx: -3, dy: 12 },
            Message::Button { button: MouseButton::Back, down: true },
            Message::Wheel { dx: 0, dy: -120 },
            Message::Key { code: 0x070004, down: true, repeat: false },
            Message::ReleaseAll,
            Message::Leave { ratio: 0.5 },
            Message::ClipStart { id: 7, total: 100 },
            Message::ClipChunk { id: 7, data: vec![1, 2, 3] },
        ];
        for m in msgs {
            let b = m.encode();
            assert_eq!(Message::decode(&b).unwrap(), m);
        }
    }

    #[test]
    fn motion_is_small() {
        // Les déplacements partent à chaque événement souris : ils doivent rester compacts.
        assert!(Message::Motion { dx: 5, dy: -5 }.encode().len() <= 4);
    }

    #[test]
    fn clip_chunk_fits_in_noise_message() {
        let m = Message::ClipChunk { id: u32::MAX, data: vec![0xff; CLIP_CHUNK] };
        assert!(m.encode().len() + 16 <= 65535);
    }
}
