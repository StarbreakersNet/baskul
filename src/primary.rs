//! Logique de la machine principale (celle qui a le clavier et la souris),
//! indépendante du système.
//!
//! Le module de capture de la plateforme appelle ces fonctions depuis ses
//! hooks d'entrée ; chaque fonction indique si l'événement doit être laissé
//! passer au système local (`Pass`) ou supprimé (`Swallow`, quand il est
//! envoyé à une pilotée).
//!
//! Les machines sont placées sur une grille : la principale en (0, 0), les
//! pilotées autour. Le curseur passe d'une machine à sa voisine connectée
//! par leur bord commun, ou directement par raccourci ou commande.

use crate::capture;
use crate::config::{MachineCfg, MediaKeys, Placement};
use crate::geometry::{Layout, Point};
use crate::hotkey::Hotkey;
use crate::keymap;
use crate::net::PeerId;
use crate::protocol::{Edge, Message, MouseButton, SwitchTo};
use crate::status;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Laisser l'événement au système local.
    Pass,
    /// Supprimer l'événement localement.
    Swallow,
}

/// Où va une touche ou un bouton enfoncé (mémorisé pour que le relâchement
/// aille au même endroit que l'appui, même si le curseur a changé de machine).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    Local,
    /// Pilotée, par son nom.
    Remote(String),
    /// Touche utilisée par un raccourci de Baskul : supprimée partout.
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Local,
    /// Le clavier et la souris agissent sur cette pilotée (par son nom : elle
    /// garde sa place si elle se reconnecte).
    Remote(String),
}

/// Événements venant des pilotées (et commandes).
#[derive(Debug, Clone, PartialEq)]
pub enum PeerEvent {
    Connected {
        id: PeerId,
        name: String,
    },
    Disconnected {
        id: PeerId,
    },
    Leave {
        id: PeerId,
        edge: Edge,
        ratio: f32,
    },
    /// Fin d'un délai de grâce (voir `RECONNECT_GRACE`) : on vérifie si la
    /// pilotée en cours est revenue.
    GraceOver,
    /// Commande `baskul switch`, lancée ici (`from` : `None`) ou sur une
    /// pilotée.
    Switch {
        from: Option<PeerId>,
        to: SwitchTo,
    },
    /// L'utilisateur est actif sur une pilotée (son propre clavier ou
    /// trackpad).
    UserActive {
        from: PeerId,
    },
}

/// Coupure pendant que l'on pilote une machine : on attend ce délai avant de
/// ramener le curseur ici. Une reconnexion entre-temps (redémarrage de Baskul
/// sur la pilotée, après un changement de réglages…) reprend le pilotage là
/// où il en était.
pub const RECONNECT_GRACE: Duration = Duration::from_secs(4);

/// Intervalle minimal entre deux signaux d'activité (message aux pilotées, ou
/// maintien en activité de cette machine).
pub const ACTIVITY_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct PrimaryOptions {
    /// Bascule en poussant la souris contre le bord de l'écran.
    pub edge_switching: bool,
    /// Côté de la principale où se place une nouvelle pilotée.
    pub new_client_side: Edge,
    pub corner: i32,
    /// Pilotées placées sur la grille.
    pub layout: Vec<Placement>,
    /// Aller sur la dernière pilotée utilisée.
    pub switch_hotkey: Option<Hotkey>,
    pub return_hotkey: Option<Hotkey>,
    /// Machine suivante (la principale, puis les pilotées).
    pub next_hotkey: Option<Hotkey>,
    pub lock_hotkey: Option<Hotkey>,
    /// Machine où agissent les touches multimédia.
    pub media_keys: MediaKeys,
    /// Activité partagée : cette machine reste active tant que l'utilisateur
    /// l'est sur une pilotée.
    pub mirror_activity: bool,
}

/// Services fournis par la plateforme.
pub trait Host {
    /// Écrans locaux (mis en cache par la plateforme).
    fn layout(&self) -> &Layout;
    /// Position actuelle du curseur.
    fn cursor(&self) -> Point;
    /// Le curseur part sur une pilotée. `at` : position actuelle du curseur.
    /// La plateforme cache le curseur et peut le recentrer, puis appeler
    /// `PrimaryCore::on_warped`.
    fn hide_cursor(&mut self, at: Point);
    /// Le curseur revient : le placer en `pos` et le réafficher.
    fn show_cursor(&mut self, pos: Point);
    /// Envoie un message à une pilotée.
    fn send(&mut self, to: PeerId, msg: Message);
    /// L'utilisateur est actif sans que ce système le voie (clavier et souris
    /// envoyés à une pilotée, ou utilisés directement sur celle-ci) :
    /// repousse la veille de l'écran, l'économiseur et le statut « absent ».
    fn keep_active(&mut self) {}
    /// La grille a changé (pilotée placée d'elle-même) : l'enregistrer.
    fn save_layout(&mut self, _machines: Vec<MachineCfg>) {}
    /// Le clavier et la souris reviennent sur cette machine, le curseur en
    /// `pos` (bulle d'arrivée).
    fn arrived(&mut self, _pos: Point) {}
}

/// Une machine de la grille.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Machine {
    Primary,
    Client(String),
}

pub struct PrimaryCore {
    opts: PrimaryOptions,
    /// Pilotées connectées : identifiant de connexion et nom.
    clients: Vec<(PeerId, String)>,
    mode: Mode,
    locked: bool,
    /// Dernière pilotée utilisée (raccourci « aller sur la pilotée »).
    last_client: Option<String>,
    /// Position réelle du curseur pendant que l'on pilote une machine.
    anchor: Point,
    /// Le curseur a été recentré : on peut filtrer les déplacements aberrants.
    warp_done: bool,
    /// Position où le curseur a quitté l'écran local.
    exit_point: Point,
    key_targets: HashMap<u32, Target>,
    button_targets: [Option<Target>; 5],
    /// Touches physiquement enfoncées (pour les raccourcis).
    pressed: HashSet<u32>,
    /// Fin du délai de grâce de la pilotée en cours, déconnectée.
    grace_until: Option<Instant>,
    /// Derniers signaux d'activité envoyés aux pilotées, et donnés à cette
    /// machine.
    peer_active_at: Option<Instant>,
    local_active_at: Option<Instant>,
}

impl PrimaryCore {
    pub fn new(opts: PrimaryOptions) -> PrimaryCore {
        PrimaryCore {
            opts,
            clients: Vec::new(),
            mode: Mode::Local,
            locked: false,
            last_client: None,
            anchor: Point::default(),
            warp_done: false,
            exit_point: Point::default(),
            key_targets: HashMap::new(),
            button_targets: Default::default(),
            pressed: HashSet::new(),
            grace_until: None,
            peer_active_at: None,
            local_active_at: None,
        }
    }

    pub fn is_remote(&self) -> bool {
        self.mode != Mode::Local
    }

    /// Au moins une pilotée connectée.
    pub fn is_connected(&self) -> bool {
        !self.clients.is_empty()
    }

