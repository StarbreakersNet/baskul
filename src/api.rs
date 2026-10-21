//! API locale du daemon, utilisée par l'app de réglages (`bascule-settings`,
//! en Tauri + Vue, dans `app/`).
//!
//! Un petit serveur HTTP écoute sur 127.0.0.1 (port aléatoire). Son port et un
//! jeton aléatoire sont écrits dans `api.json`, à côté du fichier de
//! configuration, lisible par l'utilisateur seulement. Chaque requête doit
//! présenter ce jeton, et l'en-tête `Host` est vérifié (contre le « DNS
//! rebinding ») : une page web ne peut ni lire ni modifier la configuration.

use crate::config::{self, Config};
use crate::status::{self, Conn};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const API_FILE: &str = "api.json";
const MAX_LINE: u64 = 8 * 1024;
const MAX_HEADERS: usize = 64;
const MAX_BODY: usize = 256 * 1024;

struct Server {
    path: PathBuf,
    port: u16,
    token: String,
    /// Démarrage de ce processus (ms) : l'app de réglages voit ainsi qu'un
    /// redémarrage a eu lieu.
    started: u64,
}

static RESTART: OnceLock<fn()> = OnceLock::new();

/// Fonction appelée pour redémarrer Bascule (après un enregistrement…).
/// Elle est appelée depuis un thread du serveur : elle doit seulement
/// prévenir la boucle principale.
pub fn set_restart_handler(f: fn()) {
    let _ = RESTART.set(f);
}

/// Démarre l'API et écrit son fichier de découverte.
pub fn start(config_path: &Path) -> Result<(), String> {
    let server = spawn(config_path.to_path_buf())?;
    let file = api_file(config_path);
    write_api_file(&file, server.port, &server.token)?;
    log::info!("API locale sur 127.0.0.1:{} ({})", server.port, file.display());
    Ok(())
}

/// Fichier de découverte de l'API, à côté du fichier de configuration.
pub fn api_file(config_path: &Path) -> PathBuf {
    config_path.parent().unwrap_or(Path::new(".")).join(API_FILE)
}

/// Lance l'app de réglages, installée à côté de Bascule.
pub fn launch_settings_app(config_path: &Path, welcome: bool) -> Result<Child, String> {
    let exe = settings_exe().ok_or("app de réglages introuvable")?;
    let mut cmd = Command::new(&exe);
    // Pour joindre l'API, et relancer Bascule avec la même configuration.
    cmd.env("BASCULE_API_FILE", api_file(config_path)).env("BASCULE_CONFIG", config_path);
    if welcome {
        cmd.arg("--bienvenue");
    }
    cmd.spawn().map_err(|e| format!("lancement de {} impossible : {e}", exe.display()))
}

fn settings_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) { "bascule-settings.exe" } else { "bascule-settings" };
    // À côté de Bascule (Windows, Linux, développement) ; sur macOS, dans
    // Bascule.app/Contents/Helpers.
    [dir.join(name), dir.join("../Helpers/BasculeSettings.app/Contents/MacOS").join(name)]
        .into_iter()
        .find(|p| p.is_file())
}

fn spawn(path: PathBuf) -> Result<Arc<Server>, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("API locale : {e}"))?;
    let port = listener.local_addr().map_err(|e| format!("API locale : {e}"))?.port();
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| format!("générateur aléatoire : {e}"))?;
    let token = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let started = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
    let server = Arc::new(Server { path, port, token, started });
    let s = server.clone();
    std::thread::Builder::new()
        .name("bascule-api".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let s = s.clone();
                let _ = std::thread::Builder::new().name("bascule-api-req".into()).spawn(move || serve(stream, &s));
            }
        })
        .map_err(|e| format!("API locale : {e}"))?;
    Ok(server)
}

/// Écrit `api.json`, lisible par l'utilisateur seulement.
fn write_api_file(path: &Path, port: u16, token: &str) -> Result<(), String> {
    let text = json!({ "port": port, "token": token, "pid": std::process::id(), "version": crate::VERSION });
    let tmp = path.with_extension("json.tmp");
    let _ = std::fs::remove_file(&tmp);
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let err = |e: std::io::Error| format!("écriture de {} impossible : {e}", path.display());
    opts.open(&tmp).and_then(|mut f| f.write_all(text.to_string().as_bytes())).map_err(err)?;
    std::fs::rename(&tmp, path).map_err(err)
}

// ---------------------------------------------------------------------------
// HTTP minimal
// ---------------------------------------------------------------------------

struct Request {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }
}

struct Response {
    status: u16,
    body: Value,
    /// Redémarrer Bascule une fois la réponse envoyée.
    restart: bool,
}

impl Response {
    fn json(status: u16, body: Value) -> Response {
        Response { status, body, restart: false }
    }

    fn error(status: u16, msg: &str) -> Response {
        Response::json(status, json!({ "error": msg }))
    }
}

