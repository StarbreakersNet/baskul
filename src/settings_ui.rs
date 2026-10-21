//! Page de réglages : un petit serveur HTTP local sert une page web
//! (`settings.html`) et une API JSON qui lit et modifie le fichier de
//! configuration.
//!
//! Sécurité : le serveur n'écoute que sur 127.0.0.1, vérifie l'en-tête `Host`
//! (contre le « DNS rebinding ») et exige un jeton aléatoire, passé dans un
//! en-tête, pour chaque appel à l'API. Une page d'un autre site ne peut donc
//! ni lire ni modifier la configuration.

use crate::config::{self, Config};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

const PAGE: &str = include_str!("settings.html");
const MAX_LINE: u64 = 8 * 1024;
const MAX_HEADERS: usize = 64;
const MAX_BODY: usize = 256 * 1024;

struct Server {
    path: PathBuf,
    port: u16,
    token: String,
}

static SERVER: OnceLock<Result<Arc<Server>, String>> = OnceLock::new();
static RESTART: OnceLock<fn()> = OnceLock::new();

/// Fonction appelée après un enregistrement pour redémarrer Bascule. Elle
/// est appelée depuis le thread du serveur : elle doit seulement prévenir la
/// boucle principale.
pub fn set_restart_handler(f: fn()) {
    let _ = RESTART.set(f);
}

/// Adresse de la page de réglages ; le serveur démarre au premier appel.
/// `welcome` : affiche l'aide du premier lancement.
pub fn url(config_path: &Path, welcome: bool) -> Result<String, String> {
    let server = SERVER.get_or_init(|| start(config_path.to_path_buf())).clone()?;
    let welcome = if welcome { "&bienvenue" } else { "" };
    Ok(format!("http://127.0.0.1:{}/?t={}{welcome}", server.port, server.token))
}

fn start(path: PathBuf) -> Result<Arc<Server>, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| format!("serveur de réglages : {e}"))?;
    let port = listener.local_addr().map_err(|e| format!("serveur de réglages : {e}"))?.port();
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| format!("générateur aléatoire : {e}"))?;
    let token = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let server = Arc::new(Server { path, port, token });
    let s = server.clone();
    std::thread::Builder::new()
        .name("bascule-settings".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let s = s.clone();
                let _ =
                    std::thread::Builder::new().name("bascule-settings-req".into()).spawn(move || serve(stream, &s));
            }
        })
        .map_err(|e| format!("serveur de réglages : {e}"))?;
    log::info!("page de réglages : http://127.0.0.1:{port}/");
    Ok(server)
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
    content_type: &'static str,
    body: Vec<u8>,
    /// Redémarrer Bascule une fois la réponse envoyée.
    restart: bool,
}

