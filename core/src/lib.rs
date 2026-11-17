//! Cœur de Baskul, indépendant de toute interface : réseau chiffré,
//! protocole, logique de la machine principale et des pilotées,
//! presse-papiers, configuration, état.
//!
//! Il tourne dans ses propres threads et n'attend jamais l'interface : la
//! couche de chaque système (capture et injection du clavier et de la
//! souris, icône, fenêtres) l'appelle et lui fournit ses services par des
//! traits (`primary::Host`, `secondary::Emulator`) et des fonctions
//! (`services::ClipboardHooks`).

#![allow(clippy::collapsible_if, clippy::too_many_arguments)]

pub mod activity;
pub mod capture;
pub mod clipboard;
pub mod config;
pub mod crypto;
pub mod discovery;
pub mod geometry;
pub mod hotkey;
pub mod i18n;
pub mod iface;
pub mod keymap;
pub mod latency;
pub mod net;
pub mod primary;
pub mod protocol;
pub mod secondary;
pub mod services;
pub mod smooth;
pub mod status;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
