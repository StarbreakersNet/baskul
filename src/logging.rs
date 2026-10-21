//! Journal asynchrone : les appels `log::info!` etc. ne bloquent jamais (ils
//! sont aussi utilisés depuis les hooks d'entrée, qui doivent rester très
//! rapides). Un thread dédié écrit dans le fichier et sur la sortie d'erreur.

use crossbeam_channel::{Receiver, Sender, bounded};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;

enum Item {
    Line(String),
    Flush(Sender<()>),
}

struct Logger {
    tx: Sender<Item>,
    level: log::LevelFilter,
}

static LOGGER: OnceLock<Logger> = OnceLock::new();
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let target = record.target().trim_start_matches("bascule::");
        let line = format!("{now} {:<5} [{target}] {}\n", record.level(), record.args());
        // try_send : si le thread d'écriture est en retard, on perd la ligne
        // plutôt que de bloquer un hook d'entrée.
        let _ = self.tx.try_send(Item::Line(line));
    }

    fn flush(&self) {
        let (ack_tx, ack_rx) = bounded(1);
        if self.tx.send_timeout(Item::Flush(ack_tx), Duration::from_millis(200)).is_ok() {
            let _ = ack_rx.recv_timeout(Duration::from_millis(500));
        }
    }
}

fn open(path: &Path) -> Option<File> {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > MAX_LOG_SIZE {
            let _ = std::fs::rename(path, path.with_extension("old.log"));
        }
    }
    OpenOptions::new().create(true).append(true).open(path).ok()
}

fn writer(rx: Receiver<Item>, path: PathBuf, to_stderr: bool) {
    let mut file = open(&path);
    let mut written: u64 = 0;
    for item in rx {
        match item {
            Item::Line(line) => {
                if to_stderr {
                    let _ = std::io::stderr().write_all(line.as_bytes());
                }
                if let Some(f) = file.as_mut() {
                    let _ = f.write_all(line.as_bytes());
                    written += line.len() as u64;
                    if written > MAX_LOG_SIZE {
                        drop(file.take());
                        file = open(&path);
                        written = 0;
                    }
                }
            }
            Item::Flush(ack) => {
                if let Some(f) = file.as_mut() {
                    let _ = f.flush();
                }
                let _ = ack.send(());
            }
        }
    }
}

/// Initialise le journal. `to_stderr` : recopier aussi sur la sortie d'erreur.
pub fn init(path: PathBuf, level: log::LevelFilter, to_stderr: bool) {
    let (tx, rx) = bounded::<Item>(4096);
    let p = path.clone();
    std::thread::Builder::new()
        .name("bascule-log".into())
        .spawn(move || writer(rx, p, to_stderr))
        .expect("thread de journal");
    let _ = LOG_PATH.set(path);
    if LOGGER.set(Logger { tx, level }).is_ok() {
        let _ = log::set_logger(LOGGER.get().unwrap());
        log::set_max_level(level);
    }
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("?");
        log::error!("ERREUR FATALE (panic) dans le thread « {name} » : {info}");
        flush();
    }));
}

pub fn flush() {
    if let Some(l) = LOGGER.get() {
        log::Log::flush(l);
    }
}

pub fn log_path() -> Option<PathBuf> {
    LOG_PATH.get().cloned()
}
