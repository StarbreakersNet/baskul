//! Icône de Baskul (zone de notification, barre des menus) : ce qu'elle
//! affiche et ce que fait son menu, quel que soit le système. L'app (Tauri)
//! la dessine avec [`model`] et transmet les choix du menu à [`act`].
//!
//! La couche de chaque système fournit ce qui lui est propre (style de
//! l'icône, lancement à l'ouverture de session, autorisations, exécution des
//! actions sur son propre thread) par un [`Backend`].

use crate::i18n;
use crate::icon::{self, Style, Tone};
use crate::protocol::{Role, SwitchTo};
use crate::{services, status};
use std::sync::{Mutex, OnceLock};

/// Choix du menu de l'icône.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Le clavier et la souris reviennent sur cette machine.
    ReturnHere,
    /// Le clavier et la souris passent sur cette pilotée.
    GoTo(String),
    /// Machine pilotée : le clavier et la souris repartent sur la machine
    /// principale.
    ReturnToPrimary,
    /// Applique un profil de réglages (Baskul redémarre).
    ApplyProfile(String),
    ToggleLock,
    Settings,
    OpenLog,
    OpenFolder,
    /// Lancement à l'ouverture de session.
    ToggleLogin,
    /// macOS : autorisation "Accessibilité".
    AllowAccessibility,
    InstallUpdate,
    Restart,
    Quit,
}

impl Action {
    /// Identifiant de l'entrée de menu.
    pub fn id(&self) -> String {
        match self {
            Action::ReturnHere => "return".into(),
            Action::GoTo(name) => format!("goto:{name}"),
            Action::ReturnToPrimary => "primary".into(),
            Action::ApplyProfile(name) => format!("profile:{name}"),
            Action::ToggleLock => "lock".into(),
            Action::Settings => "settings".into(),
            Action::OpenLog => "log".into(),
            Action::OpenFolder => "folder".into(),
            Action::ToggleLogin => "login".into(),
            Action::AllowAccessibility => "accessibility".into(),
            Action::InstallUpdate => "update".into(),
            Action::Restart => "restart".into(),
            Action::Quit => "quit".into(),
        }
    }

    pub fn from_id(id: &str) -> Option<Action> {
        if let Some(name) = id.strip_prefix("goto:") {
            return Some(Action::GoTo(name.to_string()));
        }
        if let Some(name) = id.strip_prefix("profile:") {
            return Some(Action::ApplyProfile(name.to_string()));
        }
        Some(match id {
            "return" => Action::ReturnHere,
            "primary" => Action::ReturnToPrimary,
            "lock" => Action::ToggleLock,
            "settings" => Action::Settings,
            "log" => Action::OpenLog,
            "folder" => Action::OpenFolder,
            "login" => Action::ToggleLogin,
            "accessibility" => Action::AllowAccessibility,
            "update" => Action::InstallUpdate,
            "restart" => Action::Restart,
            "quit" => Action::Quit,
            _ => return None,
        })
    }
}

/// Image de l'icône, en pixels RGBA (non prémultipliés).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub rgba: Vec<u8>,
    pub size: u32,
    /// macOS : image "modèle", colorée par le système comme les autres
    /// icônes de la barre des menus.
    pub template: bool,
}

/// Entrée du menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// Texte seul (état de la connexion), non cliquable.
    Label(String),
    /// Choix ; `checked` : case à cocher (cochée ou non).
    Item {
        action: Action,
        label: String,
        checked: Option<bool>,
    },
    Separator,
}

/// Ce qu'affiche l'icône.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub image: Image,
    pub tooltip: String,
    pub menu: Vec<Entry>,
}

/// Style de l'icône, choisi par le système.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconStyle {
    pub style: Style,
    /// Taille en pixels.
    pub size: u32,
    pub template: bool,
}

/// Ce que chaque système fournit à l'icône. Appelé depuis n'importe quel
/// thread.
pub trait Backend: Send + Sync {
    fn role(&self) -> Role;
    fn icon_style(&self) -> IconStyle;
    /// Libellé et état du lancement à l'ouverture de session, s'il existe.
    fn login_item(&self) -> Option<(String, bool)>;
    /// macOS : autorisation "Accessibilité" manquante.
    fn needs_accessibility(&self) -> bool {
        false
    }
    /// Exécute un choix du menu (sur le thread du système si besoin).
    fn perform(&self, action: Action);
}

