//! Synchronisation du presse-papiers (texte, HTML, images) dans les deux sens.
//!
//! Un thread dédié surveille le presse-papiers local (compteur de changements
//! du système, interrogé toutes les 250 ms) et envoie chaque nouveau contenu à
//! l'autre machine, découpé en morceaux. Le contenu reçu est écrit dans le
//! presse-papiers local ; le changement qui en résulte n'est pas renvoyé.

use crate::config::ClipboardCfg;
use crate::net::LinkHandle;
use crate::protocol::{CLIP_CHUNK, ClipPayload, Message};
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(250);

/// Accès au presse-papiers du système.
pub trait Backend {
    /// Compteur qui change à chaque modification du presse-papiers.
    fn sequence(&mut self) -> Option<u64>;
    /// Le contenu actuel est marqué « sensible » (gestionnaire de mots de passe…).
    fn is_sensitive(&mut self) -> bool;
    fn read(&mut self, images: bool, html: bool) -> Option<ClipPayload>;
    fn write(&mut self, p: &ClipPayload) -> bool;
}

/// Messages pour le thread du presse-papiers.
#[derive(Debug)]
pub enum ClipEvent {
    Connected,
    Disconnected,
    Start { id: u32, total: u32 },
    Chunk { id: u32, data: Vec<u8> },
}

#[derive(Clone)]
pub struct ClipboardHandle {
    tx: Sender<ClipEvent>,
}

impl ClipboardHandle {
    pub fn notify(&self, ev: ClipEvent) {
        let _ = self.tx.send(ev);
    }
}

struct Incoming {
    id: u32,
    total: usize,
    data: Vec<u8>,
}

/// Empreinte rapide d'un contenu (FNV-1a 64 bits) pour éviter les renvois.
fn fingerprint(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^ (bytes.len() as u64)
}

pub struct Sync<B: Backend> {
    backend: B,
    cfg: ClipboardCfg,
    link: LinkHandle,
    connected: bool,
    last_seq: Option<u64>,
    /// Empreinte du dernier contenu envoyé ou reçu.
    last_fp: Option<u64>,
    incoming: Option<Incoming>,
    next_id: u32,
    /// Lecture à réessayer (presse-papiers occupé par une autre application).
    retry_at: Option<Instant>,
}

impl<B: Backend> Sync<B> {
    pub fn new(backend: B, cfg: ClipboardCfg, link: LinkHandle) -> Self {
        let mut s = Sync {
            backend,
            cfg,
            link,
            connected: false,
            last_seq: None,
            last_fp: None,
            incoming: None,
            next_id: 1,
            retry_at: None,
        };
        // Le contenu présent au démarrage n'est pas envoyé.
        s.last_seq = s.backend.sequence();
        s
    }

    fn max_bytes(&self) -> usize {
        (self.cfg.max_size_mb.max(1) as usize) * 1024 * 1024
    }

    pub fn handle(&mut self, ev: ClipEvent) {
        match ev {
            ClipEvent::Connected => {
                self.connected = true;
                self.incoming = None;
            }
            ClipEvent::Disconnected => {
                self.connected = false;
                self.incoming = None;
            }
            ClipEvent::Start { id, total } => {
                let total = total as usize;
                if total > self.max_bytes() + 1024 {
                    log::warn!("presse-papiers reçu trop volumineux ({total} octets) : ignoré");
                    self.incoming = None;
                    return;
                }
                self.incoming = Some(Incoming { id, total, data: Vec::with_capacity(total) });
                if total == 0 {
                    self.finish_incoming();
                }
            }
            ClipEvent::Chunk { id, data } => {
                let Some(inc) = self.incoming.as_mut() else { return };
                if inc.id != id {
                    return;
                }
                inc.data.extend_from_slice(&data);
                if inc.data.len() > inc.total {
                    log::warn!("presse-papiers reçu incohérent : ignoré");
                    self.incoming = None;
                } else if inc.data.len() == inc.total {
                    self.finish_incoming();
                }
            }
        }
    }

