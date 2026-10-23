//! Configuration (fichier TOML commenté).
//!
//! Le même format sert sur les deux machines ; chaque machine ignore les
//! sections qui ne la concernent pas. Tous les réglages se changent sans
//! recompiler : il suffit de redémarrer Baskul (menu « Redémarrer »).

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
    /// Défilement fluide : chaque cran est animé, au pixel (macOS).
    Smooth,
}

/// Défilement fluide : distance parcourue par cran (réglage « Vitesse » de
/// Mac Mouse Fix).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmoothSpeed {
    Low,
    #[default]
    Medium,
    High,
}

/// Défilement fluide : durée de l'élan (réglage « Douceur » de Mac Mouse Fix).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Smoothness {
    /// Animation courte, freinage rapide (« Normale »).
    #[default]
    Regular,
    /// Élan plus long, comme un trackpad (« Élevée »).
    High,
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
    #[serde(alias = "remote")]
    Cursor,
    /// Elles agissent toujours sur la machine principale.
    #[serde(alias = "local")]
    Primary,
    /// Elles agissent toujours sur la machine pilotée.
    Secondary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkCfg {
    /// (Machine pilotée) Adresse de la machine principale ; vide : découverte
    /// automatique.
    pub connect: String,
    /// (Machine principale) Adresse et port d'écoute.
    pub listen: String,
    /// Carte réseau à utiliser (nom donné par le système) ; vide : automatique.
    pub interface: String,
    /// Découverte automatique (Bonjour) : la principale s'annonce, les
    /// pilotées la trouvent.
    pub discovery: bool,
    pub secret: String,
}

impl Default for NetworkCfg {
    fn default() -> Self {
        NetworkCfg {
            connect: String::new(),
            listen: format!("0.0.0.0:{DEFAULT_PORT}"),
            interface: String::new(),
            discovery: true,
            secret: String::new(),
        }
    }
}

/// (Machine principale) Grille des machines.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutCfg {
    /// Pilotées placées sur la grille ; la principale est en (0, 0).
    pub machines: Vec<MachineCfg>,
}

/// Une pilotée sur la grille.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineCfg {
    /// Nom de la pilotée (son « Nom affiché »).
    pub name: String,
    /// Case : `x` vers la droite, `y` vers le bas.
    pub x: i32,
    pub y: i32,
    /// Raccourci pour aller directement sur cette pilotée (facultatif).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hotkey: String,
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
pub struct ActivityCfg {
    /// Activité partagée : cette machine reste active (écran allumé, statut
    /// « actif » dans les messageries) tant que l'utilisateur l'est sur
    /// l'autre.
    pub mirror: bool,
}