static BACKEND: OnceLock<Box<dyn Backend>> = OnceLock::new();

type Listener = Box<dyn Fn() + Send + Sync>;
static LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());

/// La couche système démarre : l'icône peut s'afficher.
pub fn set_backend(backend: impl Backend + 'static) {
    if BACKEND.set(Box::new(backend)).is_ok() {
        status::on_change(changed);
    }
    changed();
}

/// Enregistre une fonction appelée quand l'icône doit être redessinée (état
/// de la connexion…). D'autres changements (mise à jour prête, lancement à
/// l'ouverture de session) ne sont pas signalés : l'app relit aussi
/// régulièrement le modèle.
pub fn on_change(f: impl Fn() + Send + Sync + 'static) {
    LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).push(Box::new(f));
}

/// Signale un changement de l'icône.
pub fn changed() {
    for l in LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        l();
    }
}

/// Ce que doit afficher l'icône ; `None` tant que Baskul ne tourne pas
/// (configuration invalide).
pub fn model() -> Option<Model> {
    let backend = BACKEND.get()?;
    Some(build(backend.as_ref(), &status::snapshot(), crate::update::ready_version(), &crate::profiles::menu()))
}

/// Machine pilotée : demande à la machine principale de reprendre le clavier
/// et la souris (comme `baskul switch primary`). Appelable depuis n'importe
/// quel thread.
pub fn return_to_primary() {
    if let Some((_, switch)) = services::switch_handler().as_ref() {
        if let Err(e) = switch(SwitchTo::Primary) {
            log::warn!("retour sur la machine principale impossible : {e}");
        }
    }
}

/// Choix du menu, par son identifiant.
pub fn act(id: &str) {
    match (BACKEND.get(), Action::from_id(id)) {
        (Some(backend), Some(action)) => backend.perform(action),
        _ => log::debug!("entrée de menu inconnue : {id}"),
    }
}