    fn finish_incoming(&mut self) {
        let Some(inc) = self.incoming.take() else { return };
        let fp = fingerprint(&inc.data);
        let payload = match ClipPayload::decode(&inc.data) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("presse-papiers reçu illisible : {e}");
                return;
            }
        };
        if payload.is_empty() {
            return;
        }
        if self.backend.write(&payload) {
            self.last_fp = Some(fp);
            // Le changement que l'on vient de provoquer ne doit pas être renvoyé.
            self.last_seq = self.backend.sequence();
            log::info!("presse-papiers reçu ({})", describe(&payload));
        } else {
            log::warn!("impossible d'écrire le presse-papiers local");
        }
    }

    /// Vérifie le presse-papiers local et envoie son contenu s'il a changé.
    pub fn poll(&mut self) {
        let seq = self.backend.sequence();
        let changed = seq != self.last_seq;
        let retry_due = self.retry_at.is_some_and(|t| Instant::now() >= t);
        if !changed && !retry_due {
            return;
        }
        self.retry_at = None;
        if !self.connected {
            // Hors connexion : on ne mémorise pas, pour envoyer le dernier
            // contenu copié dès la connexion… sauf s'il date d'avant le démarrage.
            return;
        }
        if self.backend.is_sensitive() {
            self.last_seq = seq;
            if self.cfg.skip_sensitive {
                log::info!("contenu sensible dans le presse-papiers : non transmis");
                return;
            }
        }
        let Some(payload) = self.backend.read(self.cfg.images, self.cfg.html) else {
            // Presse-papiers vide, format non géré, ou occupé : réessai une fois.
            if changed {
                self.retry_at = Some(Instant::now() + Duration::from_millis(500));
            }
            self.last_seq = seq;
            return;
        };
        self.last_seq = seq;
        if payload.is_empty() {
            return;
        }
        let bytes = payload.encode();
        if bytes.len() > self.max_bytes() {
            log::warn!(
                "presse-papiers trop volumineux ({} Mo > {} Mo) : non transmis",
                bytes.len() / (1024 * 1024),
                self.cfg.max_size_mb
            );
            return;
        }
        let fp = fingerprint(&bytes);
        if self.last_fp == Some(fp) {
            return;
        }
        self.last_fp = Some(fp);
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        log::info!("envoi du presse-papiers ({})", describe(&payload));
        if !self.link.send_bulk(Message::ClipStart { id, total: bytes.len() as u32 }) {
            return;
        }
        for chunk in bytes.chunks(CLIP_CHUNK) {
            if !self.link.send_bulk(Message::ClipChunk { id, data: chunk.to_vec() }) {
                log::warn!("envoi du presse-papiers interrompu");
                return;
            }
        }
    }
}

fn describe(p: &ClipPayload) -> String {
    let mut parts = vec![];
    if let Some(t) = &p.text {
        parts.push(format!("texte {} car.", t.chars().count()));
    }
    if p.html.is_some() {
        parts.push("HTML".to_string());
    }
    if let Some(png) = &p.png {
        parts.push(format!("image {} Ko", png.len() / 1024));
    }
    parts.join(", ")
}

/// Démarre la synchronisation dans un thread dédié.
pub fn start<B: Backend + 'static>(
    make_backend: impl FnOnce() -> Option<B> + Send + 'static,
    cfg: ClipboardCfg,
    link: LinkHandle,
) -> ClipboardHandle {
    let (tx, rx) = unbounded::<ClipEvent>();
    let enabled = cfg.enabled;
    std::thread::Builder::new()
        .name("bascule-clipboard".into())
        .spawn(move || {
            if !enabled {
                log::info!("presse-papiers partagé désactivé");
                for _ in rx {}
                return;
            }
            let Some(backend) = make_backend() else {
                log::error!("presse-papiers inaccessible : partage désactivé");
                for _ in rx {}
                return;
            };
            run(Sync::new(backend, cfg, link), rx);
        })
        .expect("thread presse-papiers");
    ClipboardHandle { tx }
}

fn run<B: Backend>(mut sync: Sync<B>, rx: Receiver<ClipEvent>) {
    loop {
        select! {
            recv(rx) -> ev => match ev {
                Ok(ev) => sync.handle(ev),
                Err(_) => return,
            },
            default(POLL) => sync.poll(),
        }
    }
}

// ---------------------------------------------------------------------------
// Implémentation réelle (arboard + fonctions de la plateforme)
// ---------------------------------------------------------------------------

pub struct SystemBackend {
    cb: arboard::Clipboard,
    seq: fn() -> Option<u64>,
    sensitive: fn() -> bool,
}

impl SystemBackend {
    pub fn new(seq: fn() -> Option<u64>, sensitive: fn() -> bool) -> Option<SystemBackend> {
        match arboard::Clipboard::new() {
            Ok(cb) => Some(SystemBackend { cb, seq, sensitive }),
            Err(e) => {
                log::error!("presse-papiers : {e}");
                None
            }
        }
    }
}

impl Backend for SystemBackend {
    fn sequence(&mut self) -> Option<u64> {
        (self.seq)()
    }

    fn is_sensitive(&mut self) -> bool {
        (self.sensitive)()
    }

    fn read(&mut self, images: bool, html: bool) -> Option<ClipPayload> {
        let text = self.cb.get_text().ok().filter(|t| !t.is_empty());
        let html =
            if html && text.is_some() { self.cb.get().html().ok().filter(|h| !h.trim().is_empty()) } else { None };
        let png = if images && text.is_none() {
            match self.cb.get_image() {
                Ok(img) => encode_png(img.width as u32, img.height as u32, &img.bytes),
                Err(_) => None,
            }
        } else {
            None
        };
        if text.is_none() && png.is_none() {
            return None;
        }
        Some(ClipPayload { text, html, png })
    }

    fn write(&mut self, p: &ClipPayload) -> bool {
        if let Some(text) = &p.text {
            let r = match &p.html {
                Some(h) => self.cb.set_html(h.as_str(), Some(text.as_str())),
                None => self.cb.set_text(text.as_str()),
            };
            return r.map_err(|e| log::warn!("écriture du presse-papiers : {e}")).is_ok();
        }
        if let Some(png) = &p.png {
            let Some((w, h, rgba)) = decode_png(png) else {
                log::warn!("image reçue illisible");
                return false;
            };
            let img = arboard::ImageData { width: w as usize, height: h as usize, bytes: rgba.into() };
            return self.cb.set_image(img).map_err(|e| log::warn!("écriture de l'image : {e}")).is_ok();
        }
        false
    }
}

pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 || rgba.len() != (width as usize) * (height as usize) * 4 {
        return None;
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        let mut w = enc.write_header().ok()?;
        w.write_image_data(rgba).ok()?;
    }
    Some(out)
}

pub fn decode_png(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut dec = png::Decoder::new(std::io::Cursor::new(data));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    let (w, h) = (info.width, info.height);
    let px = (w as usize) * (h as usize);
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf.chunks_exact(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|g| [*g, *g, *g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    if rgba.len() != px * 4 {
        return None;
    }
    Some((w, h, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct State {
        seq: u64,
        content: Option<ClipPayload>,
        sensitive: bool,
    }

    #[derive(Clone, Default)]
    struct Fake(Arc<Mutex<State>>);

    impl Fake {
        fn user_copies(&self, text: &str) {
            let mut s = self.0.lock().unwrap();
            s.seq += 1;
            s.content = Some(ClipPayload { text: Some(text.into()), ..Default::default() });
        }
    }

    impl Backend for Fake {
        fn sequence(&mut self) -> Option<u64> {
            Some(self.0.lock().unwrap().seq)
        }
        fn is_sensitive(&mut self) -> bool {
            self.0.lock().unwrap().sensitive
        }
        fn read(&mut self, _images: bool, _html: bool) -> Option<ClipPayload> {
            self.0.lock().unwrap().content.clone()
        }
        fn write(&mut self, p: &ClipPayload) -> bool {
            let mut s = self.0.lock().unwrap();
            s.seq += 1;
            s.content = Some(p.clone());
            true
        }
    }

    fn sent(rx: &Receiver<Message>) -> Vec<Message> {
        rx.try_iter().collect()
    }

    fn reassemble(msgs: &[Message]) -> ClipPayload {
        let mut data = vec![];
        let mut total = 0;
        for m in msgs {
            match m {
                Message::ClipStart { total: t, .. } => total = *t as usize,
                Message::ClipChunk { data: d, .. } => data.extend_from_slice(d),
                _ => {}
            }
        }
        assert_eq!(data.len(), total);
        ClipPayload::decode(&data).unwrap()
    }

    #[test]
    fn sends_local_changes_once() {
        let fake = Fake::default();
        let (link, rx) = LinkHandle::detached_bulk();
        let mut s = Sync::new(fake.clone(), ClipboardCfg::default(), link);
        s.handle(ClipEvent::Connected);
        s.poll();
        assert!(sent(&rx).is_empty());
        fake.user_copies("bonjour");
        s.poll();
        let msgs = sent(&rx);
        assert_eq!(reassemble(&msgs).text.as_deref(), Some("bonjour"));
        s.poll();
        assert!(sent(&rx).is_empty());
    }

    #[test]
    fn received_content_is_not_echoed() {
        let fake = Fake::default();
        let (link, rx) = LinkHandle::detached_bulk();
        let mut s = Sync::new(fake.clone(), ClipboardCfg::default(), link);
        s.handle(ClipEvent::Connected);
        let p = ClipPayload { text: Some("x".repeat(200_000)), ..Default::default() };
        let bytes = p.encode();
        s.handle(ClipEvent::Start { id: 9, total: bytes.len() as u32 });
        for c in bytes.chunks(CLIP_CHUNK) {
            s.handle(ClipEvent::Chunk { id: 9, data: c.to_vec() });
        }
        assert_eq!(fake.0.lock().unwrap().content.as_ref().unwrap().text.as_ref().unwrap().len(), 200_000);
        s.poll();
        assert!(sent(&rx).is_empty());
    }

    #[test]
    fn sensitive_content_is_skipped() {
        let fake = Fake::default();
        let (link, rx) = LinkHandle::detached_bulk();
        let mut s = Sync::new(fake.clone(), ClipboardCfg::default(), link);
        s.handle(ClipEvent::Connected);
        fake.0.lock().unwrap().sensitive = true;
        fake.user_copies("mot-de-passe");
        s.poll();
        assert!(sent(&rx).is_empty());
    }

    #[test]
    fn too_big_is_skipped() {
        let fake = Fake::default();
        let (link, rx) = LinkHandle::detached_bulk();
        let cfg = ClipboardCfg { max_size_mb: 1, ..Default::default() };
        let mut s = Sync::new(fake.clone(), cfg, link);
        s.handle(ClipEvent::Connected);
        fake.user_copies(&"y".repeat(2 * 1024 * 1024));
        s.poll();
        assert!(sent(&rx).is_empty());
    }

    #[test]
    fn png_roundtrip() {
        let (w, h) = (3u32, 2u32);
        let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i * 7) as u8).collect();
        let png = encode_png(w, h, &rgba).unwrap();
        assert_eq!(decode_png(&png).unwrap(), (w, h, rgba));
    }
}