impl Response {
    fn json(status: u16, v: Value) -> Response {
        Response { status, content_type: "application/json; charset=utf-8", body: v.to_string().into(), restart: false }
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
    let head = format!(
        "HTTP/1.1 {} {reason}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         X-Content-Type-Options: nosniff\r\n\
         Referrer-Policy: no-referrer\r\n\
         Content-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; \
         connect-src 'self'; img-src data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'\r\n\
         Connection: close\r\n\r\n",
        resp.status,
        resp.content_type,
        resp.body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&resp.body)?;
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
            log::info!("nouvelle configuration enregistrée : redémarrage");
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
    let path = req.target.split(['?', '#']).next().unwrap_or_default();
    if path == "/" {
        if req.method != "GET" {
            return Response::error(405, "méthode non autorisée");
        }
        return Response { status: 200, content_type: "text/html; charset=utf-8", body: PAGE.into(), restart: false };
    }
    if !path.starts_with("/api/") {
        return Response::error(404, "page introuvable");
    }
    if req.header("x-bascule-token") != Some(server.token.as_str()) {
        return Response::error(403, "jeton invalide : rouvrez les réglages depuis le menu de Bascule");
    }
    if let Some(origin) = req.header("origin") {
        if !local.iter().any(|l| origin == format!("http://{l}")) {
            return Response::error(403, "origine non autorisée");
        }
    }
    match (req.method.as_str(), path) {
        ("GET", "/api/config") => Response::json(200, config_info(&server.path)),
        ("POST", "/api/config") => match save(&server.path, &req.body) {
            Ok(()) => {
                let restart = RESTART.get().is_some();
                Response { restart, ..Response::json(200, json!({ "ok": true, "restart": restart })) }
            }
            Err(e) => Response::error(422, &e),
        },
        ("POST", "/api/open-file") => {
            crate::platform::open_in_editor(&server.path);
            Response::json(200, json!({ "ok": true }))
        }
        (_, "/api/config" | "/api/open-file") => Response::error(405, "méthode non autorisée"),
        _ => Response::error(404, "introuvable"),
    }
}

/// Configuration actuelle et informations utiles à la page.
fn config_info(path: &Path) -> Value {
    let (config, error) = match config::load_or_create(path) {
        Ok((cfg, _)) => (serde_json::to_value(&cfg).unwrap_or(Value::Null), Value::Null),
        Err(e) => (Value::Null, Value::String(e)),
    };
    let running = RESTART.get().is_some();
    let status = if running {
        let snap = crate::status::snapshot();
        json!({ "text": snap.summary(), "connected": snap.is_connected() })
    } else {
        Value::Null
    };
    json!({
        "config": config,
        "error": error,
        "path": path.display().to_string(),
        "os": std::env::consts::OS,
        "hostname": config::hostname(),
        "address": local_address(),
        "running": running,
        "status": status,
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
    log::info!("configuration modifiée depuis la page de réglages");
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

    struct TempConfig(PathBuf);

    impl TempConfig {
        fn new(text: &str) -> TempConfig {
            let mut b = [0u8; 8];
            getrandom::fill(&mut b).unwrap();
            let dir = std::env::temp_dir().join(format!("bascule-test-{}", u64::from_le_bytes(b)));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("config.toml");
            std::fs::write(&path, text).unwrap();
            TempConfig(path)
        }
    }

    impl Drop for TempConfig {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
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
    fn page_and_api() {
        let tmp = TempConfig::new(&config::default_config_text("primary", "abcd-efgh-jkmn-pqrs"));
        let server = start(tmp.0.clone()).unwrap();
        let (port, token) = (server.port, server.token.clone());
        let host = format!("Host: 127.0.0.1:{port}");

        let (code, page) = request(port, &format!("GET /?bienvenue HTTP/1.1\r\n{host}"), "");
        assert_eq!(code, 200);
        assert!(page.contains("<html"));
        // DNS rebinding : autre nom d'hôte refusé.
        assert_eq!(request(port, "GET / HTTP/1.1\r\nHost: attaquant.example:80", "").0, 403);
        // API sans jeton, ou depuis une autre origine : refusée.
        assert_eq!(request(port, &format!("GET /api/config HTTP/1.1\r\n{host}"), "").0, 403);
        let auth = format!("{host}\r\nX-Bascule-Token: {token}");
        let evil = format!("POST /api/config HTTP/1.1\r\n{auth}\r\nOrigin: http://attaquant.example");
        assert_eq!(request(port, &evil, "{}").0, 403);

        let (code, body) = request(port, &format!("GET /api/config HTTP/1.1\r\n{auth}"), "");
        assert_eq!(code, 200);
        let info: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(info["config"]["network"]["secret"], "abcd-efgh-jkmn-pqrs");
        assert_eq!(info["config"]["hotkeys"]["switch_to_secondary"], "Shift+F12");
        assert_eq!(info["running"], false);

        // Enregistrement.
        let mut cfg = info["config"].clone();
        cfg["network"]["connect"] = json!("192.168.1.30");
        cfg["screen"]["edge_switching"] = json!(false);
        let post = format!("POST /api/config HTTP/1.1\r\n{auth}\r\nOrigin: http://127.0.0.1:{port}");
        let (code, body) = request(port, &post, &cfg.to_string());
        assert_eq!(code, 200, "{body}");
        let text = std::fs::read_to_string(&tmp.0).unwrap();
        assert!(text.contains("connect = \"192.168.1.30\"\n"));
        assert!(text.contains("edge_switching = false\n"));
        assert!(text.contains("# Secret partagé"));

        // Réglage invalide : refusé avec un message, fichier inchangé.
        cfg["mouse"]["speed"] = json!(100);
        let (code, body) = request(port, &post, &cfg.to_string());
        assert_eq!(code, 422);
        assert!(body.contains("speed"), "{body}");
        assert_eq!(std::fs::read_to_string(&tmp.0).unwrap(), text);
        // Champ inconnu : refusé.
        cfg["mouse"]["speed"] = json!(1.0);
        cfg["mouse"]["vitesse"] = json!(2);
        assert_eq!(request(port, &post, &cfg.to_string()).0, 422);
    }
}