fn build(
    backend: &dyn Backend,
    snap: &status::Snapshot,
    update: Option<String>,
    (profiles, active): &(Vec<String>, Option<String>),
) -> Model {
    let style = backend.icon_style();
    let image = Image {
        rgba: icon::render(style.size, Tone::of(&snap.conn), snap.remote_active, style.style),
        size: style.size,
        template: style.template,
    };
    let item = |action: Action, label: String, checked: Option<bool>| Entry::Item { action, label, checked };
    let mut menu = vec![Entry::Label(snap.summary())];
    if backend.needs_accessibility() {
        menu.push(item(Action::AllowAccessibility, i18n::t("menu.allowAccessibility"), None));
    }
    if let Some(v) = update {
        menu.push(item(Action::InstallUpdate, i18n::tf("menu.updateReady", &[("version", &v)]), None));
    }
    menu.push(Entry::Separator);
    if backend.role() == Role::Primary {
        menu.push(item(Action::ReturnHere, i18n::t("menu.returnHere"), None));
        for p in &snap.peers {
            let active = snap.active_peer.as_deref() == Some(p.name.as_str());
            menu.push(item(Action::GoTo(p.name.clone()), i18n::tf("menu.goTo", &[("name", &p.name)]), Some(active)));
        }
        menu.push(item(Action::ToggleLock, i18n::t("menu.lock"), Some(snap.locked)));
        menu.push(Entry::Separator);
    } else if let Some(primary) = snap.peers.first().filter(|_| snap.remote_active) {
        menu.push(item(Action::ReturnToPrimary, i18n::tf("menu.returnTo", &[("name", &primary.name)]), None));
        menu.push(Entry::Separator);
    }
    for name in profiles {
        let label = i18n::tf("menu.profile", &[("name", name)]);
        menu.push(item(Action::ApplyProfile(name.clone()), label, Some(active.as_ref() == Some(name))));
    }
    if !profiles.is_empty() {
        menu.push(Entry::Separator);
    }
    menu.push(item(Action::Settings, i18n::t("menu.settings"), None));
    menu.push(item(Action::OpenLog, i18n::t("menu.openLog"), None));
    menu.push(item(Action::OpenFolder, i18n::t("menu.openFolder"), None));
    menu.push(Entry::Separator);
    if let Some((label, on)) = backend.login_item() {
        menu.push(item(Action::ToggleLogin, label, Some(on)));
        menu.push(Entry::Separator);
    }
    menu.push(item(Action::Restart, i18n::t("menu.restart"), None));
    menu.push(item(Action::Quit, i18n::t("menu.quit"), None));
    Model { image, tooltip: format!("Baskul — {}", snap.summary()), menu }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::{Conn, Peer, Snapshot};

    struct Fake(Role);

    impl Backend for Fake {
        fn role(&self) -> Role {
            self.0
        }
        fn icon_style(&self) -> IconStyle {
            IconStyle { style: Style::Color, size: 16, template: false }
        }
        fn login_item(&self) -> Option<(String, bool)> {
            Some(("login".into(), true))
        }
        fn perform(&self, _action: Action) {}
    }

    fn snap(peers: &[&str], active: Option<&str>) -> Snapshot {
        Snapshot {
            conn: Conn::Waiting(String::new()),
            remote_active: active.is_some(),
            active_peer: active.map(str::to_string),
            locked: true,
            warning: None,
            peers: peers.iter().map(|n| Peer { name: n.to_string(), os: "macos".into(), since: 0 }).collect(),
            edges_paused_by: None,
        }
    }

    fn actions(m: &Model) -> Vec<(Action, Option<bool>)> {
        m.menu
            .iter()
            .filter_map(|e| match e {
                Entry::Item { action, checked, .. } => Some((action.clone(), *checked)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn ids_round_trip() {
        for a in [
            Action::ReturnHere,
            Action::GoTo("Mac « mini »:2".into()),
            Action::ReturnToPrimary,
            Action::ToggleLock,
            Action::Settings,
            Action::OpenLog,
            Action::OpenFolder,
            Action::ToggleLogin,
            Action::AllowAccessibility,
            Action::InstallUpdate,
            Action::Restart,
            Action::Quit,
        ] {
            assert_eq!(Action::from_id(&a.id()), Some(a));
        }
        assert_eq!(Action::from_id("status"), None);
    }

    #[test]
    fn primary_menu_lists_peers_lock_and_profiles() {
        let m = build(
            &Fake(Role::Primary),
            &snap(&["Atelier", "Salon"], Some("Salon")),
            Some("0.9.0".into()),
            &(vec!["Jeu".into()], Some("Jeu".into())),
        );
        assert_eq!(m.image.rgba.len(), 16 * 16 * 4);
        assert!(matches!(&m.menu[0], Entry::Label(_)));
        assert_eq!(
            actions(&m),
            [
                (Action::InstallUpdate, None),
                (Action::ReturnHere, None),
                (Action::GoTo("Atelier".into()), Some(false)),
                (Action::GoTo("Salon".into()), Some(true)),
                (Action::ToggleLock, Some(true)),
                (Action::ApplyProfile("Jeu".into()), Some(true)),
                (Action::Settings, None),
                (Action::OpenLog, None),
                (Action::OpenFolder, None),
                (Action::ToggleLogin, Some(true)),
                (Action::Restart, None),
                (Action::Quit, None),
            ]
        );
    }

    #[test]
    fn secondary_menu_has_no_switching() {
        let m = build(&Fake(Role::Secondary), &snap(&["Principale"], None), None, &(Vec::new(), None));
        let a = actions(&m);
        assert!(!a.iter().any(|(a, _)| matches!(
            a,
            Action::ReturnHere | Action::GoTo(_) | Action::ToggleLock | Action::ReturnToPrimary
        )));
        assert_eq!(a.first(), Some(&(Action::Settings, None)));
    }

    #[test]
    fn piloted_secondary_menu_offers_the_return() {
        let mut s = snap(&["Principale"], None);
        s.remote_active = true;
        let m = build(&Fake(Role::Secondary), &s, None, &(Vec::new(), None));
        let a = actions(&m);
        assert_eq!(a.first(), Some(&(Action::ReturnToPrimary, None)));
        assert!(!a.iter().any(|(a, _)| matches!(a, Action::ReturnHere | Action::GoTo(_) | Action::ToggleLock)));
    }
}
