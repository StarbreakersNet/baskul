//! Presse-papiers partagé entre toutes les machines : texte, HTML, images
//! (dans leur format d'origine) et fichiers.
//!
//! Un thread dédié surveille le presse-papiers local (compteur de changements
//! du système, interrogé toutes les 250 ms) et envoie chaque nouveau contenu
//! aux autres machines : un en-tête (texte, HTML, images, liste des fichiers)
//! puis le contenu des fichiers, lu au fil de l'envoi, découpé en morceaux.
//! Un nouveau copier interrompt l'envoi en cours.
//!
//! Le contenu reçu est écrit dans le presse-papiers local (les fichiers dans
//! un dossier temporaire, d'où on les colle) ; le changement qui en résulte
//! n'est pas renvoyé. La machine principale relaie ce qu'elle reçoit d'une
//! pilotée aux autres pilotées.

use crate::app::ClipboardHooks;
use crate::config::ClipboardCfg;
use crate::net::{LinkHandle, PeerId};
use crate::protocol::{CLIP_CHUNK, ClipFile, ClipHeader, ClipImage, ImageFormat, Message};
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(250);
/// Transferts reçus gardés sur le disque (on peut encore les coller).
const KEPT_TRANSFERS: usize = 3;

/// Ce qu'on accepte de lire et d'envoyer.
#[derive(Debug, Clone)]
pub struct ReadOptions {
    pub images: bool,
    pub html: bool,
    pub files: bool,
    /// Taille maximale des fichiers (octets).
    pub max_files: u64,
}

/// Contenu du presse-papiers local, prêt à être envoyé.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Content {
    pub header: ClipHeader,
    /// Fichier local de chaque entrée de `header.files` (`None` : dossier).
    pub sources: Vec<Option<PathBuf>>,
}

/// Accès au presse-papiers du système.
pub trait Backend {
    /// Compteur qui change à chaque modification du presse-papiers.
    fn sequence(&mut self) -> Option<u64>;
    /// Le contenu actuel est marqué « sensible » (gestionnaire de mots de passe…).
    fn is_sensitive(&mut self) -> bool;
    fn read(&mut self, opts: &ReadOptions) -> Result<Content, Unread>;
    /// Écrit un contenu reçu ; `roots` : fichiers et dossiers de premier niveau,
    /// déjà sur le disque.
    fn write(&mut self, header: &ClipHeader, roots: &[PathBuf]) -> bool;
}

/// Pourquoi le presse-papiers local n'a pas été lu.
#[derive(Debug, PartialEq)]
pub enum Unread {
    /// Vide, ou occupé par une autre application.
    Empty,
    /// Contenu présent mais non transmissible ; la raison va au journal.
    Unsupported(String),
}

/// Messages pour le thread du presse-papiers.
#[derive(Debug)]
pub enum ClipEvent {
    Connected { peer: PeerId },
    Disconnected { peer: PeerId },
    Start { peer: PeerId, id: u32, header: u32, total: u64 },
    Chunk { peer: PeerId, id: u32, data: Vec<u8> },
    Cancel { peer: PeerId, id: u32 },
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

/// Empreinte rapide d'un contenu (FNV-1a 64 bits) pour éviter les renvois.
fn fingerprint(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^ (bytes.len() as u64)
}

/// Chemin relatif sûr (pas de « .. », pas de chemin absolu ni de lecteur).
fn safe_relative(path: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for c in path.split('/') {
        if c.is_empty() || c == "." || c == ".." || c.contains(['\\', ':', '\0']) {
            return None;
        }
        out.push(c);
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

// ---------------------------------------------------------------------------
// Dossier des fichiers reçus
// ---------------------------------------------------------------------------

/// Fichiers reçus : un dossier par transfert, les plus anciens supprimés.
struct Staging {
    base: PathBuf,
    next: u64,
    dirs: Vec<PathBuf>,
}

impl Staging {
    fn new(base: PathBuf) -> Staging {
        // Restes d'une exécution précédente.
        let _ = std::fs::remove_dir_all(&base);
        Staging { base, next: 1, dirs: Vec::new() }
    }

    fn new_dir(&mut self) -> io::Result<PathBuf> {
        while self.dirs.len() >= KEPT_TRANSFERS {
            let old = self.dirs.remove(0);
            let _ = std::fs::remove_dir_all(old);
        }
        let dir = self.base.join(self.next.to_string());
        self.next += 1;
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)?;
        self.dirs.push(dir.clone());
        Ok(dir)
    }

    fn discard(&mut self, dir: &Path) {
        self.dirs.retain(|d| d != dir);
        let _ = std::fs::remove_dir_all(dir);
    }
}

fn staging_base() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("Baskul").join("clipboard")
}

// ---------------------------------------------------------------------------
// Réception
// ---------------------------------------------------------------------------

/// Écriture des fichiers d'un transfert, dans l'ordre de l'en-tête.
struct FileSink {
    dir: PathBuf,
    /// Fichiers (chemin complet, taille), dans l'ordre.
    files: Vec<(PathBuf, u64)>,
    idx: usize,
    cur: Option<(File, u64)>,
}

impl FileSink {
    fn new(dir: PathBuf, header: &ClipHeader) -> Result<FileSink, String> {
        let mut files = Vec::new();
        for f in &header.files {
            let rel = safe_relative(&f.path).ok_or_else(|| format!("chemin refusé : {}", f.path))?;
            let full = dir.join(rel);
            match f.size {
                None => std::fs::create_dir_all(&full).map_err(|e| e.to_string())?,
                Some(size) => files.push((full, size)),
            }
        }
        let mut sink = FileSink { dir, files, idx: 0, cur: None };
        sink.open_next().map_err(|e| e.to_string())?;
        Ok(sink)
    }

