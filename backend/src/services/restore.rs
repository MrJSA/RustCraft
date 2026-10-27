//! Restoring a backup from the admin (Settings → Export & Backups), so nobody needs a shell for it.
//!
//! The backup ZIP comes either straight from the GitHub backup repository or as an upload. The database
//! dump is loaded with `psql` in one transaction — if anything fails the database stays exactly as it was.
//! Then the uploaded files are copied back and the backend exits; Docker restarts it, which runs the
//! migrations a backup from an older shop version still needs and starts with fresh connections.

use crate::services::backup::{human_size, GitHub, TempDir};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{error, info, warn};

/// Seconds between a finished restore and the restart, so the admin page still sees the result
const RESTART_DELAY_SECS: u64 = 3;

#[derive(Debug, Clone, Serialize)]
pub struct RestoreStatus {
    /// idle | running | success | failed
    pub state: &'static str,
    pub step: String,
    pub message: String,
    pub file: String,
    pub started_at: Option<DateTime<Utc>>,
}

static STATUS: Mutex<Option<RestoreStatus>> = Mutex::new(None);

pub fn status() -> RestoreStatus {
    STATUS.lock().ok().and_then(|s| s.clone()).unwrap_or(RestoreStatus {
        state: "idle",
        step: String::new(),
        message: String::new(),
        file: String::new(),
        started_at: None,
    })
}

pub fn is_running() -> bool {
    status().state == "running"
}

fn update(f: impl FnOnce(&mut RestoreStatus)) {
    if let Ok(mut guard) = STATUS.lock() {
        if let Some(s) = guard.as_mut() {
            f(s);
        }
    }
}

fn step(text: &str) {
    info!("Restore: {}", text);
    update(|s| s.step = text.to_string());
}

pub enum Source {
    GitHub { repo: String, token: String, asset_id: u64 },
    Upload(axum::body::Bytes),
}

/// Starts a restore in the background; progress is read with [`status`].
pub fn start(pool: PgPool, source: Source, file: String, keep_backups_off: bool) -> Result<(), String> {
    if crate::services::backup::is_running() {
        return Err("A backup is being created right now. Wait until it has finished.".to_string());
    }
    {
        let mut guard = STATUS.lock().map_err(|_| "Restore status is unavailable.".to_string())?;
        if guard.as_ref().is_some_and(|s| s.state == "running") {
            return Err("A restore is already running.".to_string());
        }
        *guard = Some(RestoreStatus {
            state: "running",
            step: "Starting…".to_string(),
            message: String::new(),
            file: file.clone(),
            started_at: Some(Utc::now()),
        });
    }
    tokio::spawn(async move {
        match run(&pool, source, keep_backups_off).await {
            Ok(message) => {
                info!("Restore of {} finished: {}", file, message);
                update(|s| {
                    s.state = "success";
                    s.step = "Restarting the shop…".to_string();
                    s.message = message;
                });
                tokio::time::sleep(std::time::Duration::from_secs(RESTART_DELAY_SECS)).await;
                info!("Restarting the backend to load the restored data");
                std::process::exit(0);
            }
            Err(e) => {
                error!("Restore of {} failed: {}", file, e);
                update(|s| {
                    s.state = "failed";
                    s.message = e;
                });
            }
        }
    });
    Ok(())
}

