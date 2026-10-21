//! Configuration (fichier TOML commenté).
//!
//! Le même format sert sur les deux machines ; chaque machine ignore les
//! sections qui ne la concernent pas. Tous les réglages se changent sans
//! recompiler : il suffit de redémarrer Bascule (menu « Redémarrer »).

use crate::hotkey::Hotkey;
use crate::protocol::{DEFAULT_PORT, Edge, Role};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleCfg {
    Primary,
    Secondary,
}

impl From<RoleCfg> for Role {
    fn from(r: RoleCfg) -> Role {
        match r {
            RoleCfg::Primary => Role::Primary,
            RoleCfg::Secondary => Role::Secondary,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeCfg {
    Left,
    Right,
    Top,
    Bottom,
}

impl From<EdgeCfg> for Edge {
    fn from(e: EdgeCfg) -> Edge {
        match e {
            EdgeCfg::Left => Edge::Left,
            EdgeCfg::Right => Edge::Right,
            EdgeCfg::Top => Edge::Top,
            EdgeCfg::Bottom => Edge::Bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollMode {
    /// Défilement par lignes (comme une molette classique sous Windows).
    Lines,
    /// Défilement au pixel (précis, comme un trackpad).
    Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SideButtonsMode {
    /// Boutons 4/5 transmis tels quels (Chrome, Firefox, VS Code…).
    Buttons,
    /// Geste « balayage » précédent/suivant (Safari, Finder, Xcode…).
    Swipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsoSwap {
    Auto,
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKeys {
    /// Les touches multimédia agissent sur la machine où se trouve le curseur.
    Remote,
    /// Les touches multimédia agissent toujours sur la machine principale.
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkCfg {
    pub listen: String,
    pub connect: String,
    pub secret: String,
}

impl Default for NetworkCfg {
    fn default() -> Self {
        NetworkCfg { listen: format!("0.0.0.0:{DEFAULT_PORT}"), connect: String::new(), secret: String::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScreenCfg {
    /// Bascule en poussant la souris contre le bord de l'écran.
    pub edge_switching: bool,
    pub secondary_position: EdgeCfg,
    pub corner_size: i32,
}

impl Default for ScreenCfg {
    fn default() -> Self {
        ScreenCfg { edge_switching: true, secondary_position: EdgeCfg::Right, corner_size: 10 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MouseCfg {
    pub speed: f64,
}

impl Default for MouseCfg {
    fn default() -> Self {
        MouseCfg { speed: 1.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScrollCfg {
    pub mode: ScrollMode,
    pub lines_per_notch: f64,
    pub pixels_per_notch: f64,
    pub invert_vertical: bool,
    pub invert_horizontal: bool,
}

impl Default for ScrollCfg {
    fn default() -> Self {
        ScrollCfg {
            mode: ScrollMode::Lines,
            lines_per_notch: 3.0,
            pixels_per_notch: 60.0,
            invert_vertical: false,
            invert_horizontal: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ButtonsCfg {
    pub side_buttons: SideButtonsMode,
    pub swap_side_buttons: bool,
}

impl Default for ButtonsCfg {
    fn default() -> Self {
        ButtonsCfg { side_buttons: SideButtonsMode::Buttons, swap_side_buttons: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KeyboardCfg {
    pub iso_swap: IsoSwap,
    pub media_keys: MediaKeys,
    pub remap: BTreeMap<String, String>,
}

impl Default for KeyboardCfg {
    fn default() -> Self {
        KeyboardCfg { iso_swap: IsoSwap::Auto, media_keys: MediaKeys::Remote, remap: BTreeMap::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClipboardCfg {
    pub enabled: bool,
    pub images: bool,
    pub html: bool,
    pub max_size_mb: u32,
    pub skip_sensitive: bool,
}

impl Default for ClipboardCfg {
    fn default() -> Self {
        ClipboardCfg { enabled: true, images: true, html: true, max_size_mb: 32, skip_sensitive: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HotkeysCfg {
    pub switch_to_secondary: String,
    pub return_to_primary: String,
    pub toggle_lock: String,
}

impl Default for HotkeysCfg {
    fn default() -> Self {
        HotkeysCfg {
            switch_to_secondary: "Shift+F12".into(),
            return_to_primary: "Shift+F11".into(),
            toggle_lock: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LogCfg {
    pub level: String,
}

impl Default for LogCfg {
    fn default() -> Self {
        LogCfg { level: "info".into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub role: RoleCfg,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub network: NetworkCfg,
    #[serde(default)]
    pub screen: ScreenCfg,
    #[serde(default)]
    pub mouse: MouseCfg,
    #[serde(default)]
    pub scroll: ScrollCfg,
    #[serde(default)]
    pub buttons: ButtonsCfg,
    #[serde(default)]
    pub keyboard: KeyboardCfg,
    #[serde(default)]
    pub clipboard: ClipboardCfg,
    #[serde(default)]
    pub hotkeys: HotkeysCfg,
    #[serde(default)]
    pub log: LogCfg,
}

/// Point de connexion réseau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Listen(String),
    Connect(String),
}

/// Réglages validés et prêts à l'emploi.
#[derive(Debug, Clone)]
pub struct Settings {
    pub role: Role,
    pub name: String,
    pub endpoint: Endpoint,
    pub secret: String,
    pub edge_switching: bool,
    pub secondary_position: Edge,
    pub corner_size: i32,
    pub mouse_speed: f64,
    pub scroll: ScrollCfg,
    pub buttons: ButtonsCfg,
    pub iso_swap: IsoSwap,
    pub media_keys: MediaKeys,
    /// Remappage HID → HID (côté machine pilotée).
    pub remap: BTreeMap<u32, u32>,
    pub clipboard: ClipboardCfg,
    pub switch_hotkey: Option<Hotkey>,
    pub return_hotkey: Option<Hotkey>,
    pub lock_hotkey: Option<Hotkey>,
    pub log_level: log::LevelFilter,
}

impl Config {
    pub fn parse(text: &str) -> Result<Config, String> {
        // TextEdit (macOS) remplace par défaut les guillemets droits par des
        // guillemets typographiques : on les accepte.
        let text: String = text
            .chars()
            .map(|c| match c {
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{00AB}' | '\u{00BB}' => '"',
                '\u{2018}' | '\u{2019}' => '\'',
                c => c,
            })
            .collect();
        toml::from_str(&text).map_err(|e| format!("configuration invalide : {e}"))
    }

    pub fn validate(&self) -> Result<Settings, String> {
        let secret = self.network.secret.trim().to_string();
        if secret.len() < 12 {
            return Err(
                "le secret partagé ([network] secret) doit faire au moins 12 caractères et être identique sur les deux machines"
                    .into(),
            );
        }
        let endpoint = if !self.network.connect.trim().is_empty() {
            Endpoint::Connect(with_default_port(self.network.connect.trim()))
        } else {
            let l = self.network.listen.trim();
            let l = if l.is_empty() { format!("0.0.0.0:{DEFAULT_PORT}") } else { l.to_string() };
            Endpoint::Listen(l)
        };
        if !(0.05..=20.0).contains(&self.mouse.speed) {
            return Err("[mouse] speed doit être compris entre 0.05 et 20".into());
        }
        if !(0.1..=50.0).contains(&self.scroll.lines_per_notch) {
            return Err("[scroll] lines_per_notch doit être compris entre 0.1 et 50".into());
        }
        if !(1.0..=2000.0).contains(&self.scroll.pixels_per_notch) {
            return Err("[scroll] pixels_per_notch doit être compris entre 1 et 2000".into());
        }
        let mut remap = BTreeMap::new();
        for (from, to) in &self.keyboard.remap {
            let f = crate::keymap::hid_from_name(from)
                .ok_or_else(|| format!("[keyboard.remap] touche inconnue « {from} »"))?;
            let t =
                crate::keymap::hid_from_name(to).ok_or_else(|| format!("[keyboard.remap] touche inconnue « {to} »"))?;
            remap.insert(f, t);
        }
        let switch_hotkey = Hotkey::parse(&self.hotkeys.switch_to_secondary)
            .map_err(|e| format!("[hotkeys] switch_to_secondary : {e}"))?;
        let return_hotkey =
            Hotkey::parse(&self.hotkeys.return_to_primary).map_err(|e| format!("[hotkeys] return_to_primary : {e}"))?;
        let lock_hotkey =
            Hotkey::parse(&self.hotkeys.toggle_lock).map_err(|e| format!("[hotkeys] toggle_lock : {e}"))?;
        if self.role == RoleCfg::Primary
            && !self.screen.edge_switching
            && (switch_hotkey.is_none() || return_hotkey.is_none())
        {
            return Err("sans bascule par les bords ([screen] edge_switching = false), les raccourcis \
                 [hotkeys] switch_to_secondary et return_to_primary sont obligatoires"
                .into());
        }
        let log_level = match self.log.level.trim().to_ascii_lowercase().as_str() {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" | "" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            other => return Err(format!("[log] level inconnu « {other} »")),
        };
        let name = if self.name.trim().is_empty() { hostname() } else { self.name.trim().to_string() };
        Ok(Settings {
            role: self.role.into(),
            name,
            endpoint,
            secret,
            edge_switching: self.screen.edge_switching,
            secondary_position: self.screen.secondary_position.into(),
            corner_size: self.screen.corner_size.max(0),
            mouse_speed: self.mouse.speed,
            scroll: self.scroll.clone(),
            buttons: self.buttons.clone(),
            iso_swap: self.keyboard.iso_swap,
            media_keys: self.keyboard.media_keys,
            remap,
            clipboard: self.clipboard.clone(),
            switch_hotkey,
            return_hotkey,
            lock_hotkey,
            log_level,
        })
    }
}

/// Ajoute le port par défaut à une adresse qui n'en a pas.
fn with_default_port(s: &str) -> String {
    use std::net::{IpAddr, SocketAddr};
    if s.parse::<SocketAddr>().is_ok() {
        return s.to_string();
    }
    if let Ok(ip) = s.parse::<IpAddr>() {
        return SocketAddr::new(ip, DEFAULT_PORT).to_string();
    }
    if let Some((host, port)) = s.rsplit_once(':') {
        if !host.contains(':') && port.parse::<u16>().is_ok() {
            return s.to_string();
        }
    }
    format!("{s}:{DEFAULT_PORT}")
}

/// Dossier de configuration de Bascule.
pub fn config_dir() -> PathBuf {
    dirs::config_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from(".")).join("Bascule")
}

pub fn default_config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Dossier des journaux.
pub fn log_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library").join("Logs").join("Bascule");
        }
    }
    config_dir()
}

/// Nom de la machine (pour l'affichage).
pub fn hostname() -> String {
    if let Ok(n) = std::env::var("COMPUTERNAME") {
        if !n.trim().is_empty() {
            return n.trim().to_string();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let mut buf = [0u8; 256];
        // SAFETY: le tampon est valide et sa taille est transmise.
        let r = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if r == 0 {
            let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
            let s = String::from_utf8_lossy(&buf[..end]).to_string();
            let s = s.trim_end_matches(".local").to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    if let Ok(n) = std::env::var("HOSTNAME") {
        if !n.trim().is_empty() {
            return n.trim().to_string();
        }
    }
    "machine".to_string()
}

/// Génère un secret aléatoire lisible (24 caractères, alphabet sans ambiguïté).
pub fn generate_secret() -> String {
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let mut bytes = [0u8; 24];
    getrandom::fill(&mut bytes).expect("générateur aléatoire du système");
    let s: String = bytes.iter().map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char).collect();
    // Groupes de 4 pour faciliter la recopie.
    s.as_bytes().chunks(4).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join("-")
}

/// Rôle proposé par défaut selon le système.
pub fn default_role() -> &'static str {
    if cfg!(target_os = "macos") { "secondary" } else { "primary" }
}

/// Fichier de configuration initial, commenté.
pub fn default_config_text(role: &str, secret: &str) -> String {
    TEMPLATE.replace("@@ROLE@@", role).replace("@@SECRET@@", secret).replace("@@PORT@@", &DEFAULT_PORT.to_string())
}

const TEMPLATE: &str = r##"# =====================================================================
#  Bascule — configuration
#  Après modification : menu Bascule > « Redémarrer ».
# =====================================================================

# Rôle de cette machine :
#   "primary"   = la machine qui a le clavier et la souris physiques
#   "secondary" = la machine pilotée à distance
role = "@@ROLE@@"

# Nom affiché (facultatif ; par défaut le nom de la machine).
name = ""

[network]
# Une machine ÉCOUTE, l'autre SE CONNECTE.
#  - Machine qui écoute : laisser `connect` vide.
#  - Machine qui se connecte : mettre l'adresse IP (ou le nom .local) de l'autre.
# Conseillé : le Mac écoute (connect vide) et le PC se connecte au Mac.
# Exemple sur le PC :  connect = "192.168.1.30"   (adresse IP du Mac)
listen = "0.0.0.0:@@PORT@@"
connect = ""

# Secret partagé : DOIT être identique sur les deux machines.
# Copiez cette valeur dans la configuration de l'autre machine.
secret = "@@SECRET@@"

[screen]
# (Machine principale) Passer sur l'autre machine en poussant la souris
# contre le bord de l'écran. false = uniquement avec les raccourcis
# clavier (section [hotkeys]).
edge_switching = true
# (Machine principale) Où se trouve l'écran de la machine secondaire,
# par rapport à la machine principale : "right", "left", "top" ou "bottom".
secondary_position = "right"
# Taille en pixels des coins où le passage d'un écran à l'autre est
# désactivé : on peut ainsi « lancer » la souris dans un coin (bouton ✕
# d'une fenêtre agrandie, « Afficher le bureau »…) sans changer d'écran.
# 0 = aucun.
corner_size = 10

[mouse]
# (Machine pilotée) Multiplicateur des déplacements reçus.
speed = 1.0

[scroll]
# (Machine pilotée) "lines" = par lignes, comme une molette Windows ;
# "pixels" = défilement précis au pixel.
mode = "lines"
# Nombre de lignes par cran de molette (mode "lines").
lines_per_notch = 3.0
# Nombre de pixels par cran de molette (mode "pixels").
pixels_per_notch = 60.0
# Inverser le sens de défilement.
invert_vertical = false
invert_horizontal = false

[buttons]
# (Machine pilotée, macOS) Boutons latéraux 4/5 de la souris :
#   "buttons" = transmis tels quels comme boutons 4/5
#               (Chrome, Firefox, Edge, VS Code, JetBrains…)
#   "swipe"   = transformés en geste « balayage » précédent/suivant, compris
#               par presque toutes les apps Mac (Safari, Finder, Xcode, Notes…)
side_buttons = "buttons"
# Inverser les boutons 4 et 5.
swap_side_buttons = false

[keyboard]
# (Machine pilotée, macOS) Échange des touches « ² » / « < > » pour les Mac à
# clavier ISO (claviers français). "auto" détecte le type de clavier du Mac.
# Si ces deux touches sont inversées, essayez "on" ou "off".
iso_swap = "auto"
# (Machine principale) Touches multimédia (volume, lecture…) :
#   "remote" = agissent sur la machine où se trouve le curseur
#   "local"  = agissent toujours sur la machine principale
media_keys = "remote"

# (Machine pilotée) Remappage de touches physiques, par nom ou code HID.
# Exemple : faire de Verr. Maj un Ctrl sur le Mac :
#   CapsLock = "ControlLeft"
[keyboard.remap]

[clipboard]
enabled = true
images = true
# Conserver la mise en forme (HTML) quand elle est disponible.
html = true
# Taille maximale transférée (Mo).
max_size_mb = 32
# Ne pas transférer les contenus marqués « sensibles » (gestionnaires de mots
# de passe…).
skip_sensitive = true

[hotkeys]
# Format : modificateurs (Ctrl, Shift, Alt, Win / Cmd) puis une touche,
# séparés par « + ». Exemples : "Shift+F12", "Ctrl+Alt+B". Vide = désactivé.
#
# (Machine principale) Envoie le curseur sur la machine secondaire, là où
# il était la dernière fois.
switch_to_secondary = "Shift+F12"
# (Machine principale) Ramène le curseur sur la machine principale.
# Astuce : le même raccourci que ci-dessus permet d'aller et venir.
return_to_primary = "Shift+F11"
# (Machine principale) Verrouille / déverrouille le curseur sur l'écran courant.
# Exemple : "ScrollLock". Vide = désactivé.
toggle_lock = ""

[log]
# "error", "warn", "info", "debug" ou "trace".
level = "info"
"##;

/// Charge la configuration ; la crée avec des valeurs par défaut si absente.
/// Renvoie aussi `true` si le fichier vient d'être créé.
pub fn load_or_create(path: &Path) -> Result<(Config, bool), String> {
    if !path.exists() {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("impossible de créer {} : {e}", dir.display()))?;
        }
        let text = default_config_text(default_role(), &generate_secret());
        std::fs::write(path, &text).map_err(|e| format!("impossible d'écrire {} : {e}", path.display()))?;
        let cfg = Config::parse(&text)?;
        return Ok((cfg, true));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("impossible de lire {} : {e}", path.display()))?;
    // Tolère un BOM UTF-8 (fréquent avec le Bloc-notes).
    let text = text.trim_start_matches('\u{feff}');
    Ok((Config::parse(text)?, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_parses_and_validates() {
        for role in ["primary", "secondary"] {
            let text = default_config_text(role, &generate_secret());
            let cfg = Config::parse(&text).unwrap();
            let s = cfg.validate().unwrap();
            assert_eq!(s.endpoint, Endpoint::Listen(format!("0.0.0.0:{DEFAULT_PORT}")));
            assert_eq!(s.secondary_position, Edge::Right);
            assert!(s.edge_switching);
            assert_eq!(s.switch_hotkey, Hotkey::parse("Shift+F12").unwrap());
            assert_eq!(s.return_hotkey, Hotkey::parse("Shift+F11").unwrap());
            assert!(s.lock_hotkey.is_none());
            assert_eq!(s.scroll.mode, ScrollMode::Lines);
        }
    }

    #[test]
    fn missing_sections_use_defaults() {
        let s = Config::parse("role = \"primary\"\n[network]\nsecret = \"abcd-efgh-jkmn-pqrs\"\n")
            .unwrap()
            .validate()
            .unwrap();
        assert!(s.edge_switching);
        assert!(s.switch_hotkey.is_some() && s.return_hotkey.is_some());
    }

    #[test]
    fn hotkeys_required_without_edge_switching() {
        let base = default_config_text("primary", "abcd-efgh-jkmn-pqrs")
            .replace("edge_switching = true", "edge_switching = false");
        assert!(!Config::parse(&base).unwrap().validate().unwrap().edge_switching);
        let no_switch = base.replace("switch_to_secondary = \"Shift+F12\"", "switch_to_secondary = \"\"");
        assert!(Config::parse(&no_switch).unwrap().validate().is_err());
        let no_return = base.replace("return_to_primary = \"Shift+F11\"", "return_to_primary = \"\"");
        assert!(Config::parse(&no_return).unwrap().validate().is_err());
        // Sans importance sur la machine pilotée.
        let secondary = no_switch.replace("role = \"primary\"", "role = \"secondary\"");
        assert!(Config::parse(&secondary).unwrap().validate().is_ok());
    }

    #[test]
    fn connect_adds_default_port() {
        let text = default_config_text("secondary", "abcd-efgh-jkmn-pqrs")
            .replace("connect = \"\"", "connect = \"192.168.1.20\"");
        let s = Config::parse(&text).unwrap().validate().unwrap();
        assert_eq!(s.endpoint, Endpoint::Connect(format!("192.168.1.20:{DEFAULT_PORT}")));
        let text = default_config_text("secondary", "abcd-efgh-jkmn-pqrs")
            .replace("connect = \"\"", "connect = \"pc.local:4000\"");
        let s = Config::parse(&text).unwrap().validate().unwrap();
        assert_eq!(s.endpoint, Endpoint::Connect("pc.local:4000".into()));
    }

    #[test]
    fn remap_and_errors() {
        let text = default_config_text("secondary", "abcd-efgh-jkmn-pqrs")
            .replace("[keyboard.remap]\n", "[keyboard.remap]\nCapsLock = \"ControlLeft\"\n");
        let s = Config::parse(&text).unwrap().validate().unwrap();
        assert_eq!(s.remap.get(&crate::keymap::HID_CAPS_LOCK), Some(&crate::keymap::HID_CONTROL_LEFT));

        let bad = default_config_text("secondary", "court");
        assert!(Config::parse(&bad).unwrap().validate().is_err());
        let typo = default_config_text("primary", "abcd-efgh-jkmn-pqrs").replace("[mouse]\n", "[mouse]\nsped = 2\n");
        assert!(Config::parse(&typo).is_err());
    }

    #[test]
    fn smart_quotes_are_accepted() {
        let text = default_config_text("primary", "abcd-efgh-jkmn-pqrs")
            .replace("connect = \"\"", "connect = \u{201C}192.168.1.30\u{201D}");
        let s = Config::parse(&text).unwrap().validate().unwrap();
        assert_eq!(s.endpoint, Endpoint::Connect(format!("192.168.1.30:{DEFAULT_PORT}")));
    }

    #[test]
    fn secret_format() {
        let s = generate_secret();
        assert_eq!(s.len(), 24 + 5);
        assert_ne!(s, generate_secret());
    }
}