    /// Ouvre le fichier suivant (les fichiers vides sont créés aussitôt).
    fn open_next(&mut self) -> io::Result<()> {
        while self.cur.is_none() && self.idx < self.files.len() {
            let (path, size) = &self.files[self.idx];
            self.idx += 1;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let f = File::create(path)?;
            if *size > 0 {
                self.cur = Some((f, *size));
            }
        }
        Ok(())
    }

    fn write(&mut self, mut data: &[u8]) -> io::Result<()> {
        while !data.is_empty() {
            let Some((f, remaining)) = self.cur.as_mut() else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "données en trop"));
            };
            let n = (*remaining).min(data.len() as u64) as usize;
            f.write_all(&data[..n])?;
            *remaining -= n as u64;
            data = &data[n..];
            if *remaining == 0 {
                self.cur = None;
                self.open_next()?;
            }
        }
        Ok(())
    }
}

struct Incoming {
    id: u32,
    header_len: usize,
    total: u64,
    received: u64,
    header_buf: Vec<u8>,
    header: Option<ClipHeader>,
    files: Option<FileSink>,
    /// Transfert ignoré (trop volumineux, illisible…) : on laisse passer la fin.
    discard: bool,
}

// ---------------------------------------------------------------------------
// Envoi (thread dédié, interrompu par un nouveau copier)
// ---------------------------------------------------------------------------

struct Job {
    generation: u64,
    header: Vec<u8>,
    /// Fichiers à lire (chemin et taille annoncée), dans l'ordre.
    files: Vec<(PathBuf, u64)>,
    to: Vec<PeerId>,
}

/// Lecture des octets à envoyer : l'en-tête, puis chaque fichier, ramené à
/// sa taille annoncée (un fichier qui a changé entre-temps est complété ou
/// coupé).
struct Stream {
    header: Vec<u8>,
    pos: usize,
    files: std::vec::IntoIter<(PathBuf, u64)>,
    cur: Option<(File, u64)>,
}

impl Stream {
    fn fill(&mut self, buf: &mut Vec<u8>, max: usize) -> io::Result<()> {
        buf.clear();
        while buf.len() < max {
            if self.pos < self.header.len() {
                let n = (max - buf.len()).min(self.header.len() - self.pos);
                buf.extend_from_slice(&self.header[self.pos..self.pos + n]);
                self.pos += n;
                continue;
            }
            if self.cur.is_none() {
                match self.files.next() {
                    Some((path, size)) => self.cur = Some((File::open(&path)?, size)),
                    None => return Ok(()),
                }
            }
            let (f, remaining) = self.cur.as_mut().expect("fichier ouvert");
            if *remaining == 0 {
                self.cur = None;
                continue;
            }
            let want = (max - buf.len()).min(*remaining as usize);
            let start = buf.len();
            buf.resize(start + want, 0);
            let n = f.read(&mut buf[start..])?;
            if n > 0 {
                buf.truncate(start + n);
            }
            // n == 0 : fichier raccourci entre-temps, complété par des zéros.
            *remaining -= (buf.len() - start) as u64;
        }
        Ok(())
    }
}

fn sender_loop(rx: Receiver<Job>, link: LinkHandle, generation: Arc<AtomicU64>) {
    let mut next_id: u32 = 1;
    let mut buf = Vec::with_capacity(CLIP_CHUNK);
    for job in rx {
        if job.generation != generation.load(Ordering::Acquire) {
            continue;
        }
        let id = next_id;
        next_id = next_id.wrapping_add(1);
        let total = job.header.len() as u64 + job.files.iter().map(|(_, s)| s).sum::<u64>();
        let header = job.header.len() as u32;
        let mut to: Vec<PeerId> =
            job.to.into_iter().filter(|p| link.send_bulk(*p, Message::ClipStart { id, header, total })).collect();
        let mut stream = Stream { header: job.header, pos: 0, files: job.files.into_iter(), cur: None };
        let mut sent = 0u64;
        while sent < total && !to.is_empty() {
            let cancelled = job.generation != generation.load(Ordering::Acquire);
            if cancelled || stream.fill(&mut buf, CLIP_CHUNK).is_err() || buf.is_empty() {
                if !cancelled {
                    log::warn!("envoi du presse-papiers interrompu : fichier illisible");
                }
                for p in &to {
                    link.send_bulk(*p, Message::ClipCancel { id });
                }
                break;
            }
            sent += buf.len() as u64;
            to.retain(|p| link.send_bulk(*p, Message::ClipChunk { id, data: buf.clone() }));
        }
    }
}

