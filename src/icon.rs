//! Dessin de l'icône de la zone de notification (Windows) et de la barre des
//! menus (macOS), en pixels RGBA, sans fichier image.
//!
//! Deux petits écrans côte à côte : celui de gauche représente la machine
//! principale, celui de droite la machine secondaire. L'écran où se trouve le
//! curseur est plein. La couleur indique l'état de la connexion.
//!
//! En monochrome (une seule couleur, assortie aux icônes du système), l'état
//! se lit à la forme : en attente, l'écran de l'autre machine est estompé ;
//! en erreur, tout est estompé et barré.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Waiting,
    Connected,
    Error,
}

impl Tone {
    /// Couleur correspondant à l'état de la connexion.
    pub fn of(conn: &crate::status::Conn) -> Tone {
        use crate::status::Conn;
        match conn {
            Conn::Connected(_) => Tone::Connected,
            Conn::Error(_) => Tone::Error,
            Conn::Waiting(_) => Tone::Waiting,
        }
    }

    fn rgb(self) -> [u8; 3] {
        match self {
            Tone::Waiting => [150, 150, 150],
            Tone::Connected => [46, 160, 67],
            Tone::Error => [215, 58, 73],
        }
    }
}

/// Couleurs de l'icône.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// La couleur indique l'état de la connexion.
    Color,
    /// Une seule couleur (macOS : noir, l'image servant de modèle).
    Mono([u8; 3]),
}

/// Opacité des parties estompées (monochrome).
const DIM: u8 = 100;

/// Rend une icône carrée `size`×`size` en RGBA (non prémultiplié).
pub fn render(size: u32, tone: Tone, remote_active: bool, style: Style) -> Vec<u8> {
    let s = size.max(8) as f32;
    let mut px = vec![0u8; (size * size * 4) as usize];
    let [r, g, b] = match style {
        Style::Color => tone.rgb(),
        Style::Mono(rgb) => rgb,
    };
    // Opacité de chaque écran (monochrome : l'état se lit à la forme).
    let alpha = |screen: usize| match (style, tone) {
        (Style::Color, _) | (Style::Mono(_), Tone::Connected) => 255,
        (Style::Mono(_), Tone::Waiting) => {
            if screen == 0 {
                255
            } else {
                DIM
            }
        }
        (Style::Mono(_), Tone::Error) => DIM,
    };
    // Coordonnées sur une grille de 16 unités.
    let u = s / 16.0;
    let screens = [
        // (x0, y0, x1, y1) en unités
        (0.5f32, 3.0f32, 7.5f32, 10.0f32),
        (8.5, 3.0, 15.5, 10.0),
    ];
    let filled = if remote_active { 1 } else { 0 };
    let stroke = (u * 1.2).max(1.0);
    let mut put = |x: u32, y: u32, a: u8| {
        if x < size && y < size {
            let i = ((y * size + x) * 4) as usize;
            if a >= px[i + 3] {
                px[i] = r;
                px[i + 1] = g;
                px[i + 2] = b;
                px[i + 3] = a;
            }
        }
    };
    for (idx, (x0, y0, x1, y1)) in screens.iter().enumerate() {
        let (ax0, ay0, ax1, ay1) = (x0 * u, y0 * u, x1 * u, y1 * u);
        for y in 0..size {
            for x in 0..size {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let inside = fx >= ax0 && fx < ax1 && fy >= ay0 && fy < ay1;
                if !inside {
                    continue;
                }
                let border = fx < ax0 + stroke || fx >= ax1 - stroke || fy < ay0 + stroke || fy >= ay1 - stroke;
                if idx == filled || border {
                    put(x, y, alpha(idx));
                }
            }
        }
        // Pied de l'écran.
        let cx = (x0 + x1) / 2.0 * u;
        for y in 0..size {
            for x in 0..size {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let stem = (fx - cx).abs() < stroke * 0.6 && fy >= ay1 && fy < ay1 + 2.0 * u;
                let base = (fx - cx).abs() < 2.0 * u && fy >= ay1 + 2.0 * u && fy < ay1 + 2.0 * u + stroke;
                if stem || base {
                    put(x, y, alpha(idx));
                }
            }
        }
    }
    // Erreur (monochrome) : une barre en travers.
    if matches!(style, Style::Mono(_)) && tone == Tone::Error {
        for y in 0..size {
            for x in 0..size {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                // Droite de (2, 1) à (14, 15) en unités.
                let d = ((fy - u) * 12.0 - (fx - 2.0 * u) * 14.0).abs() / (12.0f32.hypot(14.0));
                if d < stroke * 0.75 && fx > 1.5 * u && fx < 14.5 * u {
                    put(x, y, 255);
                }
            }
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_expected_size_and_content() {
        for size in [16, 20, 24, 32, 48] {
            let a = render(size, Tone::Connected, false, Style::Color);
            let b = render(size, Tone::Connected, true, Style::Color);
            assert_eq!(a.len(), (size * size * 4) as usize);
            assert_ne!(a, b);
            assert!(a.chunks(4).any(|p| p[3] == 255));
        }
    }

    #[test]
    fn mono_uses_one_color_and_shows_state_by_shape() {
        let black = Style::Mono([0, 0, 0]);
        let icons: Vec<Vec<u8>> =
            [Tone::Connected, Tone::Waiting, Tone::Error].iter().map(|t| render(32, *t, false, black)).collect();
        for icon in &icons {
            assert!(icon.chunks(4).all(|p| p[3] == 0 || p[..3] == [0, 0, 0]));
        }
        // Les trois états se distinguent.
        assert_ne!(icons[0], icons[1]);
        assert_ne!(icons[1], icons[2]);
        assert_ne!(icons[0], icons[2]);
    }
}
