//! Messages échangés entre la machine principale et les pilotées.
//!
//! Sérialisation : postcard (binaire compact), puis chiffrement Noise (voir
//! `crypto.rs`) et encadrement par une longueur sur 2 octets (voir `net.rs`).

use serde::{Deserialize, Serialize};

/// Incrémenté à chaque changement incompatible du protocole.
pub const PROTOCOL_VERSION: u32 = 4;

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

/// Machine où le clavier et la souris doivent agir (commande `baskul switch`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SwitchTo {
    Primary,
    /// Pilotée, par son nom.
    Client(String),
    /// Dernière pilotée utilisée (à défaut, la première connectée).
    LastClient,
    /// Machine suivante : la principale, puis les pilotées dans l'ordre.
    Next,
    /// Pilotée d'où vient la demande.
    Sender,
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
    /// Présentation : la pilotée se présente, la principale répond par sa
    /// propre présentation (ou par `Refused`).
    Hello {
        version: u32,
        role: Role,
        name: String,
        os: String,
        /// Identifiant propre à la machine (deux pilotées de même nom sont
        /// refusées).
        id: String,
    },
    /// Connexion refusée, avec la raison à afficher.
    Refused {
        reason: String,
    },
    /// Maintien de connexion (envoyé quand rien d'autre ne passe).
    Ping,

    // --- Principale → pilotée ---
    /// Le curseur entre sur la pilotée par le bord `edge`, à la position
    /// relative `ratio` (0 = début du bord, 1 = fin). `exits` : bords par
    /// lesquels il peut ressortir vers une machine voisine.
    Enter {
        edge: Edge,
        ratio: f32,
        exits: Vec<Edge>,
    },
    /// Bascule par raccourci ou commande : le curseur de la pilotée reprend là
    /// où il était.
    Jump {
        exits: Vec<Edge>,
    },
    /// Les bords de sortie ont changé (une machine voisine s'est connectée ou
    /// déconnectée, la grille a changé).
    Exits {
        exits: Vec<Edge>,
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

    // --- Pilotée → principale ---
    /// Le curseur ressort par le bord `edge`, à la position relative `ratio`.
    Leave {
        edge: Edge,
        ratio: f32,
    },
    /// Commande `baskul switch` lancée sur la pilotée : c'est la principale
    /// qui décide où agissent le clavier et la souris.
    Switch {
        to: SwitchTo,
    },

    // --- Dans les deux sens ---
    /// Début d'un contenu de presse-papiers : en-tête (`ClipHeader`) de
    /// `header` octets, suivi du contenu des fichiers, `total` octets en tout.
    ClipStart {
        id: u32,
        header: u32,
        total: u64,
    },
    ClipChunk {
        id: u32,
        data: Vec<u8>,
    },
    /// Envoi abandonné (fichier devenu illisible…).
    ClipCancel {
        id: u32,
    },
    /// L'utilisateur est actif sur la machine qui envoie ce message (au plus
    /// un message toutes les quelques secondes) : celle qui le reçoit reste
    /// active elle aussi (veille de l'écran, économiseur, statut « absent »).
    UserActive,
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        // postcard ne peut échouer que sur des types non sérialisables.
        postcard::to_stdvec(self).expect("sérialisation du message")
    }

    pub fn decode(bytes: &[u8]) -> Result<Message, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

/// Format d'une image du presse-papiers, transmise telle quelle.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Tiff,
    Webp,
    Heic,
    Bmp,
}

impl ImageFormat {
    pub const ALL: [ImageFormat; 7] = [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Gif,
        ImageFormat::Tiff,
        ImageFormat::Webp,
        ImageFormat::Heic,
        ImageFormat::Bmp,
    ];