// ---------------------------------------------------------------------------
// Synchronisation
// ---------------------------------------------------------------------------

pub struct Sync<B: Backend> {
    backend: B,
    cfg: ClipboardCfg,
    peers: Vec<PeerId>,
    last_seq: Option<u64>,
    /// Empreinte du dernier en-tête envoyé ou reçu.
    last_fp: Option<u64>,
    incoming: HashMap<PeerId, Incoming>,
    /// Lecture à réessayer (presse-papiers occupé par une autre application).
    retry_at: Option<Instant>,
    staging: Staging,
    jobs: Sender<Job>,
    generation: Arc<AtomicU64>,
}

impl<B: Backend> Sync<B> {
    pub fn new(backend: B, cfg: ClipboardCfg, link: LinkHandle) -> Self {
        Self::with_staging(backend, cfg, link, staging_base())
    }

    fn with_staging(backend: B, cfg: ClipboardCfg, link: LinkHandle, base: PathBuf) -> Self {
        let (jobs, rx) = unbounded::<Job>();
        let generation = Arc::new(AtomicU64::new(0));
        let g = generation.clone();
        std::thread::Builder::new()
            .name("baskul-clipboard-tx".into())
            .spawn(move || sender_loop(rx, link, g))
            .expect("thread d'envoi du presse-papiers");
        let mut s = Sync {
            backend,
            cfg,
            peers: Vec::new(),
            last_seq: None,
            last_fp: None,
            incoming: HashMap::new(),
            retry_at: None,
            staging: Staging::new(base),
            jobs,
            generation,
        };
        // Le contenu présent au démarrage n'est pas envoyé.
        s.last_seq = s.backend.sequence();
        s
    }

    fn max_bytes(&self) -> usize {
        (self.cfg.max_size_mb.max(1) as usize) * 1024 * 1024
    }

    fn max_files(&self) -> u64 {
        u64::from(self.cfg.max_files_mb.max(1)) * 1024 * 1024
    }

    pub fn handle(&mut self, ev: ClipEvent) {
        match ev {
            ClipEvent::Connected { peer } => {
                if !self.peers.contains(&peer) {
                    self.peers.push(peer);
                }
            }
            ClipEvent::Disconnected { peer } => {
                self.peers.retain(|p| *p != peer);
                self.drop_incoming(peer);
            }
            ClipEvent::Start { peer, id, header, total } => self.start_incoming(peer, id, header as usize, total),
            ClipEvent::Chunk { peer, id, data } => self.chunk(peer, id, &data),
            ClipEvent::Cancel { peer, id } => {
                if self.incoming.get(&peer).is_some_and(|i| i.id == id) {
                    log::info!("presse-papiers : envoi abandonné par l'autre machine");
                    self.drop_incoming(peer);
                }
            }
        }
    }

    fn drop_incoming(&mut self, peer: PeerId) {
        if let Some(inc) = self.incoming.remove(&peer) {
            if let Some(f) = inc.files {
                self.staging.discard(&f.dir);
            }
        }
    }

    fn start_incoming(&mut self, peer: PeerId, id: u32, header_len: usize, total: u64) {
        self.drop_incoming(peer);
        let files = total.saturating_sub(header_len as u64);
        let discard = if header_len > self.max_bytes() + 1024 || header_len as u64 > total {
            log::warn!("presse-papiers reçu trop volumineux ({header_len} octets) : ignoré");
            true
        } else if files > 0 && !self.cfg.files {
            log::info!("fichiers reçus ignorés : copie de fichiers désactivée");
            true
        } else if files > self.max_files() {
            log::warn!("fichiers reçus trop volumineux ({} Mo) : ignorés", files >> 20);
            true
        } else {
            false
        };
        let inc = Incoming {
            id,
            header_len,
            total,
            received: 0,
            header_buf: Vec::with_capacity(if discard { 0 } else { header_len }),
            header: None,
            files: None,
            discard,
        };
        self.incoming.insert(peer, inc);
        if total == 0 {
            self.finish_incoming(peer);
        }
    }