async fn run(pool: &PgPool, source: Source, keep_backups_off: bool) -> Result<String, String> {
    let work = TempDir(std::env::temp_dir().join(format!("shop-restore-{}", Utc::now().format("%Y%m%d-%H%M%S"))));
    tokio::fs::create_dir_all(&work.0).await.map_err(|e| format!("Cannot create a temporary folder: {}", e))?;
    let zip_path = work.0.join("backup.zip");

    match source {
        Source::GitHub { repo, token, asset_id } => {
            step("Downloading the backup from GitHub…");
            GitHub::new(&repo, &token).download_asset(asset_id, &zip_path).await?;
        }
        Source::Upload(bytes) => {
            tokio::fs::write(&zip_path, &bytes).await.map_err(|e| format!("Cannot store the uploaded file: {}", e))?;
        }
    }

    step("Checking the backup…");
    let (zip, dir) = (zip_path.clone(), work.0.clone());
    let dump = tokio::task::spawn_blocking(move || extract_dump(&zip, &dir))
        .await
        .map_err(|e| format!("Checking the backup crashed: {}", e))??;

    step("Restoring the database…");
    load_dump(&dump).await?;
    let _ = tokio::fs::remove_file(&dump).await;
    if keep_backups_off {
        let _ = sqlx::query("UPDATE backup_settings SET enabled = FALSE WHERE id = 1").execute(pool).await;
    }

    step("Restoring uploaded files…");
    let zip = zip_path.clone();
    let files = tokio::task::spawn_blocking(move || extract_uploads(&zip, Path::new("uploads")))
        .await
        .map_err(|e| format!("Restoring the uploaded files crashed: {}", e))
        .and_then(|r| r);
    let size = tokio::fs::metadata(&zip_path).await.map(|m| m.len()).unwrap_or(0);
    Ok(match files {
        Ok(n) => format!("Backup restored ({}, database and {} uploaded file(s)).", human_size(size), n),
        Err(e) => {
            warn!("Restore: uploaded files incomplete: {}", e);
            format!("The database was restored, but not all uploaded files: {}", e)
        }
    })
}

/// Checks that the ZIP is a shop backup this shop version can load and extracts `database.sql`.
fn extract_dump(zip_path: &Path, work: &Path) -> Result<PathBuf, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "This file is not a shop backup (it is not a ZIP archive).".to_string())?;

    if let Ok(mut entry) = archive.by_name("backup-info.json") {
        let mut text = String::new();
        let _ = entry.read_to_string(&mut text);
        let info: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        if info["format"].as_str() != Some("rustwebshop-backup") {
            return Err("This ZIP file is not a backup of this shop software.".to_string());
        }
    }

    let target = work.join("database.sql");
    {
        let mut entry = archive
            .by_name("database.sql")
            .map_err(|_| "This ZIP file is not a shop backup (database.sql is missing).".to_string())?;
        let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("The backup file is damaged: {}", e))?;
    }

    let unknown = unknown_migrations(&target)?;
    if !unknown.is_empty() {
        return Err(format!(
            "This backup was made with a newer shop version (it contains the database changes {}). Update this shop first (Settings → System & Updates), then restore again.",
            unknown.join(", ")
        ));
    }
    Ok(target)
}

/// Schema migrations recorded in the dump that this shop version does not know.
fn unknown_migrations(dump: &Path) -> Result<Vec<String>, String> {
    let reader = std::io::BufReader::new(std::fs::File::open(dump).map_err(|e| e.to_string())?);
    let mut in_table = false;
    let mut found = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("The database dump cannot be read: {}", e))?;
        if in_table {
            if line == "\\." {
                break;
            }
            found.extend(line.split('\t').next().map(str::to_string));
        } else if line.starts_with("COPY public._schema_migrations ") {
            in_table = true;
        }
    }
    if !in_table {
        return Err("The database dump in this backup is incomplete (it has no schema version).".to_string());
    }
    let known: Vec<&str> = crate::db::MIGRATIONS.iter().map(|(v, _)| *v).collect();
    // Older shop versions recorded some migrations by their file name (`0008_….sql`)
    Ok(found.into_iter().filter(|v| !known.contains(&v.strip_suffix(".sql").unwrap_or(v))).collect())
}

/// Loads the dump in one transaction. The schema is emptied first, so tables a newer shop version added
/// (and their links to the restored tables) cannot block it; the migrations recreate them after the restart.
async fn load_dump(dump: &Path) -> Result<(), String> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is not set.".to_string())?;
    let output = tokio::process::Command::new("psql")
        .args(["--no-psqlrc", "--quiet", "--set", "ON_ERROR_STOP=1", "--single-transaction"])
        .args(["--command", "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"])
        .arg("--file")
        .arg(dump)
        .arg("--dbname")
        .arg(&database_url)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "psql is not installed in the backend container. Rebuild it: docker compose up -d --build backend".to_string()
            } else {
                format!("Could not start psql: {}", e)
            }
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail: String = stderr.trim().chars().rev().take(600).collect::<Vec<_>>().into_iter().rev().collect();
        return Err(format!("Loading the database failed — nothing was changed. {}", detail));
    }
    Ok(())
}

