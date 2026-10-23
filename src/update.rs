//! Mises à jour, façon Discord : Baskul cherche une nouvelle version parmi
//! les Releases de GitHub, la télécharge en arrière-plan et vérifie son
//! empreinte SHA-256 ; il ne reste qu'à redémarrer pour l'installer (menu de
//! l'icône ou app de réglages).
//!
//! Les téléchargements passent par `curl`, présent sur macOS et sur Windows
//! (depuis Windows 10 1803) : aucune bibliothèque TLS à embarquer.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Dépôt dont les Releases sont suivies.
pub const REPO: &str = "StarbreakersNet/baskul";
/// Première vérification après le démarrage, puis intervalle.
const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 3600);
/// Fichier des empreintes publié avec chaque Release.
const SUMS: &str = "SHA256SUMS";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum State {
    /// Pas encore vérifié.
    Idle,
    Checking,
    UpToDate,
    /// Nouvelle version, à installer à la main (Linux, ou installation
    /// automatique impossible).
    Available {
        version: String,
        notes: String,
        url: String,
    },
    Downloading {
        version: String,
        done: u64,
        total: u64,
    },
    /// Téléchargée et vérifiée : un redémarrage l'installe.
    Ready {
        version: String,
        notes: String,
        url: String,
    },
    Installing {
        version: String,
    },
    Error {
        message: String,
    },
}

struct Shared {
    state: State,
    /// Dernière vérification (ms depuis 1970).
    checked_at: Option<u64>,
    /// Archive téléchargée, prête à installer.
    archive: Option<PathBuf>,
    busy: bool,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared { state: State::Idle, checked_at: None, archive: None, busy: false });

fn shared() -> std::sync::MutexGuard<'static, Shared> {
    SHARED.lock().unwrap_or_else(|e| e.into_inner())
}

fn set_state(state: State) {
    shared().state = state;
}

pub fn state() -> State {
    shared().state.clone()
}

/// État pour l'app de réglages.
pub fn info() -> serde_json::Value {
    let s = shared();
    let mut v = serde_json::to_value(&s.state).unwrap_or_default();
    v["current"] = crate::VERSION.into();
    v["checkedAt"] = s.checked_at.into();
    v["canInstall"] = can_install().into();
    v["releases"] = format!("https://github.com/{REPO}/releases").into();
    v
}

/// Version prête à installer (menu de l'icône).
pub fn ready_version() -> Option<String> {
    match state() {
        State::Ready { version, .. } => Some(version),
        _ => None,
    }
}

/// Vérifications automatiques : peu après le démarrage, puis toutes les six
/// heures.
pub fn start_auto_check() {
    let _ = std::thread::Builder::new().name("baskul-update".into()).spawn(|| {
        std::thread::sleep(FIRST_CHECK);
        loop {
            check(false);
            std::thread::sleep(CHECK_EVERY);
        }
    });
}

/// Vérifie maintenant, dans un thread (bouton de l'app de réglages).
pub fn check_in_background() {
    let _ = std::thread::Builder::new().name("baskul-update-check".into()).spawn(|| check(true));
}

/// Cherche une nouvelle version et la télécharge.
fn check(manual: bool) {
    {
        let mut s = shared();
        // Déjà en cours, ou déjà prête : rien à refaire.
        if s.busy || matches!(s.state, State::Ready { .. } | State::Installing { .. }) {
            return;
        }
        s.busy = true;
        s.state = State::Checking;
    }
    let result = find_update();
    {
        let mut s = shared();
        s.checked_at = Some(now_ms());
    }
    let release = match result {
        Ok(Some(r)) => r,
        Ok(None) => {
            if manual {
                log::info!("Baskul est à jour ({})", crate::VERSION);
            }
            finish(State::UpToDate);
            return;
        }
        Err(e) => {
            log::warn!("recherche de mise à jour : {e}");
            finish(State::Error { message: e });
            return;
        }
    };
    log::info!("nouvelle version disponible : {}", release.version);
    let available =
        State::Available { version: release.version.clone(), notes: release.notes.clone(), url: release.url.clone() };
    let (Some(asset), true) = (release.asset.clone(), can_install()) else {
        finish(available);
        return;
    };
    match download(&release, &asset) {
        Ok(path) => {
            log::info!("version {} téléchargée et vérifiée : prête à installer", release.version);
            let mut s = shared();
            s.archive = Some(path);
            s.busy = false;
            s.state = State::Ready { version: release.version, notes: release.notes, url: release.url };
        }
        Err(e) => {
            log::warn!("téléchargement de la mise à jour : {e}");
            finish(available);
        }
    }
}