    fn chunk(&mut self, peer: PeerId, id: u32, mut data: &[u8]) {
        let Some(inc) = self.incoming.get_mut(&peer).filter(|i| i.id == id) else { return };
        inc.received += data.len() as u64;
        if inc.received > inc.total {
            log::warn!("presse-papiers reçu incohérent : ignoré");
            self.drop_incoming(peer);
            return;
        }
        if !inc.discard {
            if inc.header.is_none() {
                let n = (inc.header_len - inc.header_buf.len()).min(data.len());
                inc.header_buf.extend_from_slice(&data[..n]);
                data = &data[n..];
                if inc.header_buf.len() == inc.header_len {
                    match ClipHeader::decode(&inc.header_buf) {
                        Ok(h) => inc.header = Some(h),
                        Err(e) => {
                            log::warn!("presse-papiers reçu illisible : {e}");
                            inc.discard = true;
                        }
                    }
                    if let Some(h) = inc.header.as_ref().filter(|h| !h.files.is_empty()) {
                        let sink = self.staging.new_dir().map_err(|e| e.to_string()).and_then(|d| FileSink::new(d, h));
                        match sink {
                            Ok(s) => inc.files = Some(s),
                            Err(e) => {
                                log::warn!("fichiers reçus : {e}");
                                inc.discard = true;
                            }
                        }
                    }
                }
            }
            if !data.is_empty() && !inc.discard {
                let written = inc.files.as_mut().map(|f| f.write(data));
                if !matches!(written, Some(Ok(()))) {
                    log::warn!("fichiers reçus : écriture impossible ({:?})", written.and_then(Result::err));
                    inc.discard = true;
                }
            }
        }
        if inc.received == inc.total {
            self.finish_incoming(peer);
        }
    }