    /// Pilotée où agissent le clavier et la souris.
    pub fn active_client(&self) -> Option<&str> {
        match &self.mode {
            Mode::Remote(name) => Some(name),
            Mode::Local => None,
        }
    }

    fn id_of(&self, name: &str) -> Option<PeerId> {
        self.clients.iter().find(|(_, n)| n == name).map(|(id, _)| *id)
    }

    fn name_of(&self, id: PeerId) -> Option<String> {
        self.clients.iter().find(|(i, _)| *i == id).map(|(_, n)| n.clone())
    }

    fn send_to<H: Host>(&self, host: &mut H, name: &str, msg: Message) {
        if let Some(id) = self.id_of(name) {
            host.send(id, msg);
        }
    }

    fn input_target(&self) -> Target {
        match &self.mode {
            Mode::Remote(name) if self.id_of(name).is_some() => Target::Remote(name.clone()),
            // Coupure pendant le pilotage (délai de grâce) : rien ne part ni
            // n'arrive aux applications d'ici.
            Mode::Remote(_) => Target::Consumed,
            Mode::Local => Target::Local,
        }
    }

    fn local_buttons_down(&self) -> bool {
        self.button_targets.contains(&Some(Target::Local))
    }

    // ------------------------------------------------------------------
    // Grille
    // ------------------------------------------------------------------

    fn cell_of(&self, m: &Machine) -> Option<(i32, i32)> {
        match m {
            Machine::Primary => Some((0, 0)),
            Machine::Client(name) => self.opts.layout.iter().find(|p| &p.name == name).map(|p| p.cell),
        }
    }

    /// Machine connectée dans la case voisine de `from`, du côté `edge`.
    fn neighbor(&self, from: &Machine, edge: Edge) -> Option<Machine> {
        let (x, y) = self.cell_of(from)?;
        let cell = match edge {
            Edge::Left => (x - 1, y),
            Edge::Right => (x + 1, y),
            Edge::Top => (x, y - 1),
            Edge::Bottom => (x, y + 1),
        };
        if cell == (0, 0) {
            return Some(Machine::Primary);
        }
        let p = self.opts.layout.iter().find(|p| p.cell == cell)?;
        self.id_of(&p.name).map(|_| Machine::Client(p.name.clone()))
    }

    /// Bords de `m` par lesquels on rejoint une machine voisine connectée.
    fn exits(&self, m: &Machine) -> Vec<Edge> {
        if !self.opts.edge_switching {
            return Vec::new();
        }
        [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom]
            .into_iter()
            .filter(|e| self.neighbor(m, *e).is_some())
            .collect()
    }

    fn client_exits(&self, name: &str) -> Vec<Edge> {
        self.exits(&Machine::Client(name.to_string()))
    }

    /// Pilotées connectées, dans l'ordre de lecture de la grille (lignes de
    /// haut en bas, puis de gauche à droite).
    fn connected_in_order(&self) -> Vec<String> {
        let mut names: Vec<(Option<(i32, i32)>, String)> = self
            .clients
            .iter()
            .map(|(_, n)| (self.cell_of(&Machine::Client(n.clone())).map(|(x, y)| (y, x)), n.clone()))
            .collect();
        names.sort();
        names.into_iter().map(|(_, n)| n).collect()
    }

    /// Une pilotée inconnue se connecte : elle prend la première case libre
    /// à côté de la principale, et la grille est enregistrée.
    fn place<H: Host>(&mut self, host: &mut H, name: &str) {
        if self.opts.layout.iter().any(|p| p.name == name) {
            return;
        }
        let machines: Vec<MachineCfg> = self
            .opts
            .layout
            .iter()
            .map(|p| MachineCfg {
                name: p.name.clone(),
                x: p.cell.0,
                y: p.cell.1,
                hotkey: p.hotkey.as_ref().map(ToString::to_string).unwrap_or_default(),
            })
            .collect();
        let cell = crate::config::free_cell(&machines, self.opts.new_client_side);
        log::info!("« {name} » placée sur la grille en {cell:?}");
        self.opts.layout.push(Placement { name: name.to_string(), cell, hotkey: None });
        let mut machines = machines;
        machines.push(MachineCfg { name: name.to_string(), x: cell.0, y: cell.1, hotkey: String::new() });
        host.save_layout(machines);
    }

    /// La pilotée en cours apprend ses nouveaux bords de sortie.
    fn refresh_exits<H: Host>(&self, host: &mut H) {
        if let Mode::Remote(name) = &self.mode {
            self.send_to(host, name, Message::Exits { exits: self.client_exits(name) });
        }
    }

    // ------------------------------------------------------------------
    // Activité partagée
    // ------------------------------------------------------------------

    /// Entrée réelle de l'utilisateur : les machines qui ne la reçoivent pas
    /// sont tenues pour actives, elles aussi. Quand l'utilisateur s'arrête, les
    /// signaux cessent et chaque machine compte son inactivité normalement.
    fn user_input<H: Host>(&mut self, host: &mut H) {
        let active = self.active_client().and_then(|n| self.id_of(n));
        if self.is_remote() {
            // Clavier et souris agissent là-bas : ce système ne voit rien.
            self.keep_active(host);
        }
        self.notify_clients(host, active);
    }

    /// Signal d'activité aux pilotées (sauf `except`) ; chacune décide selon
    /// son propre réglage.
    fn notify_clients<H: Host>(&mut self, host: &mut H, except: Option<PeerId>) {
        if !due(self.peer_active_at) {
            return;
        }
        let targets: Vec<PeerId> = self.clients.iter().map(|(id, _)| *id).filter(|id| Some(*id) != except).collect();
        if targets.is_empty() {
            return;
        }
        self.peer_active_at = Some(Instant::now());
        for id in targets {
            host.send(id, Message::UserActive);
        }
    }

    fn keep_active<H: Host>(&mut self, host: &mut H) {
        if self.opts.mirror_activity && due(self.local_active_at) {
            self.local_active_at = Some(Instant::now());
            host.keep_active();
        }
    }

    // ------------------------------------------------------------------
    // Souris
    // ------------------------------------------------------------------

    /// Déplacement en mode local : `cur` = position réelle du curseur,
    /// `next` = position demandée (éventuellement hors écran).
    pub fn local_motion<H: Host>(&mut self, host: &mut H, cur: Point, next: Point) -> Decision {
        // Un déplacement nul n'est pas une activité (c'est celui de
        // `Host::keep_active`).
        if cur != next {
            self.user_input(host);
        }
        if self.is_remote() || !self.opts.edge_switching || self.locked || self.local_buttons_down() {
            return Decision::Pass;
        }
        for edge in self.exits(&Machine::Primary) {
            let layout = host.layout();
            if !layout.pushes_against(cur, next, edge, self.opts.corner) {
                continue;
            }
            let Some(Machine::Client(name)) = self.neighbor(&Machine::Primary, edge) else { continue };
            let ratio = layout.ratio_on_edge(layout.clamp(next), edge);
            log::debug!("bascule vers « {name} » (ratio {ratio:.3})");
            let exits = self.client_exits(&name);
            self.go_remote(host, &name, cur, Message::Enter { edge: edge.opposite(), ratio, exits });
            return Decision::Swallow;
        }
        Decision::Pass
    }