    /// Format d'après les premiers octets du fichier.
    pub fn sniff(data: &[u8]) -> Option<ImageFormat> {
        Some(match data {
            [0x89, b'P', b'N', b'G', ..] => ImageFormat::Png,
            [0xff, 0xd8, 0xff, ..] => ImageFormat::Jpeg,
            [b'G', b'I', b'F', b'8', ..] => ImageFormat::Gif,
            [b'I', b'I', 42, 0, ..] | [b'M', b'M', 0, 42, ..] => ImageFormat::Tiff,
            [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => ImageFormat::Webp,
            [_, _, _, _, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c' | b'x', ..]
            | [_, _, _, _, b'f', b't', b'y', b'p', b'm', b'i', b'f', b'1', ..] => ImageFormat::Heic,
            [b'B', b'M', ..] => ImageFormat::Bmp,
            _ => return None,
        })
    }

    /// Format d'après l'extension d'un nom de fichier.
    pub fn from_extension(name: &str) -> Option<ImageFormat> {
        let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
        Some(match ext.as_str() {
            "png" => ImageFormat::Png,
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "gif" => ImageFormat::Gif,
            "tif" | "tiff" => ImageFormat::Tiff,
            "webp" => ImageFormat::Webp,
            "heic" | "heif" => ImageFormat::Heic,
            "bmp" => ImageFormat::Bmp,
            _ => return None,
        })
    }

    /// Type uniforme macOS.
    pub fn uti(self) -> &'static str {
        match self {
            ImageFormat::Png => "public.png",
            ImageFormat::Jpeg => "public.jpeg",
            ImageFormat::Gif => "com.compuserve.gif",
            ImageFormat::Tiff => "public.tiff",
            ImageFormat::Webp => "org.webmproject.webp",
            ImageFormat::Heic => "public.heic",
            ImageFormat::Bmp => "com.microsoft.bmp",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Webp => "webp",
            ImageFormat::Heic => "heic",
            ImageFormat::Bmp => "bmp",
        }
    }
}

/// Une représentation d'image : les octets du fichier, sans conversion.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClipImage {
    pub format: ImageFormat,
    pub data: Vec<u8>,
}

/// Fichier ou dossier copié. Le contenu des fichiers suit l'en-tête, dans
/// l'ordre de la liste.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClipFile {
    /// Chemin relatif, séparé par « / » (le premier élément est le nom de
    /// l'élément copié).
    pub path: String,
    /// Taille (octets) ; `None` pour un dossier.
    pub size: Option<u64>,
}

/// En-tête d'un contenu de presse-papiers.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ClipHeader {
    pub text: Option<String>,
    pub html: Option<String>,
    /// Représentations d'une image, la première étant l'originale ; les
    /// suivantes servent de secours aux apps qui ne lisent pas son format.
    pub images: Vec<ClipImage>,
    pub files: Vec<ClipFile>,
}

impl ClipHeader {
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.html.is_none() && self.images.is_empty() && self.files.is_empty()
    }

    /// Taille totale du contenu des fichiers.
    pub fn files_size(&self) -> u64 {
        self.files.iter().filter_map(|f| f.size).sum()
    }

    pub fn encode(&self) -> Vec<u8> {
        postcard::to_stdvec(self).expect("sérialisation du presse-papiers")
    }

    pub fn decode(bytes: &[u8]) -> Result<ClipHeader, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let msgs = vec![
            Message::Hello {
                version: PROTOCOL_VERSION,
                role: Role::Primary,
                name: "Bureau".into(),
                os: "windows".into(),
                id: "abcd".into(),
            },
            Message::Refused { reason: "nom déjà pris".into() },
            Message::Ping,
            Message::Enter { edge: Edge::Left, ratio: 0.25, exits: vec![Edge::Left, Edge::Bottom] },
            Message::Jump { exits: vec![Edge::Right] },
            Message::Exits { exits: vec![] },
            Message::Motion { dx: -3, dy: 12 },
            Message::Button { button: MouseButton::Back, down: true },
            Message::Wheel { dx: 0, dy: -120 },
            Message::Key { code: 0x070004, down: true, repeat: false },
            Message::ReleaseAll,
            Message::Leave { edge: Edge::Top, ratio: 0.5 },
            Message::Switch { to: SwitchTo::Client("Atelier".into()) },
            Message::ClipStart { id: 7, header: 20, total: 100 },
            Message::ClipChunk { id: 7, data: vec![1, 2, 3] },
            Message::ClipCancel { id: 7 },
            Message::UserActive,
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

    #[test]
    fn image_formats_are_recognized() {
        let png = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        assert_eq!(ImageFormat::sniff(&png), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::sniff(&[0xff, 0xd8, 0xff, 0xe0]), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some(ImageFormat::Webp));
        assert_eq!(ImageFormat::sniff(b"\0\0\0\x18ftypheic"), Some(ImageFormat::Heic));
        assert_eq!(ImageFormat::sniff(b"texte"), None);
        assert_eq!(ImageFormat::from_extension("Photo.JPEG"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("notes.txt"), None);
    }
}