    fn finish_incoming(&mut self, peer: PeerId) {
        let Some(inc) = self.incoming.remove(&peer) else { return };
        if inc.discard {
            if let Some(f) = inc.files {
                self.staging.discard(&f.dir);
            }
            return;
        }
        let Some(header) = inc.header.filter(|h| !h.is_empty()) else { return };
        let fp = fingerprint(&inc.header_buf);
        let dir = inc.files.as_ref().map(|f| f.dir.clone());
        let sources: Vec<Option<PathBuf>> = header
            .files
            .iter()
            .map(|f| f.size.and(dir.as_ref()).and_then(|d| safe_relative(&f.path).map(|r| d.join(r))))
            .collect();
        let roots = dir.as_ref().map(|d| roots_of(d, &header.files)).unwrap_or_default();
        // Une image copiée en tant que fichier se colle aussi comme image.
        let mut local = header.clone();
        if local.images.is_empty() && self.cfg.images {
            if let ([file], [Some(path)]) = (header.files.as_slice(), sources.as_slice()) {
                if file.size.is_some_and(|s| s as usize <= self.max_bytes()) {
                    if let Some(img) = std::fs::read(path).ok().and_then(image_of) {
                        local.images.push(img);
                    }
                }
            }
        }
        if self.backend.write(&local, &roots) {
            self.last_fp = Some(fp);
            // Le changement que l'on vient de provoquer ne doit pas être renvoyé.
            self.last_seq = self.backend.sequence();
            log::info!("presse-papiers reçu ({})", describe(&header));
        } else {
            log::warn!("impossible d'écrire le presse-papiers local");
        }
        // Machine principale : les autres pilotées le reçoivent aussi.
        let others: Vec<PeerId> = self.peers.iter().copied().filter(|p| *p != peer).collect();
        if !others.is_empty() {
            self.queue(Content { header, sources }, inc.header_buf, others);
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
        if self.peers.is_empty() {
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
        let opts = ReadOptions {
            images: self.cfg.images,
            html: self.cfg.html,
            files: self.cfg.files,
            max_files: self.max_files(),
        };
        let content = match self.backend.read(&opts) {
            Ok(c) => c,
            Err(why) => {
                // Presse-papiers occupé, ou contenu pas encore disponible :
                // réessai une fois, puis explication dans le journal.
                if changed {
                    self.retry_at = Some(Instant::now() + Duration::from_millis(500));
                } else if let Unread::Unsupported(why) = why {
                    log::info!("presse-papiers non transmis : {why}");
                }
                self.last_seq = seq;
                return;
            }
        };
        self.last_seq = seq;
        if content.header.is_empty() {
            return;
        }
        let bytes = content.header.encode();
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
        log::info!("envoi du presse-papiers ({})", describe(&content.header));
        let to = self.peers.clone();
        self.queue(content, bytes, to);
    }

    /// Confie un envoi au thread d'envoi ; un envoi plus ancien encore en
    /// cours est abandonné.
    fn queue(&mut self, content: Content, header: Vec<u8>, to: Vec<PeerId>) {
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let files =
            content.header.files.iter().zip(content.sources).filter_map(|(f, src)| Some((src?, f.size?))).collect();
        let _ = self.jobs.send(Job { generation, header, files, to });
    }
}

/// Éléments de premier niveau d'un transfert (premier composant de chaque
/// chemin, sans doublon).
fn roots_of(dir: &Path, files: &[ClipFile]) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for f in files {
        if let Some(first) = f.path.split('/').next().and_then(safe_relative) {
            let p = dir.join(first);
            if !roots.contains(&p) {
                roots.push(p);
            }
        }
    }
    roots
}

/// Image d'après le contenu d'un fichier.
fn image_of(data: Vec<u8>) -> Option<ClipImage> {
    ImageFormat::sniff(&data).map(|format| ClipImage { format, data })
}

fn describe(h: &ClipHeader) -> String {
    let mut parts = vec![];
    if let Some(t) = &h.text {
        parts.push(format!("texte {} car.", t.chars().count()));
    }
    if h.html.is_some() {
        parts.push("HTML".to_string());
    }
    if let Some(img) = h.images.first() {
        parts.push(format!("image {:?} {} Ko", img.format, img.data.len() / 1024));
    }
    if !h.files.is_empty() {
        let n = h.files.iter().filter(|f| f.size.is_some()).count();
        parts.push(format!("{n} fichier(s), {} Ko", h.files_size() / 1024));
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
        .name("baskul-clipboard".into())
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
// Lecture des fichiers copiés
// ---------------------------------------------------------------------------

/// Liste des fichiers et dossiers copiés (dossiers parcourus), avec la taille
/// de chaque fichier. Les liens symboliques sont ignorés.
pub fn collect_files(roots: &[PathBuf], max: u64) -> Result<(Vec<ClipFile>, Vec<Option<PathBuf>>), String> {
    struct Walk {
        files: Vec<ClipFile>,
        sources: Vec<Option<PathBuf>>,
        total: u64,
        max: u64,
    }
    impl Walk {
        fn visit(&mut self, path: &Path, rel: &str) -> Result<(), String> {
            let meta = std::fs::symlink_metadata(path).map_err(|e| format!("{rel} : {e}"))?;
            if meta.file_type().is_symlink() {
                log::debug!("lien symbolique ignoré : {}", path.display());
                return Ok(());
            }
            if meta.is_dir() {
                self.files.push(ClipFile { path: rel.to_string(), size: None });
                self.sources.push(None);
                let mut entries: Vec<_> =
                    std::fs::read_dir(path).map_err(|e| format!("{rel} : {e}"))?.filter_map(Result::ok).collect();
                entries.sort_by_key(|e| e.file_name());
                for e in entries {
                    let name = e.file_name().to_string_lossy().into_owned();
                    self.visit(&e.path(), &format!("{rel}/{name}"))?;
                }
            } else {
                self.total += meta.len();
                if self.total > self.max {
                    return Err(format!("fichiers trop volumineux (plus de {} Mo)", self.max >> 20));
                }
                self.files.push(ClipFile { path: rel.to_string(), size: Some(meta.len()) });
                self.sources.push(Some(path.to_path_buf()));
            }
            Ok(())
        }
    }
    let mut w = Walk { files: Vec::new(), sources: Vec::new(), total: 0, max };
    for root in roots {
        let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).filter(|n| safe_relative(n).is_some());
        if let Some(name) = name {
            w.visit(root, &name)?;
        }
    }
    Ok((w.files, w.sources))
}

// ---------------------------------------------------------------------------
// Implémentation réelle (arboard + fonctions de la plateforme)
// ---------------------------------------------------------------------------

pub struct SystemBackend {
    cb: arboard::Clipboard,
    hooks: ClipboardHooks,
}

impl SystemBackend {
    pub fn new(hooks: ClipboardHooks) -> Option<SystemBackend> {
        match arboard::Clipboard::new() {
            Ok(cb) => Some(SystemBackend { cb, hooks }),
            Err(e) => {
                log::error!("presse-papiers : {e}");
                None
            }
        }
    }

    /// Image copiée : formats d'origine lus par la plateforme ; à défaut,
    /// l'image d'arboard en PNG.
    fn read_images(&mut self) -> Result<Vec<ClipImage>, String> {
        let images = (self.hooks.read_images)();
        if !images.is_empty() {
            return Ok(images);
        }
        match self.cb.get_image() {
            Ok(img) => Ok(encode_png(img.width as u32, img.height as u32, &img.bytes)
                .map(|data| vec![ClipImage { format: ImageFormat::Png, data }])
                .unwrap_or_default()),
            Err(arboard::Error::ContentNotAvailable) => Ok(Vec::new()),
            Err(e) => Err(format!("image illisible : {e}")),
        }
    }

    /// Écriture par arboard, quand la plateforme ne s'en charge pas.
    fn write_generic(&mut self, h: &ClipHeader, roots: &[PathBuf]) -> bool {
        let r = if !roots.is_empty() {
            self.cb.set().file_list(roots)
        } else if let Some(text) = &h.text {
            match &h.html {
                Some(html) => self.cb.set_html(html.as_str(), Some(text.as_str())),
                None => self.cb.set_text(text.as_str()),
            }
        } else if let Some((w, hgt, rgba)) = h.images.iter().find_map(|i| decode_image(&i.data)) {
            let img = arboard::ImageData { width: w as usize, height: hgt as usize, bytes: rgba.into() };
            self.cb.set_image(img)
        } else {
            log::warn!("contenu reçu sans format utilisable ici");
            return false;
        };
        r.map_err(|e| log::warn!("écriture du presse-papiers : {e}")).is_ok()
    }
}

impl Backend for SystemBackend {
    fn sequence(&mut self) -> Option<u64> {
        (self.hooks.sequence)().or_else(|| {
            // Pas de compteur fourni par le système (Linux) : empreinte du texte.
            use std::hash::{Hash, Hasher};
            let text = self.cb.get_text().ok()?;
            let mut h = std::collections::hash_map::DefaultHasher::new();
            text.hash(&mut h);
            Some(h.finish())
        })
    }

    fn is_sensitive(&mut self) -> bool {
        (self.hooks.sensitive)()
    }

    fn read(&mut self, opts: &ReadOptions) -> Result<Content, Unread> {
        // En premier : le Finder accompagne un fichier copié de son nom en texte.
        if let Ok(roots) = self.cb.get().file_list() {
            if !roots.is_empty() {
                if !opts.files {
                    return Err(Unread::Unsupported("fichiers copiés : copie de fichiers désactivée".into()));
                }
                let (files, sources) = collect_files(&roots, opts.max_files).map_err(Unread::Unsupported)?;
                if files.is_empty() {
                    return Err(Unread::Unsupported("fichiers copiés : rien de transmissible".into()));
                }
                return Ok(Content { header: ClipHeader { files, ..Default::default() }, sources });
            }
        }
        let text = self.cb.get_text().ok().filter(|t| !t.is_empty());
        if text.is_some() {
            let html = if opts.html { self.cb.get().html().ok().filter(|h| !h.trim().is_empty()) } else { None };
            return Ok(Content { header: ClipHeader { text, html, ..Default::default() }, sources: Vec::new() });
        }
        let mut why = None;
        if opts.images {
            match self.read_images() {
                Ok(images) if !images.is_empty() => {
                    return Ok(Content { header: ClipHeader { images, ..Default::default() }, sources: Vec::new() });
                }
                Ok(_) => {}
                Err(e) => why = Some(e),
            }
        }
        let formats = (self.hooks.formats)();
        if why.is_none() && formats.is_empty() {
            return Err(Unread::Empty);
        }
        let mut why = why.unwrap_or_else(|| "format non pris en charge".to_string());
        if !formats.is_empty() {
            why = format!("{why} ({})", formats.join(", "));
        }
        if !opts.images {
            why.push_str(" ; partage des images désactivé");
        }
        Err(Unread::Unsupported(why))
    }

    fn write(&mut self, header: &ClipHeader, roots: &[PathBuf]) -> bool {
        match (self.hooks.write)(header, roots) {
            Some(ok) => ok,
            None => self.write_generic(header, roots),
        }
    }
}

/// Décode une image (tout format lu par la bibliothèque `image`) en RGBA.
pub fn decode_image(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::load_from_memory(data).ok()?.into_rgba8();
    Some((img.width(), img.height(), img.into_raw()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct State {
        seq: u64,
        content: Option<Content>,
        sensitive: bool,
        /// Contenu présent mais non transmissible.
        unsupported: bool,
        reads: u32,
        written: Vec<(ClipHeader, Vec<PathBuf>)>,
    }

    #[derive(Clone, Default)]
    struct Fake(Arc<Mutex<State>>);

    impl Fake {
        fn user_copies(&self, content: Content) {
            let mut s = self.0.lock().unwrap();
            s.seq += 1;
            s.content = Some(content);
        }
        fn user_copies_text(&self, text: &str) {
            self.user_copies(Content {
                header: ClipHeader { text: Some(text.into()), ..Default::default() },
                sources: vec![],
            });
        }
        fn written(&self) -> Vec<(ClipHeader, Vec<PathBuf>)> {
            self.0.lock().unwrap().written.clone()
        }
    }

    impl Backend for Fake {
        fn sequence(&mut self) -> Option<u64> {
            Some(self.0.lock().unwrap().seq)
        }
        fn is_sensitive(&mut self) -> bool {
            self.0.lock().unwrap().sensitive
        }
        fn read(&mut self, _opts: &ReadOptions) -> Result<Content, Unread> {
            let mut s = self.0.lock().unwrap();
            s.reads += 1;
            if s.unsupported {
                return Err(Unread::Unsupported("format inconnu".into()));
            }
            s.content.clone().ok_or(Unread::Empty)
        }
        fn write(&mut self, header: &ClipHeader, roots: &[PathBuf]) -> bool {
            let mut s = self.0.lock().unwrap();
            s.seq += 1;
            s.written.push((header.clone(), roots.to_vec()));
            true
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("baskul-clip-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn sync(fake: &Fake, name: &str) -> (Sync<Fake>, Receiver<Message>) {
        sync_with(fake, name, ClipboardCfg::default())
    }

    fn sync_with(fake: &Fake, name: &str, cfg: ClipboardCfg) -> (Sync<Fake>, Receiver<Message>) {
        let (link, rx) = LinkHandle::detached_bulk();
        let mut s = Sync::with_staging(fake.clone(), cfg, link, temp_dir(name));
        s.handle(ClipEvent::Connected { peer: 1 });
        (s, rx)
    }

    /// Messages envoyés, jusqu'au contenu complet (ou à son abandon).
    fn transfer(rx: &Receiver<Message>) -> Vec<Message> {
        let mut out = vec![];
        let mut expected = None;
        let mut got = 0u64;
        loop {
            let m = rx.recv_timeout(Duration::from_secs(5)).expect("envoi");
            let done = match &m {
                Message::ClipStart { total, .. } => {
                    expected = Some(*total);
                    *total == 0
                }
                Message::ClipChunk { data, .. } => {
                    got += data.len() as u64;
                    expected == Some(got)
                }
                Message::ClipCancel { .. } => true,
                _ => false,
            };
            out.push(m);
            if done {
                return out;
            }
        }
    }

    /// Rejoue un transfert sur une autre machine, comme venant de la machine 1.
    fn deliver(to: &mut Sync<Fake>, msgs: Vec<Message>) {
        for m in msgs {
            match m {
                Message::ClipStart { id, header, total } => to.handle(ClipEvent::Start { peer: 1, id, header, total }),
                Message::ClipChunk { id, data } => to.handle(ClipEvent::Chunk { peer: 1, id, data }),
                Message::ClipCancel { id } => to.handle(ClipEvent::Cancel { peer: 1, id }),
                _ => {}
            }
        }
    }

    fn nothing_sent(rx: &Receiver<Message>) -> bool {
        rx.recv_timeout(Duration::from_millis(200)).is_err()
    }

    #[test]
    fn sends_local_changes_once() {
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "once");
        s.poll();
        assert!(nothing_sent(&rx));
        fake.user_copies_text("bonjour");
        s.poll();
        let b = Fake::default();
        let (mut other, _) = sync(&b, "once-b");
        deliver(&mut other, transfer(&rx));
        assert_eq!(b.written()[0].0.text.as_deref(), Some("bonjour"));
        s.poll();
        assert!(nothing_sent(&rx));
    }

    #[test]
    fn received_content_is_not_echoed() {
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "echo");
        let header = ClipHeader { text: Some("x".repeat(200_000)), ..Default::default() };
        let bytes = header.encode();
        s.handle(ClipEvent::Start { peer: 1, id: 9, header: bytes.len() as u32, total: bytes.len() as u64 });
        for c in bytes.chunks(CLIP_CHUNK) {
            s.handle(ClipEvent::Chunk { peer: 1, id: 9, data: c.to_vec() });
        }
        assert_eq!(fake.written()[0].0.text.as_ref().unwrap().len(), 200_000);
        s.poll();
        assert!(nothing_sent(&rx));
    }

    #[test]
    fn sensitive_content_is_skipped() {
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "sensitive");
        fake.0.lock().unwrap().sensitive = true;
        fake.user_copies_text("mot-de-passe");
        s.poll();
        assert!(nothing_sent(&rx));
    }

    #[test]
    fn too_big_is_skipped() {
        let fake = Fake::default();
        let (mut s, rx) = sync_with(&fake, "big", ClipboardCfg { max_size_mb: 1, ..Default::default() });
        fake.user_copies_text(&"y".repeat(2 * 1024 * 1024));
        s.poll();
        assert!(nothing_sent(&rx));
    }

    #[test]
    fn unsupported_content_is_read_twice_then_left() {
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "unsupported");
        {
            let mut st = fake.0.lock().unwrap();
            st.seq += 1;
            st.unsupported = true;
        }
        s.poll();
        s.poll();
        assert_eq!(fake.0.lock().unwrap().reads, 1);
        s.retry_at = Some(Instant::now());
        s.poll();
        s.poll();
        assert_eq!(fake.0.lock().unwrap().reads, 2);
        assert!(nothing_sent(&rx));
    }

    #[test]
    fn images_travel_in_their_original_format() {
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "image");
        let jpeg = ClipImage { format: ImageFormat::Jpeg, data: vec![0xff, 0xd8, 0xff, 1, 2, 3] };
        fake.user_copies(Content {
            header: ClipHeader { images: vec![jpeg.clone()], ..Default::default() },
            sources: vec![],
        });
        s.poll();
        let b = Fake::default();
        let (mut other, _) = sync(&b, "image-b");
        deliver(&mut other, transfer(&rx));
        assert_eq!(b.written()[0].0.images, [jpeg]);
    }

    #[test]
    fn files_and_folders_are_streamed_to_disk() {
        let src = temp_dir("files-src");
        std::fs::create_dir_all(src.join("Dossier/Sous")).unwrap();
        std::fs::write(src.join("Dossier/a.txt"), "aaa").unwrap();
        std::fs::write(src.join("Dossier/Sous/vide"), "").unwrap();
        let big: Vec<u8> = (0..150_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(src.join("gros.bin"), &big).unwrap();
        let (files, sources) = collect_files(&[src.join("Dossier"), src.join("gros.bin")], 10 << 20).unwrap();
        assert_eq!(
            files.iter().map(|f| (f.path.as_str(), f.size)).collect::<Vec<_>>(),
            [
                ("Dossier", None),
                ("Dossier/Sous", None),
                ("Dossier/Sous/vide", Some(0)),
                ("Dossier/a.txt", Some(3)),
                ("gros.bin", Some(150_000))
            ]
        );
        assert!(collect_files(&[src.join("gros.bin")], 1000).is_err());

        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "files");
        fake.user_copies(Content { header: ClipHeader { files, ..Default::default() }, sources });
        s.poll();
        let b = Fake::default();
        let (mut other, _) = sync(&b, "files-b");
        deliver(&mut other, transfer(&rx));
        let (_, roots) = b.written()[0].clone();
        assert_eq!(roots.len(), 2);
        assert_eq!(std::fs::read(roots[0].join("a.txt")).unwrap(), b"aaa");
        assert!(roots[0].join("Sous/vide").is_file());
        assert_eq!(std::fs::read(&roots[1]).unwrap(), big);
    }

    #[test]
    fn an_image_file_also_pastes_as_an_image() {
        let src = temp_dir("imgfile");
        let png = encode_png(2, 2, &[9; 16]).unwrap();
        std::fs::write(src.join("capture.png"), &png).unwrap();
        let (files, sources) = collect_files(&[src.join("capture.png")], 1 << 20).unwrap();
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "imgfile-a");
        fake.user_copies(Content { header: ClipHeader { files, ..Default::default() }, sources });
        s.poll();
        let b = Fake::default();
        let (mut other, _) = sync(&b, "imgfile-b");
        deliver(&mut other, transfer(&rx));
        let (h, roots) = b.written()[0].clone();
        assert_eq!(h.images, [ClipImage { format: ImageFormat::Png, data: png }]);
        assert_eq!(roots.len(), 1);
    }