fn finish(state: State) {
    let mut s = shared();
    s.state = state;
    s.busy = false;
}

/// Installation automatique possible : macOS (dans Baskul.app) et Windows.
fn can_install() -> bool {
    match std::env::consts::OS {
        "macos" => std::env::current_exe().ok().and_then(|e| crate::platform::app_bundle(&e)).is_some(),
        "windows" => true,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Recherche
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GhAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Release {
    version: String,
    notes: String,
    url: String,
    /// Archive pour ce système.
    asset: Option<GhAsset>,
    sums: Option<GhAsset>,
}

/// Archive publiée pour ce système.
fn asset_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "Baskul-macOS.zip",
        "windows" => "Baskul-Windows.zip",
        _ => "Baskul-Linux.tar.gz",
    }
}

fn find_update() -> Result<Option<Release>, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let out = curl().args(["-H", "Accept: application/vnd.github+json", &url]).output().map_err(curl_missing)?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(if err.contains("404") {
            "aucune version publiée (dépôt introuvable ou privé)".into()
        } else {
            format!("GitHub injoignable ({})", err.trim())
        });
    }
    parse_release(&String::from_utf8_lossy(&out.stdout), crate::VERSION, asset_name())
}

fn parse_release(json: &str, current: &str, asset: &str) -> Result<Option<Release>, String> {
    let r: GhRelease = serde_json::from_str(json).map_err(|e| format!("réponse de GitHub illisible : {e}"))?;
    let version = r.tag_name.trim().trim_start_matches(['v', 'V']).to_string();
    if !is_newer(&version, current) {
        return Ok(None);
    }
    let find = |name: &str| r.assets.iter().find(|a| a.name == name).cloned();
    Ok(Some(Release {
        notes: r.body.unwrap_or_default(),
        url: r.html_url,
        asset: find(asset),
        sums: find(SUMS),
        version,
    }))
}