    /// Le curseur quitte cette machine depuis `cur` ; `msg` le fait entrer sur
    /// la pilotée `name`.
    fn go_remote<H: Host>(&mut self, host: &mut H, name: &str, cur: Point, msg: Message) {
        self.mode = Mode::Remote(name.to_string());
        self.last_client = Some(name.to_string());
        self.exit_point = cur;
        self.anchor = cur;
        self.warp_done = false;
        self.send_to(host, name, msg);
        host.hide_cursor(cur);
        status::set_active_peer(Some(name));
    }

    /// Passe d'une pilotée à une autre ; `msg` fait entrer le curseur sur la
    /// nouvelle.
    fn hand_over<H: Host>(&mut self, host: &mut H, name: &str, msg: Message) {
        self.mode = Mode::Remote(name.to_string());
        self.last_client = Some(name.to_string());
        self.grace_until = None;
        self.send_to(host, name, msg);
        status::set_active_peer(Some(name));
    }

    /// La plateforme a recentré le curseur (mode distant).
    pub fn on_warped(&mut self, anchor: Point) {
        if self.is_remote() {
            self.anchor = anchor;
            self.warp_done = true;
        }
    }

    /// Déplacement absolu en mode distant (Windows : le curseur réel reste
    /// immobile, la position demandée moins la position réelle donne le
    /// déplacement).
    pub fn remote_abs_motion<H: Host>(&mut self, host: &mut H, pt: Point) {
        let dx = pt.x - self.anchor.x;
        let dy = pt.y - self.anchor.y;
        if dx == 0 && dy == 0 {
            return;
        }
        if self.warp_done && self.is_bogus(host, dx, dy) {
            log::debug!("déplacement aberrant ignoré ({dx}, {dy})");
            return;
        }
        self.remote_motion(host, dx, dy);
    }