    #[test]
    fn files_can_be_turned_off() {
        let fake = Fake::default();
        let (mut s, _rx) = sync_with(&fake, "nofiles", ClipboardCfg { files: false, ..Default::default() });
        let header = ClipHeader { files: vec![ClipFile { path: "f".into(), size: Some(3) }], ..Default::default() };
        let bytes = header.encode();
        s.handle(ClipEvent::Start { peer: 1, id: 1, header: bytes.len() as u32, total: bytes.len() as u64 + 3 });
        s.handle(ClipEvent::Chunk { peer: 1, id: 1, data: bytes });
        s.handle(ClipEvent::Chunk { peer: 1, id: 1, data: vec![1, 2, 3] });
        assert!(fake.written().is_empty());
    }

    #[test]
    fn primary_relays_to_other_machines() {
        let fake = Fake::default();
        let (mut s, rx) = sync(&fake, "relay");
        // Contenu reçu de la machine 2 : la machine 1 le reçoit aussi.
        s.handle(ClipEvent::Connected { peer: 2 });
        let header = ClipHeader { text: Some("relais".into()), ..Default::default() };
        let bytes = header.encode();
        s.handle(ClipEvent::Start { peer: 2, id: 4, header: bytes.len() as u32, total: bytes.len() as u64 });
        s.handle(ClipEvent::Chunk { peer: 2, id: 4, data: bytes.clone() });
        let msgs = transfer(&rx);
        assert!(matches!(&msgs[1], Message::ClipChunk { data, .. } if *data == bytes));
    }

