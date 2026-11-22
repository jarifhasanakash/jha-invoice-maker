//! Persistent JSON storage.
//!
//! Data lives in `%APPDATA%\JHA Invoice Maker\database.json`. If a file named
//! `portable.txt` (or an existing `database.json`) sits next to the executable,
//! the data is kept there instead, so the app can run from a USB stick.
//!
//! On first launch, data from the previous Electron version is imported
//! automatically from its Chromium "Local Storage" (read from a temporary copy,
//! the old files are never modified).

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{self, Database};

const APP_DIR: &str = "JHA Invoice Maker";
const DB_FILE: &str = "database.json";

pub struct Storage {
    pub path: PathBuf,
}

/// What happened when the database was opened.
pub enum LoadOutcome {
    Loaded,
    Fresh,
    ImportedFromElectron { clients: usize, invoices: usize },
    /// The file existed but could not be read; a copy was kept and a fresh DB started.
    Recovered { backup: PathBuf, error: String },
}

impl Storage {
    pub fn locate() -> Self {
        if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
            if dir.join("portable.txt").exists() || dir.join(DB_FILE).exists() {
                return Self { path: dir.join(DB_FILE) };
            }
        }
        let base = dirs::data_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            path: base.join(APP_DIR).join(DB_FILE),
        }
    }

    pub fn folder(&self) -> PathBuf {
        self.path.parent().map(Path::to_path_buf).unwrap_or_default()
    }

    pub fn load(&self) -> (Database, LoadOutcome) {
        match fs::read_to_string(&self.path) {
            Ok(text) => match model::import_backup(&text) {
                Ok(db) => (db, LoadOutcome::Loaded),
                Err(error) => {
                    let backup = self.path.with_extension(format!(
                        "corrupt-{}.json",
                        chrono::Local::now().format("%Y%m%d-%H%M%S")
                    ));
                    let _ = fs::copy(&self.path, &backup);
                    (Database::default(), LoadOutcome::Recovered { backup, error })
                }
            },
            Err(_) => match import_from_electron() {
                Some(db) => {
                    let outcome = LoadOutcome::ImportedFromElectron {
                        clients: db.clients.len(),
                        invoices: db.invoices.len(),
                    };
                    (db, outcome)
                }
                None => (Database::demo(), LoadOutcome::Fresh),
            },
        }
    }

    /// Atomically writes the database (temp file + rename), keeping one `.bak` copy.
    pub fn save(&self, db: &Database) -> Result<(), String> {
        let dir = self.folder();
        fs::create_dir_all(&dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
        let json = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json.as_bytes()).map_err(|e| format!("Cannot write database: {e}"))?;
        if self.path.exists() {
            let _ = fs::copy(&self.path, self.path.with_extension("json.bak"));
        }
        fs::rename(&tmp, &self.path).map_err(|e| format!("Cannot replace database: {e}"))?;
        Ok(())
    }
}

/* ------------------------------------------------------------------------- */
/* One-time import from the Electron version                                  */
/* ------------------------------------------------------------------------- */

const MIRROR_KEY: &[u8] = b"jha_invoice_db_mirror";

fn electron_leveldb_dir() -> Option<PathBuf> {
    let base = dirs::config_dir()?; // %APPDATA% (Roaming)
    ["jha-invoice-maker", "Invoice Maker", "JHA Invoice Maker"]
        .iter()
        .map(|name| base.join(name).join("Local Storage").join("leveldb"))
        .find(|p| p.join("CURRENT").exists())
}

/// Decodes a Chromium localStorage value (`0x01` + Latin-1, or `0x00` + UTF-16LE).
fn decode_chromium_string(bytes: &[u8]) -> Option<String> {
    let (tag, rest) = bytes.split_first()?;
    match tag {
        1 => Some(rest.iter().map(|&b| b as char).collect()),
        0 => {
            let units: Vec<u16> = rest
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&units).ok()
        }
        _ => None,
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "LOCK" {
            continue;
        }
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), dst.join(name))?;
        }
    }
    Ok(())
}

fn import_from_electron() -> Option<Database> {
    let src = electron_leveldb_dir()?;
    let tmp = std::env::temp_dir().join(format!("jha-invoice-import-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    copy_dir(&src, &tmp).ok()?;

    let result = (|| {
        let mut opts = rusty_leveldb::Options::default();
        opts.create_if_missing = false;
        let mut db = rusty_leveldb::DB::open(&tmp, opts).ok()?;
        let mut key = b"_file://\x00\x01".to_vec();
        key.extend_from_slice(MIRROR_KEY);
        let value = match db.get(&key) {
            Some(v) => v.to_vec(),
            None => {
                // Fall back to scanning for any origin that holds the mirror key.
                use rusty_leveldb::LdbIterator;
                let mut it = db.new_iter().ok()?;
                let mut found = None;
                while let Some((k, v)) = it.next() {
                    if k.ends_with(MIRROR_KEY) {
                        found = Some(v.to_vec());
                    }
                }
                found?
            }
        };
        let text = decode_chromium_string(&value)?;
        model::import_backup(&text).ok()
    })();

    let _ = fs::remove_dir_all(&tmp);
    result.filter(|db| {
        !(db.invoices.is_empty() && db.clients.is_empty() && !db.profile.has_custom_details())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_chromium_strings() {
        assert_eq!(decode_chromium_string(b"\x01abc").as_deref(), Some("abc"));
        assert_eq!(decode_chromium_string(&[0, 0x3F, 0x09]).as_deref(), Some("\u{093F}"));
    }
}