/// Numéros de version (`0.7.1` → `[0, 7, 1]`) ; un suffixe (`-beta`) est
/// ignoré.
fn version_numbers(v: &str) -> Option<Vec<u64>> {
    let v = v.split(['-', '+']).next()?;
    v.split('.').map(|n| n.parse().ok()).collect()
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (version_numbers(candidate), version_numbers(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Téléchargement
// ---------------------------------------------------------------------------

fn curl() -> Command {
    let mut c = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    c.args(["-fsSL", "--connect-timeout", "20", "--retry", "2", "-A"]).arg(format!("Baskul/{}", crate::VERSION));
    c.stdin(Stdio::null());
    no_window(&mut c);
    c
}

/// Windows : pas de fenêtre de console pour les outils lancés.
fn no_window(_c: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        _c.creation_flags(CREATE_NO_WINDOW);
    }
}

fn curl_missing(e: std::io::Error) -> String {
    format!("curl introuvable ({e})")
}

fn update_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("Baskul").join("update")
}

fn download(release: &Release, asset: &GhAsset) -> Result<PathBuf, String> {
    let dir = update_dir();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{} : {e}", dir.display()))?;
    let expected = match &release.sums {
        Some(sums) => {
            let out = curl().arg(&sums.browser_download_url).output().map_err(curl_missing)?;
            if !out.status.success() {
                return Err(format!("{SUMS} : téléchargement impossible"));
            }
            Some(
                checksum_for(&String::from_utf8_lossy(&out.stdout), &asset.name)
                    .ok_or_else(|| format!("{SUMS} : pas d'empreinte pour {}", asset.name))?,
            )
        }
        None => {
            log::warn!("pas de fichier {SUMS} dans la Release : seule la taille est vérifiée");
            None
        }
    };
    let path = dir.join(&asset.name);
    let mut child = curl()
        .arg("-o")
        .arg(&path)
        .arg(&asset.browser_download_url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(curl_missing)?;
    let started = Instant::now();
    let status = loop {
        if let Some(st) = child.try_wait().map_err(|e| e.to_string())? {
            break st;
        }
        if started.elapsed() > Duration::from_secs(30 * 60) {
            let _ = child.kill();
            return Err("téléchargement trop long".into());
        }
        let done = std::fs::metadata(&path).map_or(0, |m| m.len());
        set_state(State::Downloading { version: release.version.clone(), done, total: asset.size });
        std::thread::sleep(Duration::from_millis(250));
    };
    if !status.success() {
        return Err(format!("téléchargement de {} impossible", asset.name));
    }
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    if data.len() as u64 != asset.size {
        return Err(format!("{} : taille inattendue ({} octets au lieu de {})", asset.name, data.len(), asset.size));
    }
    if let Some(expected) = expected {
        let actual = hex(&Sha256::digest(&data));
        if actual != expected {
            let _ = std::fs::remove_file(&path);
            return Err(format!("{} : empreinte SHA-256 incorrecte", asset.name));
        }
    }
    Ok(path)
}

/// Empreinte d'un fichier dans un `SHA256SUMS` (`<hex>  <nom>` ou
/// `<hex> *<nom>`).
fn checksum_for(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (hash, file) = line.trim().split_once(char::is_whitespace)?;
        let file = file.trim_start().trim_start_matches('*');
        (file == name && hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| hash.to_ascii_lowercase())
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

// ---------------------------------------------------------------------------
// Installation
// ---------------------------------------------------------------------------

/// Installe la version téléchargée puis redémarre Baskul (menu de l'icône,
/// app de réglages). Les fichiers sont remplacés depuis un thread ; la boucle
/// principale redémarre ensuite comme après un changement de réglages.
pub fn install() -> Result<(), String> {
    let (archive, version) = {
        let mut s = shared();
        let (Some(archive), State::Ready { version, .. }) = (s.archive.clone(), s.state.clone()) else {
            return Err("aucune mise à jour prête".into());
        };
        s.state = State::Installing { version: version.clone() };
        (archive, version)
    };
    let _ = std::thread::Builder::new().name("baskul-update-install".into()).spawn(move || {
        log::info!("installation de la version {version}…");
        let result = if cfg!(target_os = "macos") { install_macos(&archive) } else { install_files(&archive) };
        match result {
            Ok(()) => {
                log::info!("version {version} installée : redémarrage");
                crate::api::request_restart();
            }
            Err(e) => {
                log::error!("installation de la mise à jour impossible : {e}");
                let url = format!("https://github.com/{REPO}/releases/tag/{version}");
                let mut s = shared();
                s.archive = None;
                s.state = State::Error { message: format!("installation impossible : {e} — {url}") };
            }
        }
    });
    Ok(())
}

fn run(program: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    let mut c = Command::new(program);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::null());
    no_window(&mut c);
    let out = c.output().map_err(|e| format!("{program} : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("{program} : {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

/// macOS : la nouvelle Baskul.app remplace l'ancienne, au même endroit.
fn install_macos(archive: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let bundle = crate::platform::app_bundle(&exe).ok_or("Baskul ne tourne pas depuis Baskul.app")?;
    let parent = bundle.parent().ok_or("dossier de Baskul.app introuvable")?;
    // Dossier de travail à côté de l'app : les renommages restent sur le
    // même disque.
    let work = parent.join(format!(".baskul-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work)
        .map_err(|e| format!("écriture impossible dans {} ({e}) : droits insuffisants ?", parent.display()))?;
    let result = (|| {
        run("/usr/bin/ditto", &["-x".as_ref(), "-k".as_ref(), archive.as_os_str(), work.as_os_str()])?;
        let new = work.join("Baskul.app");
        if !new.join("Contents/MacOS/baskul").is_file() {
            return Err("archive inattendue (Baskul.app absente)".to_string());
        }
        let old = work.join("Baskul-old.app");
        std::fs::rename(&bundle, &old).map_err(|e| format!("{} : {e}", bundle.display()))?;
        if let Err(e) = std::fs::rename(&new, &bundle) {
            let _ = std::fs::rename(&old, &bundle);
            return Err(format!("{} : {e}", bundle.display()));
        }
        // Téléchargée par curl, l'app n'a normalement pas d'attribut de
        // quarantaine ; on s'en assure.
        let _ = run("/usr/bin/xattr", &["-dr".as_ref(), "com.apple.quarantine".as_ref(), bundle.as_os_str()]);
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

/// Windows (et Linux) : les programmes sont remplacés à côté de l'actuel.
/// Un programme en cours d'exécution ne peut pas être effacé mais peut être
/// renommé : l'ancien devient `….old`, effacé au démarrage suivant.
fn install_files(archive: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("dossier de Baskul introuvable")?.to_path_buf();
    let work = update_dir().join("extract");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let tar = if cfg!(windows) { "tar.exe" } else { "tar" };
    run(tar, &["-xf".as_ref(), archive.as_os_str(), "-C".as_ref(), work.as_os_str()])?;
    // Archive : un dossier (Baskul\ ou baskul/) avec les deux programmes.
    let ext = std::env::consts::EXE_SUFFIX;
    let main = format!("baskul{ext}");
    let src = std::fs::read_dir(&work)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.join(&main).is_file())
        .ok_or("archive inattendue (programme absent)")?;
    let settings = format!("baskul-settings{ext}");
    replace_programs(&[(src.join(&main), exe.clone()), (src.join(&settings), dir.join(&settings))])?;
    let _ = std::fs::remove_dir_all(&work);
    Ok(())
}

/// Remplace chaque programme `(nouveau, cible)` ; en cas d'échec, tout
/// revient à l'état d'origine.
fn replace_programs(pairs: &[(PathBuf, PathBuf)]) -> Result<(), String> {
    let mut done: Vec<(&Path, PathBuf)> = Vec::new();
    for (new, target) in pairs {
        if !new.is_file() {
            continue;
        }
        let old = old_path(target);
        let _ = std::fs::remove_file(&old);
        let moved = target.exists();
        let step = (|| {
            if moved {
                std::fs::rename(target, &old)?;
            }
            std::fs::copy(new, target).map(drop)
        })();
        if let Err(e) = step {
            // Retour à l'état d'origine.
            if moved {
                let _ = std::fs::rename(&old, target);
            }
            for (target, old) in done.into_iter().rev() {
                let _ = std::fs::remove_file(target);
                let _ = std::fs::rename(&old, target);
            }
            return Err(format!("{} : {e}", target.display()));
        }
        if moved {
            done.push((target, old));
        }
    }
    Ok(())
}

fn old_path(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".old");
    target.with_file_name(name)
}

/// Au démarrage : restes d'une installation précédente.
pub fn cleanup() {
    let Ok(exe) = std::env::current_exe() else { return };
    let ext = std::env::consts::EXE_SUFFIX;
    if let Some(dir) = exe.parent() {
        for name in [format!("baskul{ext}"), format!("baskul-settings{ext}")] {
            let _ = std::fs::remove_file(old_path(&dir.join(name)));
        }
    }
    let _ = std::fs::remove_dir_all(update_dir());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert!(is_newer("0.7.0", "0.6.0"));
        assert!(is_newer("0.10.0", "0.9.3"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("0.6.0", "0.6.0"));
        assert!(!is_newer("0.5.9", "0.6.0"));
        assert!(!is_newer("pas-une-version", "0.6.0"));
        assert!(!is_newer("0.7.0-beta", "0.7.0"));
    }

    #[test]
    fn release_is_read_from_github_json() {
        let json = r#"{
            "tag_name": "v0.7.1",
            "html_url": "https://github.com/StarbreakersNet/baskul/releases/tag/0.7.1",
            "body": "Corrections",
            "assets": [
                {"name": "Baskul-macOS.zip", "size": 1234, "browser_download_url": "https://x/mac.zip"},
                {"name": "Baskul-Windows.zip", "size": 999, "browser_download_url": "https://x/win.zip"},
                {"name": "SHA256SUMS", "size": 300, "browser_download_url": "https://x/sums"}
            ]
        }"#;
        let r = parse_release(json, "0.7.0", "Baskul-Windows.zip").unwrap().unwrap();
        assert_eq!(r.version, "0.7.1");
        assert_eq!(r.notes, "Corrections");
        assert_eq!(r.asset.unwrap().browser_download_url, "https://x/win.zip");
        assert_eq!(r.sums.unwrap().name, "SHA256SUMS");
        // Déjà à jour.
        assert_eq!(parse_release(json, "0.7.1", "Baskul-Windows.zip").unwrap(), None);
        // Pas d'archive pour ce système.
        assert_eq!(parse_release(json, "0.7.0", "Baskul-Linux.tar.gz").unwrap().unwrap().asset, None);
        assert!(parse_release("<html>", "0.7.0", "x").is_err());
    }

    #[test]
    fn checksums_are_found_by_file_name() {
        let a = "a".repeat(64);
        let b = "B".repeat(64);
        let sums = format!("{a}  Baskul-macOS.zip\n{b} *Baskul-Windows.zip\nnimportequoi\n");
        assert_eq!(checksum_for(&sums, "Baskul-macOS.zip"), Some(a));
        assert_eq!(checksum_for(&sums, "Baskul-Windows.zip"), Some("b".repeat(64)));
        assert_eq!(checksum_for(&sums, "Baskul-Linux.tar.gz"), None);
        assert_eq!(hex(&Sha256::digest(b"abc"))[..8], *"ba7816bf");
    }

    #[test]
    fn old_programs_keep_their_name() {
        assert_eq!(old_path(Path::new("/opt/baskul/baskul")), PathBuf::from("/opt/baskul/baskul.old"));
        assert_eq!(old_path(Path::new("C:/Baskul/baskul.exe")), PathBuf::from("C:/Baskul/baskul.exe.old"));
    }

    #[test]
    fn programs_are_replaced_with_a_way_back() {
        let dir = std::env::temp_dir().join(format!("baskul-update-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (new, app) = (dir.join("new"), dir.join("app"));
        std::fs::create_dir_all(&new).unwrap();
        std::fs::create_dir_all(&app).unwrap();
        let read = |p: &Path| std::fs::read_to_string(p).unwrap();
        for name in ["baskul", "baskul-settings"] {
            std::fs::write(new.join(name), format!("{name} 2")).unwrap();
            std::fs::write(app.join(name), format!("{name} 1")).unwrap();
        }
        let pairs = |second: PathBuf| [(new.join("baskul"), app.join("baskul")), (new.join("baskul-settings"), second)];
        // Échec sur le second programme (dossier absent) : le premier revient.
        assert!(replace_programs(&pairs(dir.join("absent").join("baskul-settings"))).is_err());
        assert_eq!(read(&app.join("baskul")), "baskul 1");
        assert!(!old_path(&app.join("baskul")).exists());
        // Réussite : nouveaux programmes, anciens mis de côté.
        replace_programs(&pairs(app.join("baskul-settings"))).unwrap();
        assert_eq!(read(&app.join("baskul")), "baskul 2");
        assert_eq!(read(&app.join("baskul-settings")), "baskul-settings 2");
        assert_eq!(read(&old_path(&app.join("baskul"))), "baskul 1");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