    #[test]
    fn unsafe_paths_are_refused() {
        assert!(safe_relative("a/b.txt").is_some());
        for bad in ["../x", "/etc/passwd", "a//b", "C:\\x", "a/./b", ""] {
            assert!(safe_relative(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn cancelled_transfer_leaves_nothing() {
        let fake = Fake::default();
        let (mut s, _rx) = sync(&fake, "cancel");
        let base = temp_dir("cancel");
        s.staging = Staging::new(base.clone());
        let header = ClipHeader { files: vec![ClipFile { path: "f".into(), size: Some(10) }], ..Default::default() };
        let bytes = header.encode();
        s.handle(ClipEvent::Start { peer: 1, id: 1, header: bytes.len() as u32, total: bytes.len() as u64 + 10 });
        s.handle(ClipEvent::Chunk { peer: 1, id: 1, data: bytes });
        s.handle(ClipEvent::Chunk { peer: 1, id: 1, data: vec![1; 4] });
        s.handle(ClipEvent::Cancel { peer: 1, id: 1 });
        assert!(fake.written().is_empty());
        assert_eq!(std::fs::read_dir(&base).map(|d| d.count()).unwrap_or(0), 0);
    }

    #[test]
    fn png_roundtrip() {
        let (w, h) = (3u32, 2u32);
        let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i * 7) as u8).collect();
        let png = encode_png(w, h, &rgba).unwrap();
        assert_eq!(decode_image(&png).unwrap(), (w, h, rgba));
    }
}
