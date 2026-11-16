//! Icône de Baskul (zone de notification, barre des menus), dessinée par
//! Tauri à partir du modèle de `baskul_desktop::tray` : image, info-bulle et
//! menu. Les choix du menu retournent à Baskul, qui les exécute sur le thread
//! de son système.

use baskul_desktop::tray::{self, Entry, Model};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Wry};

const ID: &str = "baskul";

/// Relecture du modèle sans signal de changement (mise à jour prête,
/// lancement à l'ouverture de session, thème de Windows…).
const POLL: Duration = Duration::from_secs(1);

/// Affiche l'icône, puis la tient à jour. Sans modèle (configuration
/// invalide : Baskul ne tourne pas), pas d'icône.
pub fn install(app: &AppHandle) {
    let Some(model) = tray::model() else { return };
    if !supported() {
        log::warn!(
            "icône indisponible : bibliothèque libayatana-appindicator3 absente (paquet libayatana-appindicator3-1)"
        );
        return;
    }
    let icon = match build(app, &model) {
        Ok(icon) => icon,
        Err(e) => {
            log::error!("icône de Baskul impossible à afficher : {e}");
            return;
        }
    };
    let (tx, rx) = mpsc::channel::<()>();
    tray::on_change(move || {
        let _ = tx.send(());
    });
    let spawned = std::thread::Builder::new().name("baskul-tray".into()).spawn(move || refresh(icon, model, rx));
    if let Err(e) = spawned {
        log::error!("mise à jour de l'icône impossible : {e}");
    }
}

fn build(app: &AppHandle, model: &Model) -> tauri::Result<TrayIcon> {
    TrayIconBuilder::with_id(ID)
        .icon(image(model))
        .icon_as_template(model.image.template)
        .tooltip(&model.tooltip)
        .menu(&menu(app, model)?)
        // Clic gauche aussi : le menu, comme le clic droit.
        .show_menu_on_left_click(true)
        .on_menu_event(|_app, event| tray::act(event.id().as_ref()))
        .build(app)
}

/// Redessine l'icône à chaque changement, et au moins chaque seconde.
fn refresh(icon: TrayIcon, mut shown: Model, rx: mpsc::Receiver<()>) {
    loop {
        match rx.recv_timeout(POLL) {
            Ok(()) | Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        // Plusieurs changements d'un coup : un seul nouveau dessin.
        while rx.try_recv().is_ok() {}
        let Some(model) = tray::model() else { continue };
        if let Err(e) = apply(&icon, &shown, &model) {
            log::warn!("mise à jour de l'icône : {e}");
        }
        shown = model;
    }
}

/// Ne change que ce qui a changé : un menu ouvert reste ouvert tant que ses
/// entrées ne changent pas.
fn apply(icon: &TrayIcon, shown: &Model, model: &Model) -> tauri::Result<()> {
    if model.image != shown.image {
        icon.set_icon(Some(image(model)))?;
        icon.set_icon_as_template(model.image.template)?;
    }
    if model.tooltip != shown.tooltip {
        icon.set_tooltip(Some(&model.tooltip))?;
    }
    if model.menu != shown.menu {
        icon.set_menu(Some(menu(icon.app_handle(), model)?))?;
    }
    Ok(())
}

fn image(model: &Model) -> Image<'static> {
    Image::new_owned(model.image.rgba.clone(), model.image.size, model.image.size)
}

fn menu(app: &AppHandle, model: &Model) -> tauri::Result<Menu<Wry>> {
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    // Les textes sont des entrées sans action : un identifiant propre à
    // chacune, ignoré par Baskul.
    for (i, entry) in model.menu.iter().enumerate() {
        match entry {
            Entry::Label(text) => {
                items.push(Box::new(MenuItem::with_id(app, format!("label:{i}"), text, false, None::<&str>)?))
            }
            Entry::Item { action, label, checked: None } => {
                items.push(Box::new(MenuItem::with_id(app, action.id(), label, true, None::<&str>)?))
            }
            Entry::Item { action, label, checked: Some(on) } => {
                items.push(Box::new(CheckMenuItem::with_id(app, action.id(), label, true, *on, None::<&str>)?))
            }
            Entry::Separator => items.push(Box::new(PredefinedMenuItem::separator(app)?)),
        }
    }
    let refs: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &refs)
}

/// Linux : l'icône passe par libayatana-appindicator3 (ou libappindicator3),
/// chargée à la demande ; absente, la créer arrêterait l'app.
#[cfg(target_os = "linux")]
fn supported() -> bool {
    ["libayatana-appindicator3.so.1", "libappindicator3.so.1", "libayatana-appindicator3.so", "libappindicator3.so"]
        .iter()
        .any(|name| {
            let Ok(name) = std::ffi::CString::new(*name) else { return false };
            // SAFETY: nom terminé par un zéro ; la bibliothèque est refermée
            // aussitôt (elle sera chargée par Tauri).
            unsafe {
                let lib = libc::dlopen(name.as_ptr(), libc::RTLD_LAZY);
                if lib.is_null() {
                    false
                } else {
                    libc::dlclose(lib);
                    true
                }
            }
        })
}

#[cfg(not(target_os = "linux"))]
fn supported() -> bool {
    true
}