fn read_line(r: &mut impl BufRead) -> Result<String, String> {
    let mut buf = Vec::new();
    r.by_ref().take(MAX_LINE).read_until(b'\n', &mut buf).map_err(|e| e.to_string())?;
    if !buf.ends_with(b"\n") {
        return Err("ligne trop longue ou incomplète".into());
    }
    let s = String::from_utf8(buf).map_err(|_| "en-tête invalide".to_string())?;
    Ok(s.trim_end_matches(['\r', '\n']).to_string())
}

fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut r = BufReader::new(stream);
    let line = read_line(&mut r)?;
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
        return Err("requête invalide".into());
    };
    let mut headers = Vec::new();
    loop {
        let l = read_line(&mut r)?;
        if l.is_empty() {
            break;
        }
        if headers.len() >= MAX_HEADERS {
            return Err("trop d'en-têtes".into());
        }
        if let Some((k, v)) = l.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    let mut req = Request { method: method.into(), target: target.into(), headers, body: Vec::new() };
    if req.header("transfer-encoding").is_some() {
        return Err("encodage de transfert non pris en charge".into());
    }
    let len: usize = match req.header("content-length") {
        Some(v) => v.parse().map_err(|_| "longueur invalide".to_string())?,
        None => 0,
    };
    if len > MAX_BODY {
        return Err("requête trop grande".into());
    }
    req.body = vec![0; len];
    r.read_exact(&mut req.body).map_err(|e| e.to_string())?;
    Ok(req)
}

fn write_response(stream: &mut TcpStream, resp: &Response) -> std::io::Result<()> {
    let reason = match resp.status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        422 => "Unprocessable Content",
        _ => "Internal Server Error",
    };
    let body = resp.body.to_string();
    let head = format!(
        "HTTP/1.1 {} {reason}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         X-Content-Type-Options: nosniff\r\n\
         Connection: close\r\n\r\n",
        resp.status,
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}

fn serve(mut stream: TcpStream, server: &Server) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let resp = match read_request(&mut stream) {
        Ok(req) => route(&req, server),
        Err(e) => Response::error(400, &e),
    };
    let sent = write_response(&mut stream, &resp).is_ok();
    let _ = stream.shutdown(Shutdown::Write);
    if sent && resp.restart {
        if let Some(restart) = RESTART.get() {
            log::info!("redémarrage demandé par l'app de réglages");
            restart();
        }
    }
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

fn route(req: &Request, server: &Server) -> Response {
    let local = [format!("127.0.0.1:{}", server.port), format!("localhost:{}", server.port)];
    if !req.header("host").is_some_and(|h| local.iter().any(|l| l == h)) {
        return Response::error(403, "hôte non autorisé");
    }
    if req.header("x-bascule-token") != Some(server.token.as_str()) {
        return Response::error(403, "jeton invalide");
    }
    if let Some(origin) = req.header("origin") {
        if !local.iter().any(|l| origin == format!("http://{l}")) {
            return Response::error(403, "origine non autorisée");
        }
    }
    let path = req.target.split('?').next().unwrap_or_default();
    let can_restart = RESTART.get().is_some();
    match (req.method.as_str(), path) {
        ("GET", "/api/status") => Response::json(200, status_info(server)),
        ("GET", "/api/config") => Response::json(200, config_info(&server.path)),
        ("POST", "/api/config") => match save(&server.path, &req.body) {
            Ok(()) => Response { restart: can_restart, ..Response::json(200, json!({ "restart": can_restart })) },
            Err(e) => Response::error(422, &e),
        },
        ("POST", "/api/restart") => {
            Response { restart: can_restart, ..Response::json(200, json!({ "restart": can_restart })) }
        }
        ("POST", "/api/open-file") => {
            crate::platform::open_in_editor(&server.path);
            Response::json(200, json!({}))
        }
        (_, "/api/status" | "/api/config" | "/api/restart" | "/api/open-file") => {
            Response::error(405, "méthode non autorisée")
        }
        _ => Response::error(404, "introuvable"),
    }
}

/// État de la connexion, pour l'affichage en direct.
fn status_info(server: &Server) -> Value {
    let s = status::snapshot();
    let (state, detail) = match &s.conn {
        Conn::Waiting(d) => ("waiting", d),
        Conn::Connected(d) => ("connected", d),
        Conn::Error(d) => ("error", d),
    };
    json!({
        "state": state,
        "detail": detail,
        "summary": s.summary(),
        "remote_active": s.remote_active,
        "locked": s.locked,
        "warning": s.warning,
        "version": crate::VERSION,
        "started": server.started,
    })
}

/// Configuration actuelle et informations utiles à l'app de réglages.
fn config_info(path: &Path) -> Value {
    let (config, error) = match config::load_or_create(path) {
        Ok((cfg, _)) => (serde_json::to_value(&cfg).unwrap_or(Value::Null), Value::Null),
        Err(e) => (Value::Null, Value::String(e)),
    };
    json!({
        "config": config,
        "error": error,
        "path": path.display().to_string(),
        "os": std::env::consts::OS,
        "hostname": config::hostname(),
        "address": local_address(),
        "version": crate::VERSION,
    })
}