impl Default for ActivityCfg {
    fn default() -> Self {
        ActivityCfg { mirror: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpdatesCfg {
    /// Recherche les nouvelles versions (au démarrage puis toutes les six
    /// heures) et les télécharge ; un redémarrage les installe.
    pub check: bool,
}

impl Default for UpdatesCfg {
    fn default() -> Self {
        UpdatesCfg { check: true }
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
    pub smooth_speed: SmoothSpeed,
    pub smoothness: Smoothness,
    /// Un cran isolé ne fait défiler que 10 pixels (option « Précision » de
    /// Mac Mouse Fix).
    pub smooth_precision: bool,
    /// Douceur élevée : défilement simulé comme sur un trackpad (gestes et
    /// élan), comme l'option de Mac Mouse Fix.
    pub trackpad_simulation: bool,
    pub invert_vertical: bool,
    pub invert_horizontal: bool,
}

impl Default for ScrollCfg {
    fn default() -> Self {
        ScrollCfg {
            mode: ScrollMode::Lines,
            lines_per_notch: 3.0,
            pixels_per_notch: 60.0,
            smooth_speed: SmoothSpeed::Medium,
            smoothness: Smoothness::Regular,
            smooth_precision: false,
            trackpad_simulation: false,
            invert_vertical: false,
            invert_horizontal: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ButtonsCfg {
    pub side_buttons: SideButtonsMode,
    /// Ancien réglage (inversion des boutons 4 et 5), retiré : encore accepté
    /// dans les fichiers existants, mais ignoré.
    #[serde(default, rename = "swap_side_buttons", skip_serializing)]
    _swap_side_buttons: serde::de::IgnoredAny,
}

impl Default for ButtonsCfg {
    fn default() -> Self {
        ButtonsCfg { side_buttons: SideButtonsMode::Buttons, _swap_side_buttons: serde::de::IgnoredAny }
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
        KeyboardCfg { iso_swap: IsoSwap::Auto, media_keys: MediaKeys::Cursor, remap: BTreeMap::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClipboardCfg {
    pub enabled: bool,
    pub images: bool,
    pub html: bool,
    /// Fichiers et dossiers copiés (Finder, Explorateur…).
    pub files: bool,
    /// Taille maximale du texte et des images (Mo).
    pub max_size_mb: u32,
    /// Taille maximale des fichiers copiés (Mo).
    pub max_files_mb: u32,
    pub skip_sensitive: bool,
}

impl Default for ClipboardCfg {
    fn default() -> Self {
        ClipboardCfg {
            enabled: true,
            images: true,
            html: true,
            files: true,
            max_size_mb: 32,
            max_files_mb: 256,
            skip_sensitive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HotkeysCfg {
    pub switch_to_secondary: String,
    pub return_to_primary: String,
    pub next_machine: String,
    pub toggle_lock: String,
}

impl Default for HotkeysCfg {
    fn default() -> Self {
        HotkeysCfg {
            switch_to_secondary: "Shift+F12".into(),
            return_to_primary: "Shift+F11".into(),
            next_machine: String::new(),
            toggle_lock: String::new(),
        }
    }
}

/// Thème de l'app de réglages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// Langue de Baskul (app de réglages et menus).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    System,
    Fr,
    En,
}

/// Style de l'icône de la barre des menus / de la zone de notification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrayIcon {
    /// En couleurs (l'état de la connexion se lit à la couleur).
    #[default]
    Color,
    /// Monochrome, comme les autres icônes du système.
    Mono,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiCfg {
    pub theme: Theme,
    pub language: Language,
    pub tray_icon: TrayIcon,
    /// Bulle d'arrivée : le nom de la machine s'affiche un instant près du
    /// curseur quand le clavier et la souris arrivent sur elle.
    pub arrival_bubble: bool,
}

impl Default for UiCfg {
    fn default() -> Self {
        UiCfg {
            theme: Theme::default(),
            language: Language::default(),
            tray_icon: TrayIcon::default(),
            arrival_bubble: true,
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
    pub layout: LayoutCfg,
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
    pub activity: ActivityCfg,
    #[serde(default)]
    pub hotkeys: HotkeysCfg,
    #[serde(default)]
    pub ui: UiCfg,
    #[serde(default)]
    pub updates: UpdatesCfg,
    #[serde(default)]
    pub log: LogCfg,
}

/// Une pilotée placée sur la grille (réglages validés).
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub name: String,
    pub cell: (i32, i32),
    pub hotkey: Option<Hotkey>,
}

/// Réglages validés et prêts à l'emploi.
#[derive(Debug, Clone)]
pub struct Settings {
    pub role: Role,
    pub name: String,
    /// (Machine principale) Adresse d'écoute.
    pub listen: String,
    /// (Machine pilotée) Adresse de la principale (`None` : découverte).
    pub connect: Option<String>,
    /// Carte réseau imposée.
    pub interface: Option<String>,
    pub discovery: bool,
    pub secret: String,
    pub layout: Vec<Placement>,
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
    /// Cette machine reste active tant que l'utilisateur l'est sur l'autre.
    pub mirror_activity: bool,
    pub check_updates: bool,
    pub switch_hotkey: Option<Hotkey>,
    pub return_hotkey: Option<Hotkey>,
    pub next_hotkey: Option<Hotkey>,
    pub lock_hotkey: Option<Hotkey>,
    pub ui: UiCfg,
    pub log_level: log::LevelFilter,
}

impl Config {
    pub fn parse(text: &str) -> Result<Config, String> {
        toml::from_str(&straight_quotes(text)).map_err(|e| format!("configuration invalide : {e}"))
    }

    pub fn validate(&self) -> Result<Settings, String> {
        let secret = self.network.secret.trim().to_string();
        if secret.len() < 12 {
            return Err(
                "le secret partagé ([network] secret) doit faire au moins 12 caractères et être identique sur les deux machines"
                    .into(),
            );
        }
        let connect = Some(self.network.connect.trim()).filter(|c| !c.is_empty()).map(with_default_port);
        let listen = match self.network.listen.trim() {
            "" => format!("0.0.0.0:{DEFAULT_PORT}"),
            l => l.to_string(),
        };
        let interface = Some(self.network.interface.trim().to_string()).filter(|i| !i.is_empty());
        let layout = self.layout.validate()?;
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
        let next_hotkey =
            Hotkey::parse(&self.hotkeys.next_machine).map_err(|e| format!("[hotkeys] next_machine : {e}"))?;
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
            listen,
            connect,
            interface,
            discovery: self.network.discovery,
            secret,
            layout,
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
            mirror_activity: self.activity.mirror,
            check_updates: self.updates.check,
            switch_hotkey,
            return_hotkey,
            next_hotkey,
            lock_hotkey,
            ui: self.ui.clone(),
            log_level,
        })
    }
}

impl LayoutCfg {
    fn validate(&self) -> Result<Vec<Placement>, String> {
        let mut out: Vec<Placement> = Vec::new();
        for m in &self.machines {
            let name = m.name.trim();
            if name.is_empty() {
                return Err("[layout] une machine n'a pas de nom".into());
            }
            if (m.x, m.y) == (0, 0) {
                return Err(format!("[layout] « {name} » : la case (0, 0) est celle de la machine principale"));
            }
            if out.iter().any(|p| p.name == name) {
                return Err(format!("[layout] « {name} » est placée deux fois"));
            }
            if let Some(p) = out.iter().find(|p| p.cell == (m.x, m.y)) {
                return Err(format!("[layout] « {name} » et « {} » sont sur la même case", p.name));
            }
            let hotkey = Hotkey::parse(&m.hotkey).map_err(|e| format!("[layout] « {name} », hotkey : {e}"))?;
            out.push(Placement { name: name.to_string(), cell: (m.x, m.y), hotkey });
        }
        Ok(out)
    }
}

/// Première case libre à côté de la principale pour une nouvelle pilotée :
/// du côté `preferred`, puis les autres côtés, puis de plus en plus loin.
pub fn free_cell(layout: &[MachineCfg], preferred: Edge) -> (i32, i32) {
    let dir = |e: Edge| match e {
        Edge::Right => (1, 0),
        Edge::Left => (-1, 0),
        Edge::Top => (0, -1),
        Edge::Bottom => (0, 1),
    };
    let order = [preferred, preferred.opposite(), Edge::Right, Edge::Left, Edge::Bottom, Edge::Top];
    let taken = |c: (i32, i32)| c == (0, 0) || layout.iter().any(|m| (m.x, m.y) == c);
    for dist in 1.. {
        for e in order {
            let (dx, dy) = dir(e);
            let c = (dx * dist, dy * dist);
            if !taken(c) {
                return c;
            }
        }
    }
    unreachable!()
}

/// Enregistre `cfg` dans le fichier `path` en gardant ses commentaires.
pub fn write_config(path: &Path, cfg: &Config) -> Result<(), String> {
    cfg.validate()?;
    let old = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("impossible de lire {} : {e}", path.display())),
    };
    let text = update_text(&old, cfg)?;
    // Le fichier écrit doit redonner exactement la configuration voulue.
    let written = serde_json::to_value(Config::parse(&text)?).map_err(|e| e.to_string())?;
    if written != serde_json::to_value(cfg).map_err(|e| e.to_string())? {
        return Err("erreur interne : le fichier produit ne correspond pas aux réglages".into());
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("impossible d'écrire {} : {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("impossible d'écrire {} : {e}", path.display()))
}

/// Identifiant de cette machine, créé au premier lancement (tests : un
/// nouvel identifiant à chaque appel, sans rien écrire).
#[cfg(test)]
pub fn machine_id() -> String {
    let mut b = [0u8; 16];
    let _ = getrandom::fill(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Identifiant de cette machine, créé au premier lancement.
#[cfg(not(test))]
pub fn machine_id() -> String {
    let path = config_dir().join("machine-id");
    if let Ok(id) = std::fs::read_to_string(&path) {
        let id = id.trim();
        if !id.is_empty() {
            return id.to_string();
        }
    }
    let mut b = [0u8; 16];
    let _ = getrandom::fill(&mut b);
    let id: String = b.iter().map(|x| format!("{x:02x}")).collect();
    let _ = std::fs::create_dir_all(config_dir());
    let _ = std::fs::write(&path, &id);
    id
}

/// TextEdit (macOS) remplace par défaut les guillemets droits par des
/// guillemets typographiques : on les accepte.
fn straight_quotes(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{00AB}' | '\u{00BB}' => '"',
            '\u{2018}' | '\u{2019}' => '\'',
            c => c,
        })
        .collect()
}

/// Écrit les valeurs de `cfg` dans le texte TOML `old` en conservant ses
/// commentaires et sa mise en forme. Les clés absentes de `old` sont
/// ajoutées avec le commentaire du modèle.
pub fn update_text(old: &str, cfg: &Config) -> Result<String, String> {
    let old = old.trim_start_matches('\u{feff}');
    let template = default_config_text(default_role(), "");
    let old = if old.trim().is_empty() { template.as_str() } else { old };
    // Les guillemets typographiques ne sont remplacés que s'il le faut : les
    // « » des commentaires restent intacts.
    let mut doc: toml_edit::DocumentMut =
        old.parse().or_else(|_| straight_quotes(old).parse()).map_err(|e| format!("configuration invalide : {e}"))?;
    let new_text = toml::to_string(cfg).map_err(|e| format!("conversion de la configuration : {e}"))?;
    let new: toml_edit::DocumentMut = new_text.parse().map_err(|e| format!("conversion de la configuration : {e}"))?;
    let template: toml_edit::DocumentMut = template.parse().map_err(|e| format!("modèle invalide : {e}"))?;
    merge_table(doc.as_table_mut(), new.as_table(), Some(template.as_table()));
    Ok(doc.to_string())
}

fn merge_table(old: &mut toml_edit::Table, new: &toml_edit::Table, template: Option<&toml_edit::Table>) {
    use toml_edit::Item;
    for (key, new_item) in new.iter() {
        let tmpl = template.and_then(|t| t.get(key));
        match (old.get_mut(key), new_item) {
            (Some(Item::Table(o)), Item::Table(n)) => merge_table(o, n, tmpl.and_then(Item::as_table)),
            // Liste de tables (grille des machines) : écrite en tableau en ligne,
            // une table par ligne, pour garder le commentaire au-dessus.
            (Some(Item::Value(o)), Item::ArrayOfTables(n)) => {
                let decor = o.decor().clone();
                *o = inline_array(n);
                *o.decor_mut() = decor;
            }
            (Some(Item::Value(o)), Item::Value(n)) => {
                // Valeur inchangée : on garde son écriture d'origine.
                if bare(o) != bare(n) {
                    let decor = o.decor().clone();
                    *o = n.clone();
                    *o.decor_mut() = decor;
                }
            }
            (Some(slot), _) => *slot = new_item.clone(),
            (None, _) => {
                let mut item = match new_item {
                    Item::ArrayOfTables(n) => Item::Value(inline_array(n)),
                    other => other.clone(),
                };
                match (&mut item, tmpl) {
                    (Item::Value(v), Some(Item::Value(t))) => *v.decor_mut() = t.decor().clone(),
                    (Item::Table(v), Some(Item::Table(t))) => *v.decor_mut() = t.decor().clone(),
                    _ => {}
                }
                match template.and_then(|t| t.key(key)) {
                    Some(k) => old.insert_formatted(k, item),
                    None => old.insert(key, item),
                };
            }
        }
    }
    // Valeurs supprimées (règles de [keyboard.remap]…). Les tables restent,
    // avec leurs commentaires.
    let stale: Vec<String> =
        old.iter().filter(|(k, v)| v.is_value() && !new.contains_key(k)).map(|(k, _)| k.to_string()).collect();
    for k in stale {
        old.remove(&k);
    }
}

fn inline_array(tables: &toml_edit::ArrayOfTables) -> toml_edit::Value {
    let mut arr = toml_edit::Array::new();
    for t in tables.iter() {
        let mut v = toml_edit::Value::InlineTable(t.clone().into_inline_table());
        v.decor_mut().set_prefix("\n  ");
        arr.push_formatted(v);
    }
    arr.set_trailing("\n");
    arr.set_trailing_comma(true);
    toml_edit::Value::Array(arr)
}

/// Écriture d'une valeur, sans espaces ni commentaire autour.
fn bare(v: &toml_edit::Value) -> String {
    let mut v = v.clone();
    v.decor_mut().clear();
    v.to_string()
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

/// Dossier de configuration de Baskul.
pub fn config_dir() -> PathBuf {
    config_base().join("Baskul")
}

fn config_base() -> PathBuf {
    dirs::config_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from("."))
}

/// Ancien nom de l'app (versions 0.6 et précédentes).
pub const OLD_NAME: &str = "Bascule";

/// Reprend les réglages de l'ancien nom (dossier « Bascule ») tant qu'il n'y
/// a pas de dossier « Baskul ». Refuse si l'ancienne version tourne encore
/// (elle tient son verrou d'instance).
pub fn migrate_old_config_dir() -> Result<bool, String> {
    migrate_dir(&config_base().join(OLD_NAME), &config_dir())
}

fn migrate_dir(old: &Path, new: &Path) -> Result<bool, String> {
    if new.exists() || !old.is_dir() {
        return Ok(false);
    }
    if let Ok(lock) = std::fs::File::options().write(true).open(old.join("bascule.lock")) {
        if matches!(lock.try_lock(), Err(std::fs::TryLockError::WouldBlock)) {
            return Err(format!(
                "L'ancienne version ({OLD_NAME}) est encore lancée : quittez-la, puis relancez Baskul."
            ));
        }
    }
    std::fs::rename(old, new).map_err(|e| format!("reprise des réglages de {} impossible : {e}", old.display()))?;
    // Fichiers propres à l'ancienne instance.
    for f in ["bascule.lock", "api.json"] {
        let _ = std::fs::remove_file(new.join(f));
    }
    Ok(true)
}

pub fn default_config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Dossier des journaux.
pub fn log_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        if let Some(home) = dirs::home_dir() {
            return home.join("Library").join("Logs").join("Baskul");
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
#  Baskul — configuration
#  Après modification : menu Baskul > « Redémarrer ».
# =====================================================================

# Rôle de cette machine :
#   "primary"   = la machine qui a le clavier et la souris physiques
#   "secondary" = la machine pilotée à distance
role = "@@ROLE@@"

# Nom affiché (facultatif ; par défaut le nom de la machine).
name = ""

[network]
# La machine principale attend les connexions ; chaque machine pilotée se
# connecte à elle.
#
# (Machine pilotée) Adresse de la machine principale : adresse IP ou nom
# .local, port @@PORT@@ par défaut. Exemple : connect = "192.168.1.20".
# Vide : la principale est trouvée automatiquement sur le réseau local.
connect = ""
# (Machine principale) Adresse et port d'écoute.
listen = "0.0.0.0:@@PORT@@"
# Carte réseau à utiliser : nom donné par le système ("en0", "Ethernet",
# "Wi-Fi"…). Vide : automatique.
interface = ""
# Découverte automatique (Bonjour) : la principale s'annonce sur le réseau
# local, les pilotées la trouvent sans adresse à saisir.
discovery = true

# Secret partagé : DOIT être identique sur toutes les machines.
# Copiez cette valeur dans la configuration des autres machines.
secret = "@@SECRET@@"

[screen]
# (Machine principale) Passer sur l'autre machine en poussant la souris
# contre le bord de l'écran. false = uniquement avec les raccourcis
# clavier (section [hotkeys]).
edge_switching = true
# (Machine principale) Côté où se place une nouvelle pilotée sur la grille
# (section [layout]) : "right", "left", "top" ou "bottom".
secondary_position = "right"
# Taille en pixels des coins où le passage d'un écran à l'autre est
# désactivé : on peut ainsi « lancer » la souris dans un coin (bouton ✕
# d'une fenêtre agrandie, « Afficher le bureau »…) sans changer d'écran.
# 0 = aucun.
corner_size = 10

[layout]
# (Machine principale) Grille des machines. La principale est en (0, 0) ;
# x augmente vers la droite, y vers le bas. On passe d'une machine à sa
# voisine par le bord commun de leurs écrans. Une pilotée qui se connecte
# pour la première fois est placée d'elle-même à côté de la principale.
# « hotkey » (facultatif) : raccourci pour aller directement sur elle.
# Exemple : machines = [{ name = "Atelier", x = 1, y = 0, hotkey = "Ctrl+Alt+1" }]
machines = []

[mouse]
# (Machine pilotée) Multiplicateur des déplacements reçus.
speed = 1.0

[scroll]
# (Machine pilotée) "lines" = par lignes, comme une molette Windows ;
# "pixels" = défilement précis au pixel ; "smooth" = défilement fluide à la
# manière de Mac Mouse Fix (macOS).
mode = "lines"
# Nombre de lignes par cran de molette (mode "lines").
lines_per_notch = 3.0
# Nombre de pixels par cran de molette (mode "pixels").
pixels_per_notch = 60.0
# Mode "smooth" : mêmes réglages que Mac Mouse Fix.
# Vitesse : "low", "medium" ou "high" (les crans rapides vont plus loin).
smooth_speed = "medium"
# Douceur : "regular" (animation courte) ou "high" (élan plus long).
smoothness = "regular"
# Précision : un cran isolé ne fait défiler que 10 pixels ; les crans
# rapprochés accélèrent toujours.
smooth_precision = false
# Douceur "high" : simulation du trackpad (gestes et élan, rebond en bout de
# page…), comme l'option de Mac Mouse Fix.
trackpad_simulation = false
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

[keyboard]
# (Machine pilotée, macOS) Échange des touches « ² » / « < > » pour les Mac à
# clavier ISO (claviers français). "auto" détecte le type de clavier du Mac.
# Si ces deux touches sont inversées, essayez "on" ou "off".
iso_swap = "auto"
# (Machine principale) Touches multimédia (volume, lecture…) :
#   "cursor"    = agissent sur la machine où se trouve le curseur
#   "primary"   = agissent toujours sur la machine principale
#   "secondary" = agissent toujours sur la machine pilotée
media_keys = "cursor"

# (Machine pilotée) Remappage de touches physiques, par nom ou code HID.
# Exemple : faire de Verr. Maj un Ctrl sur le Mac :
#   CapsLock = "ControlLeft"
[keyboard.remap]

[clipboard]
enabled = true
# Images, dans leur format d'origine (PNG, JPEG, GIF, WebP, HEIC…).
images = true
# Conserver la mise en forme (HTML) quand elle est disponible.
html = true
# Fichiers et dossiers copiés (Finder, Explorateur…) : coller sur une autre
# machine les y copie.
files = true
# Taille maximale du texte et des images (Mo).
max_size_mb = 32
# Taille maximale des fichiers copiés (Mo).
max_files_mb = 256
# Ne pas transférer les contenus marqués « sensibles » (gestionnaires de mots
# de passe…).
skip_sensitive = true

[activity]
# Activité partagée : tant que vous êtes actif sur l'autre machine, celle-ci
# reste active elle aussi (pas de veille de l'écran ni d'économiseur, statut
# « actif » dans Teams, Discord, Slack…). Quand vous vous absentez, les deux
# machines le voient.
mirror = true

[hotkeys]
# Format : modificateurs (Ctrl, Shift, Alt, Win / Cmd) puis une touche,
# séparés par « + ». Exemples : "Shift+F12", "Ctrl+Alt+B". Vide = désactivé.
#
# (Machine principale) Envoie le curseur sur la dernière pilotée utilisée,
# là où il était.
switch_to_secondary = "Shift+F12"
# (Machine principale) Ramène le curseur sur la machine principale.
# Astuce : le même raccourci que ci-dessus permet d'aller et venir.
return_to_primary = "Shift+F11"
# (Machine principale) Passe à la machine suivante : la principale, puis les
# pilotées connectées, dans l'ordre de la grille. Vide = désactivé.
next_machine = ""
# (Machine principale) Verrouille / déverrouille le curseur sur l'écran courant.
# Exemple : "ScrollLock". Vide = désactivé.
toggle_lock = ""

[ui]
# Thème de l'app de Baskul : "system", "light" (clair) ou "dark" (sombre).
theme = "system"
# Langue : "system" (celle du système), "fr" ou "en".
language = "system"
# Icône de la barre des menus / de la zone de notification : "color" ou
# "mono" (monochrome, assortie aux autres icônes du système).
tray_icon = "color"
# Bulle d'arrivée : le nom de la machine s'affiche un instant près du curseur
# quand le clavier et la souris arrivent sur elle.
arrival_bubble = true

[updates]
# Recherche les nouvelles versions de Baskul (au démarrage puis toutes les six
# heures) et les télécharge en arrière-plan ; « Redémarrer pour mettre à
# jour » (menu de l'icône ou app de réglages) les installe.
check = true

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
            assert_eq!((s.listen.as_str(), s.connect.as_deref()), (format!("0.0.0.0:{DEFAULT_PORT}").as_str(), None));
            assert!(s.discovery && s.interface.is_none() && s.layout.is_empty() && s.next_hotkey.is_none());
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
        assert_eq!(s.connect, Some(format!("192.168.1.20:{DEFAULT_PORT}")));
        let text = default_config_text("secondary", "abcd-efgh-jkmn-pqrs")
            .replace("connect = \"\"", "connect = \"pc.local:4000\"");
        let s = Config::parse(&text).unwrap().validate().unwrap();
        assert_eq!(s.connect.as_deref(), Some("pc.local:4000"));
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
        assert_eq!(s.connect, Some(format!("192.168.1.30:{DEFAULT_PORT}")));
    }

    fn edited(text: &str, f: impl FnOnce(&mut Config)) -> String {
        let mut cfg = Config::parse(text).unwrap();
        f(&mut cfg);
        let out = update_text(text, &cfg).unwrap();
        let back = Config::parse(&out).unwrap();
        assert_eq!(toml::to_string(&back).unwrap(), toml::to_string(&cfg).unwrap(), "relecture différente");
        out
    }

    #[test]
    fn update_keeps_comments_and_layout() {
        let text = default_config_text("primary", "abcd-efgh-jkmn-pqrs");
        // Aucune modification : le fichier ne change pas du tout.
        assert_eq!(edited(&text, |_| {}), text);

        let out = edited(&text, |c| {
            c.network.connect = "192.168.1.30".into();
            c.screen.edge_switching = false;
            c.mouse.speed = 1.5;
        });
        assert!(out.contains("connect = \"192.168.1.30\"\n"));
        assert!(out.contains("edge_switching = false\n"));
        assert!(out.contains("speed = 1.5\n"));
        assert!(out.contains("# Secret partagé : DOIT être identique sur toutes les machines."));
        assert_eq!(out.lines().count(), text.lines().count());
    }

    #[test]
    fn update_adds_missing_keys_with_template_comments() {
        // Fichier d'une version précédente, sans les nouvelles options.
        let old = default_config_text("secondary", "abcd-efgh-jkmn-pqrs")
            .replace("edge_switching = true\n", "")
            .replace("switch_to_secondary = \"Shift+F12\"\n", "")
            .replace("[mouse]\n", "[mouse]\n# Ma note\n");
        let out = edited(&old, |c| c.screen.edge_switching = false);
        assert!(out.contains("# Ma note\n"));
        assert!(out.contains("# contre le bord de l'écran. false = uniquement avec les raccourcis\n"));
        assert!(out.contains("edge_switching = false\n"));
        assert!(out.contains("switch_to_secondary = \"Shift+F12\"\n"));
    }

    #[test]
    fn update_minimal_file_and_remap() {
        let old = "role = \u{201C}secondary\u{201D}\n[network]\nsecret = \"abcd-efgh-jkmn-pqrs\"\n\
                   [keyboard.remap]\nCapsLock = \"ControlLeft\"\nF1 = \"F2\"\n";
        let out = edited(old, |c| {
            c.keyboard.remap.remove("F1");
            c.name = "Mac".into();
        });
        assert!(out.contains("CapsLock = \"ControlLeft\""));
        assert!(!out.contains("F1 = "));
        assert!(out.contains("name = \"Mac\""));
        // Fichier vide : on repart du modèle commenté.
        let cfg = Config::parse(&default_config_text("primary", "abcd-efgh-jkmn-pqrs")).unwrap();
        let out = update_text("", &cfg).unwrap();
        assert!(out.contains("role = \"primary\""));
        assert!(out.contains("# Rôle de cette machine"));
    }

    #[test]
    fn old_settings_still_accepted() {
        // Fichier d'une version précédente : ancienne clé et anciennes valeurs.
        let text = default_config_text("primary", "abcd-efgh-jkmn-pqrs")
            .replace("side_buttons = \"buttons\"\n", "side_buttons = \"buttons\"\nswap_side_buttons = true\n")
            .replace("media_keys = \"cursor\"", "media_keys = \"local\"");
        let cfg = Config::parse(&text).unwrap();
        assert_eq!(cfg.validate().unwrap().media_keys, MediaKeys::Primary);
        // À l'enregistrement, la clé retirée disparaît et la valeur est réécrite.
        let out = update_text(&text, &cfg).unwrap();
        assert!(!out.contains("swap_side_buttons"));
        assert!(out.contains("media_keys = \"primary\""));
        let remote = text.replace("media_keys = \"local\"", "media_keys = \"remote\"");
        assert_eq!(Config::parse(&remote).unwrap().keyboard.media_keys, MediaKeys::Cursor);
    }

    #[test]
    fn secret_format() {
        let s = generate_secret();
        assert_eq!(s.len(), 24 + 5);
        assert_ne!(s, generate_secret());
    }

    #[test]
    fn old_settings_folder_is_taken_over() {
        let base = std::env::temp_dir().join(format!("baskul-migration-{}", std::process::id()));
        let (old, new) = (base.join("Bascule"), base.join("Baskul"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("config.toml"), "role = \"primary\"").unwrap();
        std::fs::write(old.join("api.json"), "{}").unwrap();
        assert_eq!(migrate_dir(&old, &new), Ok(true));
        assert!(new.join("config.toml").is_file() && !new.join("api.json").exists() && !old.exists());
        // Déjà fait : rien à reprendre.
        assert_eq!(migrate_dir(&old, &new), Ok(false));
        // Ancienne version encore lancée : refus.
        std::fs::remove_dir_all(&new).unwrap();
        std::fs::create_dir_all(&old).unwrap();
        let lock = std::fs::File::create(old.join("bascule.lock")).unwrap();
        lock.lock().unwrap();
        assert!(migrate_dir(&old, &new).is_err());
        drop(lock);
        let _ = std::fs::remove_dir_all(&base);
    }

    fn machine(name: &str, x: i32, y: i32) -> MachineCfg {
        MachineCfg { name: name.into(), x, y, hotkey: String::new() }
    }

    #[test]
    fn layout_is_checked() {
        let ok = LayoutCfg { machines: vec![machine("Atelier", 1, 0), machine("Salon", 2, 0)] };
        let p = ok.validate().unwrap();
        assert_eq!((p[1].name.as_str(), p[1].cell), ("Salon", (2, 0)));
        for bad in [
            vec![machine("Atelier", 0, 0)],
            vec![machine("Atelier", 1, 0), machine("Atelier", 2, 0)],
            vec![machine("Atelier", 1, 0), machine("Salon", 1, 0)],
            vec![machine(" ", 1, 0)],
        ] {
            assert!(LayoutCfg { machines: bad }.validate().is_err());
        }
        let mut hk = machine("Atelier", 1, 0);
        hk.hotkey = "Ctrl+Alt+1".into();
        assert!(LayoutCfg { machines: vec![hk] }.validate().unwrap()[0].hotkey.is_some());
    }

    #[test]
    fn new_machines_go_next_to_the_primary() {
        assert_eq!(free_cell(&[], Edge::Right), (1, 0));
        assert_eq!(free_cell(&[machine("A", 1, 0)], Edge::Right), (-1, 0));
        let around = [machine("A", 1, 0), machine("B", -1, 0), machine("C", 0, 1), machine("D", 0, -1)];
        assert_eq!(free_cell(&around, Edge::Right), (2, 0));
        assert_eq!(free_cell(&[], Edge::Top), (0, -1));
    }

    #[test]
    fn layout_is_written_one_machine_per_line() {
        let text = default_config_text("primary", "abcd-efgh-jkmn-pqrs");
        let out = edited(&text, |c| {
            c.layout.machines =
                vec![machine("Atelier", 1, 0), MachineCfg { hotkey: "Ctrl+Alt+2".into(), ..machine("Salon", 2, 0) }]
        });
        assert!(out.contains("machines = [\n  { name = \"Atelier\", x = 1, y = 0 },\n  { name = \"Salon\", x = 2, y = 0, hotkey = \"Ctrl+Alt+2\" },\n]"), "{out}");
        assert!(out.contains("# (Machine principale) Grille des machines."));
        // Et de nouveau vide.
        let empty = edited(&out, |c| c.layout.machines.clear());
        assert!(empty.contains("machines = []"), "{empty}");
    }
}