/// Copies `uploads/…` from the backup into the uploads folder (existing files with the same name are replaced).
fn extract_uploads(zip_path: &Path, target: &Path) -> Result<usize, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut count = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        // Only plain names below uploads/ — no absolute paths or `..`, so nothing lands outside the uploads folder
        let Some(path) = entry.enclosed_name() else { continue };
        let Ok(rel) = path.strip_prefix("uploads") else { continue };
        if rel.as_os_str().is_empty() || rel.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
            continue;
        }
        let dest = target.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = std::fs::File::create(&dest).map_err(|e| format!("{}: {}", rel.display(), e))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("{}: {}", rel.display(), e))?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir() -> TempDir {
        let dir = std::env::temp_dir().join(format!("restore-test-{}", crate::services::auth::random_token(6)));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        for (name, data) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }

    const DUMP: &str = "CREATE TABLE x();\nCOPY public._schema_migrations (version, applied_at) FROM stdin;\n0001_initial_schema\t2026-01-01 00:00:00+00\n0002_enhanced_features\t2026-01-01 00:00:00+00\n\\.\n";

    #[test]
    fn known_migrations_pass_and_newer_ones_are_reported() {
        let dir = temp_dir();
        let dump = dir.0.join("database.sql");
        std::fs::write(&dump, DUMP).unwrap();
        assert!(unknown_migrations(&dump).unwrap().is_empty());

        std::fs::write(&dump, DUMP.replace("0002_enhanced_features\t", "0008_legal_identity_fields.sql\t")).unwrap();
        assert!(unknown_migrations(&dump).unwrap().is_empty(), "file-name entries of older shop versions are known");

        std::fs::write(&dump, DUMP.replace("\\.\n", "9999_from_the_future\t2030-01-01 00:00:00+00\n\\.\n")).unwrap();
        assert_eq!(unknown_migrations(&dump).unwrap(), ["9999_from_the_future"]);

        std::fs::write(&dump, "CREATE TABLE x();\n").unwrap();
        assert!(unknown_migrations(&dump).is_err());
    }

    #[test]
    fn only_shop_backups_are_accepted() {
        let dir = temp_dir();
        let zip = dir.0.join("b.zip");
        write_zip(&zip, &[("backup-info.json", br#"{"format":"rustwebshop-backup"}"#), ("database.sql", DUMP.as_bytes())]);
        assert_eq!(extract_dump(&zip, &dir.0).unwrap(), dir.0.join("database.sql"));

        write_zip(&zip, &[("backup-info.json", br#"{"format":"something-else"}"#), ("database.sql", DUMP.as_bytes())]);
        assert!(extract_dump(&zip, &dir.0).is_err());
        write_zip(&zip, &[("notes.txt", b"hi")]);
        assert!(extract_dump(&zip, &dir.0).is_err());
        std::fs::write(&zip, b"not a zip").unwrap();
        assert!(extract_dump(&zip, &dir.0).is_err());
    }

    #[test]
    fn uploads_are_extracted_inside_the_target_only() {
        let dir = temp_dir();
        let zip = dir.0.join("b.zip");
        write_zip(
            &zip,
            &[
                ("database.sql", b"x"),
                ("uploads/logo.png", b"png"),
                ("uploads/sub/manual.pdf", b"pdf"),
                ("uploads/../escape.txt", b"no"),
                ("/etc/evil", b"no"),
            ],
        );
        let target = dir.0.join("uploads");
        assert_eq!(extract_uploads(&zip, &target).unwrap(), 2);
        assert_eq!(std::fs::read(target.join("sub/manual.pdf")).unwrap(), b"pdf");
        assert!(!dir.0.join("escape.txt").exists());
    }
}