fn save(path: &Path, body: &[u8]) -> Result<(), String> {
    let cfg: Config = serde_json::from_slice(body).map_err(|e| format!("données reçues invalides : {e}"))?;
    cfg.validate()?;
    let old = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("impossible de lire {} : {e}", path.display())),
    };
    let text = config::update_text(&old, &cfg)?;
    // Le fichier écrit doit redonner exactement la configuration reçue.
    let written = serde_json::to_value(Config::parse(&text)?).map_err(|e| e.to_string())?;
    if written != serde_json::to_value(&cfg).map_err(|e| e.to_string())? {
        return Err("erreur interne : le fichier produit ne correspond pas aux réglages".into());
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("impossible d'écrire {} : {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("impossible d'écrire {} : {e}", path.display()))?;
    log::info!("configuration modifiée depuis l'app de réglages");
    Ok(())
}

/// Adresse IP de cette machine sur le réseau local (celle de l'interface
/// de la route par défaut ; aucun paquet n'est envoyé).
fn local_address() -> Option<String> {
    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.connect(("192.0.2.1", 9)).ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_unspecified() && !ip.is_loopback()).then(|| ip.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> TempDir {
            let mut b = [0u8; 8];
            getrandom::fill(&mut b).unwrap();
            let dir = std::env::temp_dir().join(format!("bascule-test-{}", u64::from_le_bytes(b)));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Envoie une requête brute ; renvoie le code et le corps de la réponse.
    fn request(port: u16, head: &str, body: &str) -> (u16, String) {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let req = format!("{head}\r\nContent-Length: {}\r\n\r\n{body}", body.len());
        s.write_all(req.as_bytes()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let status = out[9..12].parse().unwrap();
        let body = out.split_once("\r\n\r\n").unwrap().1.to_string();
        (status, body)
    }

    #[test]
    fn api_file_is_private() {
        let tmp = TempDir::new();
        let file = api_file(&tmp.0.join("config.toml"));
        write_api_file(&file, 1234, "abcd").unwrap();
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!((v["port"].as_u64(), v["token"].as_str()), (Some(1234), Some("abcd")));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
        }
        // Réécriture (redémarrage).
        write_api_file(&file, 5678, "efgh").unwrap();
        assert!(std::fs::read_to_string(&file).unwrap().contains("5678"));
    }

    #[test]
    fn api_requests() {
        let tmp = TempDir::new();
        let path = tmp.0.join("config.toml");
        std::fs::write(&path, config::default_config_text("primary", "abcd-efgh-jkmn-pqrs")).unwrap();
        let server = spawn(path.clone()).unwrap();
        let (port, token) = (server.port, server.token.clone());
        let host = format!("Host: 127.0.0.1:{port}");
        let auth = format!("{host}\r\nX-Bascule-Token: {token}");

        // Sans jeton, depuis un autre nom d'hôte ou une autre origine : refusé.
        assert_eq!(request(port, &format!("GET /api/config HTTP/1.1\r\n{host}"), "").0, 403);
        let rebinding = format!("GET /api/config HTTP/1.1\r\nHost: attaquant.example\r\nX-Bascule-Token: {token}");
        assert_eq!(request(port, &rebinding, "").0, 403);
        let evil = format!("POST /api/config HTTP/1.1\r\n{auth}\r\nOrigin: http://attaquant.example");
        assert_eq!(request(port, &evil, "{}").0, 403);
        assert_eq!(request(port, &format!("GET / HTTP/1.1\r\n{auth}"), "").0, 404);

        let (code, body) = request(port, &format!("GET /api/status HTTP/1.1\r\n{auth}"), "");
        assert_eq!(code, 200);
        let st: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(st["version"], crate::VERSION);
        assert!(st["started"].as_u64().unwrap() > 0);

        let (code, body) = request(port, &format!("GET /api/config HTTP/1.1\r\n{auth}"), "");
        assert_eq!(code, 200);
        let info: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(info["config"]["network"]["secret"], "abcd-efgh-jkmn-pqrs");
        assert_eq!(info["config"]["hotkeys"]["switch_to_secondary"], "Shift+F12");

        // Enregistrement.
        let mut cfg = info["config"].clone();
        cfg["network"]["connect"] = json!("192.168.1.30");
        cfg["screen"]["edge_switching"] = json!(false);
        let post = format!("POST /api/config HTTP/1.1\r\n{auth}");
        let (code, body) = request(port, &post, &cfg.to_string());
        assert_eq!(code, 200, "{body}");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("connect = \"192.168.1.30\"\n"));
        assert!(text.contains("edge_switching = false\n"));
        assert!(text.contains("# Secret partagé"));

        // Réglage invalide : refusé avec un message, fichier inchangé.
        cfg["mouse"]["speed"] = json!(100);
        let (code, body) = request(port, &post, &cfg.to_string());
        assert_eq!(code, 422);
        assert!(body.contains("speed"), "{body}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        // Champ inconnu : refusé.
        cfg["mouse"]["speed"] = json!(1.0);
        cfg["mouse"]["vitesse"] = json!(2);
        assert_eq!(request(port, &post, &cfg.to_string()).0, 422);
    }
}