    /// Déplacement relatif en mode distant.
    pub fn remote_motion<H: Host>(&mut self, host: &mut H, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }
        self.user_input(host);
        if let Target::Remote(name) = self.input_target() {
            self.send_to(host, &name, Message::Motion { dx, dy });
        }
    }

    /// Juste après le recentrage, un événement émis avant celui-ci peut donner
    /// un déplacement d'environ la distance centre → bord : on l'ignore.
    fn is_bogus<H: Host>(&self, host: &H, dx: i32, dy: i32) -> bool {
        const ZONE: i32 = 10;
        let a = self.anchor;
        let d = host.layout().nearest_display(a);
        -dx + ZONE > a.x - d.left || dx + ZONE > d.right - a.x || -dy + ZONE > a.y - d.top || dy + ZONE > d.bottom - a.y
    }

    pub fn button<H: Host>(&mut self, host: &mut H, button: MouseButton, down: bool) -> Decision {
        self.user_input(host);
        let i = button.index();
        let target = if down {
            let t = self.button_targets[i].clone().unwrap_or_else(|| self.input_target());
            self.button_targets[i] = Some(t.clone());
            t
        } else {
            self.button_targets[i].take().unwrap_or_else(|| self.input_target())
        };
        match target {
            Target::Local => Decision::Pass,
            Target::Remote(name) => {
                self.send_to(host, &name, Message::Button { button, down });
                Decision::Swallow
            }
            Target::Consumed => Decision::Swallow,
        }
    }

    pub fn wheel<H: Host>(&mut self, host: &mut H, dx: i32, dy: i32) -> Decision {
        self.user_input(host);
        match self.input_target() {
            Target::Remote(name) => {
                self.send_to(host, &name, Message::Wheel { dx, dy });
                Decision::Swallow
            }
            Target::Consumed => Decision::Swallow,
            Target::Local => Decision::Pass,
        }
    }

    // ------------------------------------------------------------------
    // Clavier
    // ------------------------------------------------------------------

    pub fn key<H: Host>(&mut self, host: &mut H, code: u32, down: bool) -> Decision {
        self.user_input(host);
        let decision = if down {
            self.pressed.insert(code);
            let (target, repeat) = match self.key_targets.get(&code) {
                Some(t) => (t.clone(), true),
                None => {
                    let t = if !keymap::is_modifier(code) && capture::is_waiting() {
                        // Saisie d'un raccourci par l'app de réglages.
                        capture::finish(Hotkey::from_pressed(code, &self.pressed).to_string());
                        Target::Consumed
                    } else {
                        self.new_key_target(host, code)
                    };
                    self.key_targets.insert(code, t.clone());
                    (t, false)
                }
            };
            self.dispatch_key(host, target, code, true, repeat)
        } else {
            let target = self.key_targets.remove(&code).unwrap_or_else(|| self.key_input_target(code));
            self.dispatch_key(host, target, code, false, false)
        };
        if !down {
            self.pressed.remove(&code);
        }
        decision
    }

    fn key_input_target(&self, code: u32) -> Target {
        if keymap::is_media(code) {
            match self.opts.media_keys {
                MediaKeys::Primary => return Target::Local,
                MediaKeys::Secondary => {
                    // La pilotée en cours, sinon la dernière utilisée.
                    let name = self.active_client().map(str::to_string).or_else(|| self.default_client());
                    if let Some(name) = name.filter(|n| self.id_of(n).is_some()) {
                        return Target::Remote(name);
                    }
                }
                _ => {}
            }
        }
        self.input_target()
    }

    fn new_key_target<H: Host>(&mut self, host: &mut H, code: u32) -> Target {
        let matches = |hk: &Option<Hotkey>| hk.as_ref().is_some_and(|hk| hk.matches(code, &self.pressed));
        if self.is_remote() && matches(&self.opts.return_hotkey) {
            log::info!("raccourci : retour sur la machine principale");
            self.come_back(host);
            return Target::Consumed;
        }
        if !self.is_remote() && matches(&self.opts.switch_hotkey) && self.is_connected() {
            log::info!("raccourci : passage sur une pilotée");
            self.switch_to(host, None, SwitchTo::LastClient);
            return Target::Consumed;
        }
        if matches(&self.opts.next_hotkey) && self.is_connected() {
            log::info!("raccourci : machine suivante");
            self.switch_to(host, None, SwitchTo::Next);
            return Target::Consumed;
        }
        let direct = self
            .opts
            .layout
            .iter()
            .find(|p| matches(&p.hotkey) && self.id_of(&p.name).is_some())
            .map(|p| p.name.clone());
        if let Some(name) = direct {
            log::info!("raccourci : passage sur « {name} »");
            self.switch_to(host, None, SwitchTo::Client(name));
            return Target::Consumed;
        }
        if matches(&self.opts.lock_hotkey) {
            let locked = self.toggle_lock();
            log::info!("curseur {}", if locked { "verrouillé" } else { "déverrouillé" });
            return Target::Consumed;
        }
        self.key_input_target(code)
    }

    fn dispatch_key<H: Host>(&mut self, host: &mut H, target: Target, code: u32, down: bool, repeat: bool) -> Decision {
        match target {
            Target::Local => Decision::Pass,
            Target::Remote(name) => {
                self.send_to(host, &name, Message::Key { code, down, repeat });
                Decision::Swallow
            }
            Target::Consumed => Decision::Swallow,
        }
    }

    // ------------------------------------------------------------------
    // Événements des pilotées
    // ------------------------------------------------------------------

    pub fn peer_event<H: Host>(&mut self, host: &mut H, ev: PeerEvent) {
        match ev {
            PeerEvent::Connected { id, name } => {
                // Reconnexion : la nouvelle connexion remplace l'ancienne.
                self.clients.retain(|(_, n)| n != &name);
                self.clients.push((id, name.clone()));
                self.place(host, &name);
                if self.active_client() == Some(name.as_str()) {
                    // Reconnexion pendant le pilotage : la pilotée (peut-être
                    // redémarrée) reprend le curseur là où il est.
                    log::info!("reconnexion : reprise du pilotage de « {name} »");
                    self.grace_until = None;
                    host.send(id, Message::Jump { exits: self.client_exits(&name) });
                } else {
                    self.refresh_exits(host);
                }
            }
            PeerEvent::Disconnected { id } => {
                // Connexion déjà remplacée par une plus récente : rien à faire.
                let Some(name) = self.name_of(id) else { return };
                self.clients.retain(|(i, _)| *i != id);
                // Les relâchements des touches envoyées là-bas n'ont plus de
                // destination : on les supprime (le système local n'a jamais vu
                // l'appui correspondant).
                let lost = Target::Remote(name.clone());
                for t in self.key_targets.values_mut().chain(self.button_targets.iter_mut().flatten()) {
                    if *t == lost {
                        *t = Target::Consumed;
                    }
                }
                if self.active_client() == Some(name.as_str()) {
                    log::info!("connexion perdue avec « {name} » pendant le pilotage : attente d'une reconnexion");
                    self.grace_until = Some(Instant::now() + RECONNECT_GRACE);
                } else {
                    self.refresh_exits(host);
                }
            }
            PeerEvent::GraceOver => {
                let expired = self.grace_until.is_some_and(|t| Instant::now() >= t);
                if let (true, Some(name)) = (expired, self.active_client().map(str::to_string)) {
                    if self.id_of(&name).is_none() {
                        log::info!("« {name} » ne revient pas : retour du curseur sur la machine principale");
                        let p = self.exit_point;
                        self.force_local(host, p);
                    }
                }
            }
            PeerEvent::Switch { from, to } => self.switch_to(host, from, to),
            PeerEvent::UserActive { from } => {
                self.keep_active(host);
                self.notify_clients(host, Some(from));
            }
            PeerEvent::Leave { id, edge, ratio } => {
                let Some(name) = self.name_of(id).filter(|n| self.active_client() == Some(n.as_str())) else { return };
                let from = Machine::Client(name.clone());
                match self.neighbor(&from, edge).filter(|_| !self.locked) {
                    Some(Machine::Primary) => {
                        let pos = host.layout().point_on_edge(edge.opposite(), ratio, 1);
                        log::debug!("retour sur la machine principale en {pos:?}");
                        self.force_local(host, pos);
                    }
                    Some(Machine::Client(next)) => {
                        log::debug!("passage de « {name} » à « {next} »");
                        let exits = self.client_exits(&next);
                        self.hand_over(host, &next, Message::Enter { edge: edge.opposite(), ratio, exits });
                    }
                    // Verrouillé, ou voisine partie entre-temps : le curseur
                    // revient sur la pilotée.
                    None => {
                        let exits = self.client_exits(&name);
                        host.send(id, Message::Enter { edge, ratio, exits });
                    }
                }
            }
        }
    }

    /// Remise à zéro complète (verrouillage de session, mise en veille…) :
    /// tout est relâché de l'autre côté et le curseur revient ici.
    pub fn reset<H: Host>(&mut self, host: &mut H) {
        if let Some(name) = self.active_client().map(str::to_string) {
            self.send_to(host, &name, Message::ReleaseAll);
        }
        self.key_targets.clear();
        self.button_targets = Default::default();
        self.pressed.clear();
        if self.is_remote() {
            let p = self.exit_point;
            self.go_local(host, p);
        }
    }

    /// Pilotée visée par « aller sur la pilotée » : la dernière utilisée si
    /// elle est connectée, sinon la première de la grille.
    fn default_client(&self) -> Option<String> {
        self.last_client
            .clone()
            .filter(|n| self.id_of(n).is_some())
            .or_else(|| self.connected_in_order().into_iter().next())
    }

    /// Raccourci ou commande `baskul switch` : le clavier et la souris passent
    /// sur la machine demandée.
    pub fn switch_to<H: Host>(&mut self, host: &mut H, from: Option<PeerId>, to: SwitchTo) {
        let target = match to {
            SwitchTo::Primary => Machine::Primary,
            SwitchTo::Client(name) => Machine::Client(name),
            SwitchTo::LastClient => match self.default_client() {
                Some(n) => Machine::Client(n),
                None => return log::warn!("aucune pilotée connectée"),
            },
            SwitchTo::Sender => match from.and_then(|id| self.name_of(id)) {
                Some(n) => Machine::Client(n),
                None => return,
            },
            SwitchTo::Next => {
                let mut order = vec![Machine::Primary];
                order.extend(self.connected_in_order().into_iter().map(Machine::Client));
                let current = match self.active_client() {
                    Some(n) => Machine::Client(n.to_string()),
                    None => Machine::Primary,
                };
                let i = order.iter().position(|m| *m == current).map_or(0, |i| i + 1);
                order[i % order.len()].clone()
            }
        };
        match target {
            Machine::Primary => {
                if self.is_remote() {
                    log::info!("retour sur la machine principale");
                    self.come_back(host);
                }
            }
            Machine::Client(name) => {
                if self.id_of(&name).is_none() {
                    log::warn!("« {name} » n'est pas connectée");
                    return;
                }
                if self.active_client() == Some(name.as_str()) {
                    return;
                }
                log::info!("passage sur « {name} »");
                let jump = Message::Jump { exits: self.client_exits(&name) };
                match self.active_client().map(str::to_string) {
                    Some(previous) => {
                        // L'ancienne pilotée relâche tout et sait qu'elle n'est
                        // plus pilotée.
                        self.send_to(host, &previous, Message::ReleaseAll);
                        self.hand_over(host, &name, jump);
                    }
                    None => {
                        let cur = host.cursor();
                        self.go_remote(host, &name, cur, jump);
                    }
                }
            }
        }
    }

    /// Ramène le curseur sur la machine principale (menu de l'icône).
    pub fn return_home<H: Host>(&mut self, host: &mut H) {
        if self.is_remote() {
            self.come_back(host);
        }
    }

    /// Retour décidé ici (raccourci, menu, commande) : la pilotée relâche tout
    /// et sait qu'elle n'est plus pilotée (son icône change).
    fn come_back<H: Host>(&mut self, host: &mut H) {
        if let Some(name) = self.active_client().map(str::to_string) {
            self.send_to(host, &name, Message::ReleaseAll);
        }
        let p = self.exit_point;
        self.force_local(host, p);
    }

    pub fn toggle_lock(&mut self) -> bool {
        self.locked = !self.locked;
        status::set_locked(self.locked);
        self.locked
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Retour sur cette machine (signalé par la bulle d'arrivée).
    fn force_local<H: Host>(&mut self, host: &mut H, pos: Point) {
        self.go_local(host, pos);
        host.arrived(pos);
    }

    fn go_local<H: Host>(&mut self, host: &mut H, pos: Point) {
        self.mode = Mode::Local;
        self.warp_done = false;
        self.grace_until = None;
        host.show_cursor(pos);
        status::set_active_peer(None);
    }
}

/// Le dernier signal d'activité date d'au moins `ACTIVITY_INTERVAL`.
fn due(last: Option<Instant>) -> bool {
    last.is_none_or(|t| t.elapsed() >= ACTIVITY_INTERVAL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::keymap::*;

    const ATELIER: PeerId = 1;
    const SALON: PeerId = 2;

    struct FakeHost {
        layout: Layout,
        cursor: Point,
        sent: Vec<(PeerId, Message)>,
        hidden_at: Option<Point>,
        shown_at: Option<Point>,
        /// Signaux d'activité envoyés aux pilotées, et donnés à cette machine.
        user_active_sent: Vec<PeerId>,
        kept_active: usize,
        saved: Option<Vec<MachineCfg>>,
        /// Retours signalés (bulle d'arrivée).
        arrivals: usize,
    }

    impl FakeHost {
        /// Messages envoyés à une pilotée.
        fn to(&self, id: PeerId) -> Vec<Message> {
            self.sent.iter().filter(|(i, _)| *i == id).map(|(_, m)| m.clone()).collect()
        }
        fn last(&self) -> Option<&Message> {
            self.sent.last().map(|(_, m)| m)
        }
    }

    impl Host for FakeHost {
        fn layout(&self) -> &Layout {
            &self.layout
        }
        fn cursor(&self) -> Point {
            self.cursor
        }
        fn hide_cursor(&mut self, at: Point) {
            self.hidden_at = Some(at);
        }
        fn show_cursor(&mut self, pos: Point) {
            self.shown_at = Some(pos);
        }
        fn send(&mut self, to: PeerId, msg: Message) {
            match msg {
                Message::UserActive => self.user_active_sent.push(to),
                m => self.sent.push((to, m)),
            }
        }
        fn keep_active(&mut self) {
            self.kept_active += 1;
        }
        fn save_layout(&mut self, machines: Vec<MachineCfg>) {
            self.saved = Some(machines);
        }
        fn arrived(&mut self, _pos: Point) {
            self.arrivals += 1;
        }
    }

    fn placement(name: &str, x: i32, y: i32) -> Placement {
        Placement { name: name.into(), cell: (x, y), hotkey: None }
    }

    /// Une pilotée, « Atelier », à droite de la principale.
    fn setup() -> (PrimaryCore, FakeHost) {
        let mut core = PrimaryCore::new(PrimaryOptions {
            edge_switching: true,
            new_client_side: Edge::Right,
            corner: 0,
            layout: vec![placement("Atelier", 1, 0)],
            switch_hotkey: Hotkey::parse("Shift+F12").unwrap(),
            return_hotkey: Hotkey::parse("Ctrl+Alt+Shift+B").unwrap(),
            next_hotkey: None,
            lock_hotkey: Hotkey::parse("ScrollLock").unwrap(),
            media_keys: MediaKeys::Cursor,
            mirror_activity: true,
        });
        let mut host = FakeHost {
            layout: Layout::new(vec![Rect::new(0, 0, 1920, 1080)]),
            cursor: Point::new(700, 300),
            sent: vec![],
            hidden_at: None,
            shown_at: None,
            user_active_sent: vec![],
            kept_active: 0,
            saved: None,
            arrivals: 0,
        };
        core.peer_event(&mut host, PeerEvent::Connected { id: ATELIER, name: "Atelier".into() });
        host.sent.clear();
        (core, host)
    }

    /// Deux pilotées côte à côte : « Atelier » puis « Salon », à droite.
    fn setup_two() -> (PrimaryCore, FakeHost) {
        let (mut core, mut host) = setup();
        core.opts.layout.push(placement("Salon", 2, 0));
        core.peer_event(&mut host, PeerEvent::Connected { id: SALON, name: "Salon".into() });
        host.sent.clear();
        (core, host)
    }

    fn go_remote(core: &mut PrimaryCore, host: &mut FakeHost) {
        let d = core.local_motion(host, Point::new(1919, 540), Point::new(1925, 540));
        assert_eq!(d, Decision::Swallow);
        assert!(core.is_remote());
        core.on_warped(Point::new(960, 540));
    }

    fn expire_grace(core: &mut PrimaryCore) {
        core.grace_until = Some(Instant::now() - Duration::from_millis(1));
    }

    fn press(core: &mut PrimaryCore, host: &mut FakeHost, keys: &[u32]) -> Vec<Decision> {
        let mut d: Vec<Decision> = keys.iter().map(|k| core.key(host, *k, true)).collect();
        d.extend(keys.iter().rev().map(|k| core.key(host, *k, false)));
        d
    }

    /// Touche A (code HID).
    const KEY_A: u32 = 0x070004;

    #[test]
    fn switch_and_return() {
        let (mut core, mut host) = setup();
        assert_eq!(core.local_motion(&mut host, Point::new(100, 100), Point::new(105, 100)), Decision::Pass);
        go_remote(&mut core, &mut host);
        assert!(
            matches!(&host.to(ATELIER)[0], Message::Enter { edge: Edge::Left, exits, .. } if *exits == [Edge::Left])
        );
        assert_eq!(host.hidden_at, Some(Point::new(1919, 540)));
        assert_eq!(core.active_client(), Some("Atelier"));

        core.remote_abs_motion(&mut host, Point::new(965, 538));
        assert_eq!(host.last(), Some(&Message::Motion { dx: 5, dy: -2 }));
        // Artefact du recentrage : ignoré.
        let n = host.sent.len();
        core.remote_abs_motion(&mut host, Point::new(1925, 540));
        assert_eq!(host.sent.len(), n);

        core.peer_event(&mut host, PeerEvent::Leave { id: ATELIER, edge: Edge::Left, ratio: 0.5 });
        assert!(!core.is_remote());
        let p = host.shown_at.unwrap();
        assert_eq!(p.x, 1918);
        assert!((p.y - 540).abs() <= 1);
    }

    #[test]
    fn only_edges_with_a_connected_neighbor_switch() {
        let (mut core, mut host) = setup();
        // Rien à gauche de la principale.
        assert_eq!(core.local_motion(&mut host, Point::new(0, 540), Point::new(-5, 540)), Decision::Pass);
        // Pilotée déconnectée : le bord droit ne mène plus nulle part.
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        assert!(!core.is_connected());
    }

    #[test]
    fn no_switch_while_dragging() {
        let (mut core, mut host) = setup();
        assert_eq!(core.button(&mut host, MouseButton::Left, true), Decision::Pass);
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        assert_eq!(core.button(&mut host, MouseButton::Left, false), Decision::Pass);
        go_remote(&mut core, &mut host);
    }

    #[test]
    fn key_release_follows_press() {
        let (mut core, mut host) = setup();
        let a = hid_from_name("A").unwrap();
        // Shift appuyé en local, puis passage sur la pilotée.
        assert_eq!(core.key(&mut host, HID_SHIFT_LEFT, true), Decision::Pass);
        go_remote(&mut core, &mut host);
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(host.last(), Some(&Message::Key { code: a, down: true, repeat: false }));
        // Répétition automatique.
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(host.last(), Some(&Message::Key { code: a, down: true, repeat: true }));
        // Le relâchement de Shift reste local.
        assert_eq!(core.key(&mut host, HID_SHIFT_LEFT, false), Decision::Pass);
        // Retour local alors que A est encore enfoncé : son relâchement va là-bas.
        core.peer_event(&mut host, PeerEvent::Leave { id: ATELIER, edge: Edge::Left, ratio: 0.5 });
        assert_eq!(core.key(&mut host, a, false), Decision::Swallow);
        assert_eq!(host.last(), Some(&Message::Key { code: a, down: false, repeat: false }));
    }

    #[test]
    fn reset_brings_cursor_back_without_bubble() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        core.reset(&mut host);
        assert!(!core.is_remote());
        assert!(host.shown_at.is_some());
        assert_eq!(host.arrivals, 0);
    }

    #[test]
    fn return_hotkey() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        let b = hid_from_name("B").unwrap();
        for k in [HID_CONTROL_LEFT, HID_ALT_LEFT, HID_SHIFT_LEFT] {
            assert_eq!(core.key(&mut host, k, true), Decision::Swallow);
        }
        let n = host.sent.len();
        assert_eq!(core.key(&mut host, b, true), Decision::Swallow);
        assert!(!core.is_remote());
        // La touche du raccourci n'est pas transmise ; la pilotée est prévenue
        // qu'elle n'est plus pilotée.
        assert_eq!(host.sent[n..], [(ATELIER, Message::ReleaseAll)]);
        assert_eq!(host.shown_at, Some(Point::new(1919, 540)));
        assert_eq!(host.arrivals, 1);
        // Relâchements : B est consommée, les modificateurs vont là-bas.
        assert_eq!(core.key(&mut host, b, false), Decision::Swallow);
        assert_eq!(host.sent.len(), n + 1);
        assert_eq!(core.key(&mut host, HID_CONTROL_LEFT, false), Decision::Swallow);
        assert_eq!(host.last(), Some(&Message::Key { code: HID_CONTROL_LEFT, down: false, repeat: false }));
    }

    #[test]
    fn disconnect_returns_cursor_and_swallows_pending_releases() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        let a = hid_from_name("A").unwrap();
        core.key(&mut host, a, true);
        core.button(&mut host, MouseButton::Right, true);
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        // Délai pas encore écoulé : rien ne change.
        core.peer_event(&mut host, PeerEvent::GraceOver);
        assert!(core.is_remote());
        expire_grace(&mut core);
        core.peer_event(&mut host, PeerEvent::GraceOver);
        assert!(!core.is_remote());
        assert_eq!(core.key(&mut host, a, false), Decision::Swallow);
        assert_eq!(core.button(&mut host, MouseButton::Right, false), Decision::Swallow);
        // Ensuite tout redevient local.
        assert_eq!(core.key(&mut host, a, true), Decision::Pass);
    }

    #[test]
    fn short_disconnect_resumes_remote_control() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        host.sent.clear();
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        // Pendant la coupure : le curseur reste là-bas, rien n'arrive ici.
        assert!(core.is_remote());
        let a = hid_from_name("A").unwrap();
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(core.key(&mut host, a, false), Decision::Swallow);
        assert_eq!(core.button(&mut host, MouseButton::Left, true), Decision::Swallow);
        assert_eq!(core.button(&mut host, MouseButton::Left, false), Decision::Swallow);
        assert_eq!(core.wheel(&mut host, 0, 120), Decision::Swallow);
        assert!(host.sent.is_empty());
        // Reconnexion (nouvelle connexion) : la pilotée reprend le curseur.
        core.peer_event(&mut host, PeerEvent::Connected { id: 7, name: "Atelier".into() });
        assert!(core.is_remote());
        assert_eq!(host.sent, [(7, Message::Jump { exits: vec![Edge::Left] })]);
        // Le délai de la coupure passée n'a plus d'effet.
        expire_grace(&mut core);
        core.grace_until = None;
        core.peer_event(&mut host, PeerEvent::GraceOver);
        assert!(core.is_remote());
        assert_eq!(core.key(&mut host, a, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&(7, Message::Key { code: a, down: true, repeat: false })));
    }

    #[test]
    fn replaced_connection_closing_late_is_ignored() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        // La pilotée se reconnecte avant que l'ancienne connexion soit fermée.
        core.peer_event(&mut host, PeerEvent::Connected { id: 7, name: "Atelier".into() });
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        assert!(core.is_connected() && core.is_remote());
        assert_eq!(core.key(&mut host, KEY_A, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&(7, Message::Key { code: KEY_A, down: true, repeat: false })));
    }

    #[test]
    fn return_hotkey_during_disconnect() {
        let (mut core, mut host) = setup();
        go_remote(&mut core, &mut host);
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        let b = hid_from_name("B").unwrap();
        press(&mut core, &mut host, &[HID_CONTROL_LEFT, HID_ALT_LEFT, HID_SHIFT_LEFT, b]);
        assert!(!core.is_remote());
        assert_eq!(host.shown_at, Some(Point::new(1919, 540)));
    }

    #[test]
    fn lock_prevents_switching() {
        let (mut core, mut host) = setup();
        assert_eq!(core.key(&mut host, HID_SCROLL_LOCK, true), Decision::Swallow);
        assert_eq!(core.key(&mut host, HID_SCROLL_LOCK, false), Decision::Swallow);
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        core.key(&mut host, HID_SCROLL_LOCK, true);
        core.key(&mut host, HID_SCROLL_LOCK, false);
        go_remote(&mut core, &mut host);
        // Verrouillé pendant le pilotage : un « Leave » renvoie le curseur là-bas.
        core.key(&mut host, HID_SCROLL_LOCK, true);
        core.peer_event(&mut host, PeerEvent::Leave { id: ATELIER, edge: Edge::Left, ratio: 0.3 });
        assert!(core.is_remote());
        assert_eq!(host.last(), Some(&Message::Enter { edge: Edge::Left, ratio: 0.3, exits: vec![Edge::Left] }));
    }

    #[test]
    fn switch_hotkey_jumps_and_returns_in_place() {
        let (mut core, mut host) = setup();
        let f12 = hid_from_name("F12").unwrap();
        let d = press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        // Maj reste locale (appui et relâchement), F12 est consommée.
        assert_eq!(d, [Decision::Pass, Decision::Swallow, Decision::Swallow, Decision::Pass]);
        assert!(core.is_remote());
        assert_eq!(host.to(ATELIER), [Message::Jump { exits: vec![Edge::Left] }]);
        assert_eq!(host.hidden_at, Some(Point::new(700, 300)));
        core.on_warped(Point::new(960, 540));

        // Retour : le curseur réapparaît là où il était.
        let b = hid_from_name("B").unwrap();
        press(&mut core, &mut host, &[HID_CONTROL_LEFT, HID_ALT_LEFT, HID_SHIFT_LEFT, b]);
        assert!(!core.is_remote());
        assert_eq!(host.shown_at, Some(Point::new(700, 300)));
    }

    #[test]
    fn hotkey_capture_for_settings() {
        let (mut core, mut host) = setup();
        capture::set_supported(true);
        assert!(capture::start());
        // Shift+F12 est relevée au lieu de faire basculer, et n'arrive pas
        // aux applications.
        let f12 = hid_from_name("F12").unwrap();
        let d = press(&mut core, &mut host, &[HID_SHIFT_RIGHT, f12]);
        assert_eq!(d, [Decision::Pass, Decision::Swallow, Decision::Swallow, Decision::Pass]);
        assert!(!core.is_remote());
        assert_eq!(capture::poll(), capture::State::Done("Shift+F12".into()));
        // Ensuite, le raccourci fonctionne de nouveau.
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
    }

    #[test]
    fn same_hotkey_toggles() {
        let (mut core, mut host) = setup();
        core.opts.return_hotkey = core.opts.switch_hotkey.clone();
        let f12 = hid_from_name("F12").unwrap();
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(!core.is_remote());
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
    }

    #[test]
    fn switch_hotkey_passes_through_when_disconnected() {
        let (mut core, mut host) = setup();
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        let f12 = hid_from_name("F12").unwrap();
        let d = press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert_eq!(d, [Decision::Pass; 4]);
        assert!(!core.is_remote());
        assert!(host.sent.is_empty());
    }

    #[test]
    fn edge_switching_disabled() {
        let (mut core, mut host) = setup();
        core.opts.edge_switching = false;
        assert_eq!(core.local_motion(&mut host, Point::new(1919, 540), Point::new(1925, 540)), Decision::Pass);
        assert!(!core.is_remote());
        // Le raccourci fonctionne, sans bord de sortie pour la pilotée.
        let f12 = hid_from_name("F12").unwrap();
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, f12]);
        assert!(core.is_remote());
        assert_eq!(host.to(ATELIER), [Message::Jump { exits: vec![] }]);
    }

    #[test]
    fn media_keys_on_secondary_even_when_local() {
        let (mut core, mut host) = setup();
        core.opts.media_keys = MediaKeys::Secondary;
        assert!(!core.is_remote());
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, true), Decision::Swallow);
        assert_eq!(host.sent.last(), Some(&(ATELIER, Message::Key { code: HID_VOLUME_UP, down: true, repeat: false })));
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, false), Decision::Swallow);
        // Les autres touches restent locales.
        assert_eq!(core.key(&mut host, hid_from_name("A").unwrap(), true), Decision::Pass);
        // Sans connexion, elles agissent ici.
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        assert_eq!(core.key(&mut host, HID_VOLUME_DOWN, true), Decision::Pass);
    }

    #[test]
    fn media_keys_local_option() {
        let (mut core, mut host) = setup();
        core.opts.media_keys = MediaKeys::Primary;
        go_remote(&mut core, &mut host);
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, true), Decision::Pass);
        assert_eq!(core.key(&mut host, HID_VOLUME_UP, false), Decision::Pass);
        assert_eq!(core.wheel(&mut host, 0, 120), Decision::Swallow);
        assert_eq!(host.last(), Some(&Message::Wheel { dx: 0, dy: 120 }));
    }

    #[test]
    fn switch_command_both_ways() {
        let (mut core, mut host) = setup();
        core.peer_event(&mut host, PeerEvent::Switch { from: None, to: SwitchTo::LastClient });
        assert!(core.is_remote());
        assert_eq!(host.to(ATELIER), vec![Message::Jump { exits: vec![Edge::Left] }]);
        assert_eq!(host.hidden_at, Some(Point::new(700, 300)));
        // Déjà là-bas : rien de plus.
        core.peer_event(&mut host, PeerEvent::Switch { from: None, to: SwitchTo::Client("Atelier".into()) });
        assert_eq!(host.sent.len(), 1);
        // Retour : la pilotée relâche tout, le curseur revient où il était.
        core.peer_event(&mut host, PeerEvent::Switch { from: None, to: SwitchTo::Primary });
        assert!(!core.is_remote());
        assert_eq!(host.to(ATELIER)[1], Message::ReleaseAll);
        assert_eq!(host.shown_at, Some(Point::new(700, 300)));
    }

    #[test]
    fn switch_command_needs_a_connected_machine() {
        let (mut core, mut host) = setup();
        core.peer_event(&mut host, PeerEvent::Switch { from: None, to: SwitchTo::Client("Inconnue".into()) });
        core.peer_event(&mut host, PeerEvent::Disconnected { id: ATELIER });
        core.peer_event(&mut host, PeerEvent::Switch { from: None, to: SwitchTo::LastClient });
        assert!(!core.is_remote());
        assert!(host.sent.is_empty());
    }

    #[test]
    fn new_machines_are_placed_and_saved() {
        let (mut core, mut host) = setup();
        core.peer_event(&mut host, PeerEvent::Connected { id: SALON, name: "Salon".into() });
        // À droite c'est pris : de l'autre côté.
        assert_eq!(core.cell_of(&Machine::Client("Salon".into())), Some((-1, 0)));
        let saved = host.saved.clone().unwrap();
        assert_eq!(
            saved.iter().map(|m| (m.name.as_str(), m.x, m.y)).collect::<Vec<_>>(),
            [("Atelier", 1, 0), ("Salon", -1, 0)]
        );
        // On y va par le bord gauche.
        assert_eq!(core.local_motion(&mut host, Point::new(0, 540), Point::new(-5, 540)), Decision::Swallow);
        assert_eq!(core.active_client(), Some("Salon"));
        // Déjà placée : rien n'est réenregistré.
        host.saved = None;
        core.peer_event(&mut host, PeerEvent::Connected { id: 9, name: "Salon".into() });
        assert!(host.saved.is_none());
    }

    #[test]
    fn edges_lead_from_one_piloted_machine_to_the_next() {
        let (mut core, mut host) = setup_two();
        go_remote(&mut core, &mut host);
        // Atelier a une voisine de chaque côté.
        assert!(matches!(&host.to(ATELIER)[0], Message::Enter { exits, .. } if *exits == [Edge::Left, Edge::Right]));
        core.peer_event(&mut host, PeerEvent::Leave { id: ATELIER, edge: Edge::Right, ratio: 0.4 });
        assert_eq!(core.active_client(), Some("Salon"));
        assert_eq!(host.to(SALON), [Message::Enter { edge: Edge::Left, ratio: 0.4, exits: vec![Edge::Left] }]);
        // Le clavier suit.
        core.key(&mut host, KEY_A, true);
        assert_eq!(host.sent.last(), Some(&(SALON, Message::Key { code: KEY_A, down: true, repeat: false })));
        // Retour vers Atelier, puis vers la principale.
        core.peer_event(&mut host, PeerEvent::Leave { id: SALON, edge: Edge::Left, ratio: 0.6 });
        assert_eq!(core.active_client(), Some("Atelier"));
        assert!(matches!(host.to(ATELIER).last(), Some(Message::Enter { edge: Edge::Right, .. })));
        core.peer_event(&mut host, PeerEvent::Leave { id: ATELIER, edge: Edge::Left, ratio: 0.5 });
        assert!(!core.is_remote());
        // Un « Leave » d'une pilotée qui n'est pas en cours est ignoré.
        core.peer_event(&mut host, PeerEvent::Leave { id: SALON, edge: Edge::Left, ratio: 0.5 });
        assert!(!core.is_remote());
    }

    #[test]
    fn exits_follow_connections() {
        let (mut core, mut host) = setup_two();
        go_remote(&mut core, &mut host);
        host.sent.clear();
        core.peer_event(&mut host, PeerEvent::Disconnected { id: SALON });
        assert_eq!(host.to(ATELIER), [Message::Exits { exits: vec![Edge::Left] }]);
        core.peer_event(&mut host, PeerEvent::Connected { id: 8, name: "Salon".into() });
        assert_eq!(host.to(ATELIER)[1], Message::Exits { exits: vec![Edge::Left, Edge::Right] });
    }

    #[test]
    fn next_machine_cycles_through_connected_ones() {
        let (mut core, mut host) = setup_two();
        let next = |core: &mut PrimaryCore, host: &mut FakeHost| {
            core.peer_event(host, PeerEvent::Switch { from: None, to: SwitchTo::Next });
            core.active_client().map(str::to_string)
        };
        assert_eq!(next(&mut core, &mut host).as_deref(), Some("Atelier"));
        assert_eq!(next(&mut core, &mut host).as_deref(), Some("Salon"));
        // En passant d'une pilotée à l'autre, la précédente relâche tout.
        assert!(host.to(ATELIER).contains(&Message::ReleaseAll));
        assert_eq!(host.to(SALON), [Message::Jump { exits: vec![Edge::Left] }]);
        assert_eq!(next(&mut core, &mut host), None);
        assert_eq!(host.shown_at, Some(Point::new(700, 300)));
    }

    #[test]
    fn hotkeys_for_one_machine_and_for_the_next() {
        let (mut core, mut host) = setup_two();
        core.opts.layout[1].hotkey = Hotkey::parse("Ctrl+Alt+2").unwrap();
        core.opts.next_hotkey = Hotkey::parse("Ctrl+Alt+N").unwrap();
        let two = hid_from_name("2").unwrap();
        press(&mut core, &mut host, &[HID_CONTROL_LEFT, HID_ALT_LEFT, two]);
        assert_eq!(core.active_client(), Some("Salon"));
        let n = hid_from_name("N").unwrap();
        press(&mut core, &mut host, &[HID_CONTROL_LEFT, HID_ALT_LEFT, n]);
        assert_eq!(core.active_client(), None);
        // « Aller sur la pilotée » : la dernière utilisée.
        press(&mut core, &mut host, &[HID_SHIFT_LEFT, hid_from_name("F12").unwrap()]);
        assert_eq!(core.active_client(), Some("Salon"));
    }

    #[test]
    fn command_from_a_piloted_machine() {
        let (mut core, mut host) = setup_two();
        core.peer_event(&mut host, PeerEvent::Switch { from: Some(SALON), to: SwitchTo::Sender });
        assert_eq!(core.active_client(), Some("Salon"));
        core.peer_event(&mut host, PeerEvent::Switch { from: Some(SALON), to: SwitchTo::Primary });
        assert!(!core.is_remote());
    }

    #[test]
    fn activity_is_mirrored_on_machines_without_input() {
        let (mut core, mut host) = setup_two();
        // Ici : les pilotées sont prévenues, une fois par intervalle.
        core.key(&mut host, KEY_A, true);
        core.key(&mut host, KEY_A, false);
        core.local_motion(&mut host, Point::new(100, 100), Point::new(101, 100));
        assert_eq!((host.user_active_sent.clone(), host.kept_active), (vec![ATELIER, SALON], 0));
        // Sur Atelier : cette machine et Salon ne voient plus rien passer.
        go_remote(&mut core, &mut host);
        core.peer_active_at = Some(Instant::now() - ACTIVITY_INTERVAL);
        core.remote_motion(&mut host, 3, 0);
        core.button(&mut host, MouseButton::Left, true);
        assert_eq!((&host.user_active_sent[2..], host.kept_active), (&[SALON][..], 1));
        // Le déplacement nul de `keep_active` ne compte pas.
        core.local_active_at = Some(Instant::now() - ACTIVITY_INTERVAL);
        core.remote_motion(&mut host, 0, 0);
        assert_eq!(host.kept_active, 1);
        // Activité sur Salon (son propre clavier) : cette machine et Atelier.
        core.peer_active_at = Some(Instant::now() - ACTIVITY_INTERVAL);
        core.peer_event(&mut host, PeerEvent::UserActive { from: SALON });
        assert_eq!((host.user_active_sent.last(), host.kept_active), (Some(&ATELIER), 2));
    }

    #[test]
    fn activity_mirroring_can_be_turned_off() {
        let (mut core, mut host) = setup();
        core.opts.mirror_activity = false;
        go_remote(&mut core, &mut host);
        core.remote_motion(&mut host, 3, 0);
        core.peer_event(&mut host, PeerEvent::UserActive { from: ATELIER });
        assert_eq!(host.kept_active, 0);
        // Les pilotées décident pour elles-mêmes : elles sont toujours prévenues.
        core.peer_event(&mut host, PeerEvent::Switch { from: None, to: SwitchTo::Primary });
        core.peer_active_at = None;
        host.user_active_sent.clear();
        core.key(&mut host, KEY_A, true);
        assert_eq!(host.user_active_sent, [ATELIER]);
    }
}
