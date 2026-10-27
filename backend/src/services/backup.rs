//! Automatic off-site backups.
//!
//! A backup is one ZIP file with a complete PostgreSQL dump (`database.sql`, everything incl. orders,
//! customers and settings) plus the uploaded files (`uploads/`). It is uploaded as a release asset to a
//! private GitHub repository chosen by the shop owner — one release per backup, the oldest ones are
//! removed so only the configured number is kept. Release assets do not grow the git history, so the
//! repository stays small no matter how many backups were made over the years.
//!
//! A scheduler checks every few minutes whether a daily / weekly (Sunday) / monthly (1st) backup is due
//! at the configured hour (UTC). Failed backups are retried hourly and the superadmins get one email.

use base64::Engine;
use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, TimeZone, Utc};
use reqwest::{Method, RequestBuilder, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, warn};

const GITHUB_API: &str = "https://api.github.com";
const TAG_PREFIX: &str = "backup-";
/// GitHub refuses release assets of 2 GiB and more
const MAX_ASSET_BYTES: u64 = 2 * 1024 * 1024 * 1024 - 1;
const CHECK_INTERVAL_SECS: u64 = 300;
const RETRY_AFTER_FAILURE_MINUTES: i64 = 60;
pub const FREQUENCIES: [&str; 3] = ["daily", "weekly", "monthly"];

static RUNNING: AtomicBool = AtomicBool::new(false);

pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BackupSettings {
    pub enabled: bool,
    pub github_repo: String,
    pub github_token: String,
    pub frequency: String,
    pub backup_hour_utc: i16,
    pub keep_count: i32,
    pub include_media: bool,
    pub token_expires_at: Option<String>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_message: Option<String>,
    pub last_file: Option<String>,
    pub last_size_bytes: Option<i64>,
    pub last_url: Option<String>,
    pub failure_notified: bool,
}

impl BackupSettings {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, BackupSettings>("SELECT * FROM backup_settings WHERE id = 1").fetch_one(pool).await
    }

    pub fn is_configured(&self) -> bool {
        !self.github_repo.is_empty() && !self.github_token.is_empty()
    }

    fn hour(&self) -> u32 {
        self.backup_hour_utc.clamp(0, 23) as u32
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        if !self.enabled || !self.is_configured() {
            return false;
        }
        let retry_ok = self
            .last_attempt_at
            .map_or(true, |t| now - t >= Duration::minutes(RETRY_AFTER_FAILURE_MINUTES) || self.last_status.as_deref() == Some("success"));
        let missed_slot = self.last_success_at.map_or(true, |t| t < last_slot(&self.frequency, self.hour(), now));
        missed_slot && retry_ok
    }

    pub fn next_run_at(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        if !self.enabled || !self.is_configured() {
            return None;
        }
        if self.last_success_at.map_or(true, |t| t < last_slot(&self.frequency, self.hour(), now)) {
            // Due now, or after the retry pause of a failed attempt
            let retry = self.last_attempt_at.map(|t| t + Duration::minutes(RETRY_AFTER_FAILURE_MINUTES));
            return Some(retry.filter(|r| *r > now && self.last_status.as_deref() == Some("failed")).unwrap_or(now));
        }
        Some(next_slot(&self.frequency, self.hour(), now))
    }

    /// What the admin API returns — never the token itself.
    pub fn public_json(&self) -> Value {
        let token = self.github_token.trim();
        let hint = (token.len() > 4).then(|| format!("…{}", &token[token.len() - 4..]));
        json!({
            "enabled": self.enabled,
            "github_repo": self.github_repo,
            "has_token": !token.is_empty(),
            "token_hint": hint,
            "token_expires_at": self.token_expires_at,
            "frequency": self.frequency,
            "backup_hour_utc": self.backup_hour_utc,
            "keep_count": self.keep_count,
            "include_media": self.include_media,
            "running": is_running(),
            "next_run_at": self.next_run_at(Utc::now()),
            "last_attempt_at": self.last_attempt_at,
            "last_success_at": self.last_success_at,
            "last_status": self.last_status,
            "last_message": self.last_message,
            "last_file": self.last_file,
            "last_size_bytes": self.last_size_bytes,
            "last_url": self.last_url,
            "releases_url": (!self.github_repo.is_empty()).then(|| format!("https://github.com/{}/releases", self.github_repo)),
        })
    }
}

// ------------------------------------------------------------------ schedule

fn at_hour(day: NaiveDate, hour: u32) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(hour, 0, 0).unwrap_or_default())
}

/// The most recent scheduled backup time at or before `now`.
fn last_slot(frequency: &str, hour: u32, now: DateTime<Utc>) -> DateTime<Utc> {
    let today = now.date_naive();
    match frequency {
        "weekly" => {
            let sunday = today - Duration::days(today.weekday().num_days_from_sunday() as i64);
            let slot = at_hour(sunday, hour);
            if slot > now { slot - Duration::days(7) } else { slot }
        }
        "monthly" => {
            let first = today.with_day(1).unwrap_or(today);
            let slot = at_hour(first, hour);
            if slot > now {
                at_hour(first.checked_sub_months(Months::new(1)).unwrap_or(first), hour)
            } else {
                slot
            }
        }
        _ => {
            let slot = at_hour(today, hour);
            if slot > now { slot - Duration::days(1) } else { slot }
        }
    }
}

fn next_slot(frequency: &str, hour: u32, now: DateTime<Utc>) -> DateTime<Utc> {
    let last = last_slot(frequency, hour, now);
    match frequency {
        "weekly" => last + Duration::days(7),
        "monthly" => last.checked_add_months(Months::new(1)).unwrap_or(last + Duration::days(31)),
        _ => last + Duration::days(1),
    }
}

/// Checks every few minutes whether a backup is due and runs it.
pub fn spawn_scheduler(pool: PgPool) {
    tokio::spawn(async move {
        let start = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
        let mut tick = tokio::time::interval_at(start, std::time::Duration::from_secs(CHECK_INTERVAL_SECS));
        loop {
            tick.tick().await;
            match BackupSettings::load(&pool).await {
                Ok(s) if s.is_due(Utc::now()) && !is_running() => {
                    let _ = run_backup(&pool).await;
                }
                Ok(_) => {}
                Err(e) => warn!("Backup scheduler cannot read its settings: {}", e),
            }
        }
    });
}

// ------------------------------------------------------------------ running a backup

struct RunningGuard;
impl Drop for RunningGuard {
    fn drop(&mut self) {
        RUNNING.store(false, Ordering::SeqCst);
    }
}

/// Deletes the temporary working directory when the backup ends (successfully or not).
pub(crate) struct TempDir(pub PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub struct BackupOutcome {
    pub file: String,
    pub size: u64,
    pub url: String,
    pub notes: Vec<String>,
}

/// Runs one backup now and records the result. Only one backup runs at a time.
pub async fn run_backup(pool: &PgPool) -> Result<(), String> {
    if crate::services::restore::is_running() {
        return Err("A backup is being restored right now.".to_string());
    }
    if RUNNING.swap(true, Ordering::SeqCst) {
        return Err("A backup is already running.".to_string());
    }
    let _guard = RunningGuard;

    let settings = BackupSettings::load(pool).await.map_err(|e| e.to_string())?;
    let _ = sqlx::query("UPDATE backup_settings SET last_attempt_at = NOW(), last_status = 'running', last_message = NULL WHERE id = 1")
        .execute(pool)
        .await;
    info!("Starting backup to GitHub repository {}", settings.github_repo);

    match create_backup(pool, &settings).await {
        Ok(outcome) => {
            let mut message = format!("Backup uploaded ({}).", human_size(outcome.size));
            for note in &outcome.notes {
                message.push(' ');
                message.push_str(note);
            }
            info!("Backup {} uploaded ({} bytes)", outcome.file, outcome.size);
            let _ = sqlx::query(
                "UPDATE backup_settings SET last_success_at = NOW(), last_status = 'success', last_message = $1, last_file = $2,
                 last_size_bytes = $3, last_url = $4, failure_notified = FALSE, updated_at = NOW() WHERE id = 1",
            )
            .bind(&message)
            .bind(&outcome.file)
            .bind(outcome.size as i64)
            .bind(&outcome.url)
            .execute(pool)
            .await;
            Ok(())
        }
        Err(e) => {
            error!("Backup failed: {}", e);
            let _ = sqlx::query("UPDATE backup_settings SET last_status = 'failed', last_message = $1, updated_at = NOW() WHERE id = 1")
                .bind(&e)
                .execute(pool)
                .await;
            // One email per failure streak, not one per hourly retry
            if !settings.failure_notified && notify_failure(pool, &settings.github_repo, &e).await {
                let _ = sqlx::query("UPDATE backup_settings SET failure_notified = TRUE WHERE id = 1").execute(pool).await;
            }
            Err(e)
        }
    }
}

async fn create_backup(pool: &PgPool, settings: &BackupSettings) -> Result<BackupOutcome, String> {
    if !settings.is_configured() {
        return Err("No GitHub repository or access token is configured.".to_string());
    }
    let gh = GitHub::new(&settings.github_repo, &settings.github_token);
    let repo = gh.check_repo().await?;
    if let Some(expiry) = &repo.token_expires_at {
        let _ = sqlx::query("UPDATE backup_settings SET token_expires_at = $1 WHERE id = 1").bind(expiry).execute(pool).await;
    }
    gh.ensure_initialized().await?;

    let now = Utc::now();
    let stamp = now.format("%Y%m%d-%H%M%S").to_string();
    let file_name = format!("shop-backup-{}.zip", stamp);
    let work = TempDir(std::env::temp_dir().join(format!("shop-backup-{}", stamp)));
    tokio::fs::create_dir_all(&work.0).await.map_err(|e| format!("Cannot create a temporary folder: {}", e))?;

    let dump_path = work.0.join("database.sql");
    dump_database(&dump_path).await?;

    let store_name: String = sqlx::query_scalar("SELECT store_name FROM store_settings WHERE id = 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let info = json!({
        "format": "rustwebshop-backup",
        "format_version": 1,
        "created_at": now.to_rfc3339(),
        "store_name": store_name,
        "contains": if settings.include_media { json!(["database.sql", "uploads/"]) } else { json!(["database.sql"]) },
    });

    let zip_path = work.0.join(&file_name);
    let uploads = settings.include_media.then(|| PathBuf::from("uploads"));
    let (zip_out, dump_in) = (zip_path.clone(), dump_path.clone());
    let file_count = tokio::task::spawn_blocking(move || write_archive(&zip_out, &dump_in, uploads.as_deref(), &info))
        .await
        .map_err(|e| format!("Creating the backup archive crashed: {}", e))??;
    let _ = tokio::fs::remove_file(&dump_path).await;

    let size = tokio::fs::metadata(&zip_path).await.map_err(|e| e.to_string())?.len();
    if size > MAX_ASSET_BYTES {
        return Err(format!(
            "The backup is {} — GitHub accepts files up to 2 GB. Turn off \"Include uploaded files\" or clean up the media library.",
            human_size(size)
        ));
    }

    let tag = format!("{}{}", TAG_PREFIX, stamp);
    let title = format!("Backup {} UTC", now.format("%Y-%m-%d %H:%M"));
    let body = format!(
        "Automatic backup of **{}**.\n\n- Database: complete dump (`database.sql`)\n- Uploaded files: {}\n- Size: {}\n\nTo restore: shop admin → Settings → Export & Backups → Restore a Backup (see `BACKUP.md` in the shop repository).",
        if store_name.is_empty() { "the shop" } else { &store_name },
        if settings.include_media { format!("{} file(s) in `uploads/`", file_count) } else { "not included".to_string() },
        human_size(size)
    );
    let release = gh.create_draft_release(&tag, &title, &body).await?;
    if let Err(e) = gh.upload_asset(&release, &file_name, &zip_path, size).await {
        let _ = gh.delete_release(release.id).await;
        return Err(e);
    }
    let url = gh.publish_release(release.id).await.inspect_err(|_| {
        let gh = gh.clone();
        let id = release.id;
        tokio::spawn(async move {
            let _ = gh.delete_release(id).await;
        });
    })?;

    let mut notes = Vec::new();
    match gh.prune(settings.keep_count.max(1) as usize).await {
        Ok(0) => {}
        Ok(n) => notes.push(format!("{} old backup(s) removed.", n)),
        Err(e) => notes.push(format!("Old backups could not be removed: {}", e)),
    }
    if let Some(expiry) = repo.token_expires_at.as_deref().and_then(parse_github_time) {
        let days = (expiry - Utc::now()).num_days();
        if days <= 14 {
            notes.push(format!("The GitHub token expires in {} day(s) — create a new one and save it.", days.max(0)));
        }
    }
    Ok(BackupOutcome { file: file_name, size, url, notes })
}

/// Full plain-SQL dump. `--clean --if-exists` lets it be loaded into an existing (fresh) shop database.
async fn dump_database(target: &Path) -> Result<(), String> {
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is not set.".to_string())?;
    let output = tokio::process::Command::new("pg_dump")
        .args(["--no-owner", "--no-privileges", "--clean", "--if-exists", "--encoding=UTF8", "--file"])
        .arg(target)
        .arg("--dbname")
        .arg(&database_url)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "pg_dump is not installed in the backend container. Rebuild it: docker compose up -d --build backend".to_string()
            } else {
                format!("Could not start pg_dump: {}", e)
            }
        })?;
    if !output.status.success() {
        return Err(format!("The database dump failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    Ok(())
}

/// Writes the backup ZIP and returns the number of uploaded files it contains.
fn write_archive(target: &Path, dump: &Path, uploads: Option<&Path>, info: &Value) -> Result<usize, String> {
    use zip::write::SimpleFileOptions;
    let io = |e: std::io::Error| format!("Writing the backup archive failed: {}", e);
    let zip_err = |e: zip::result::ZipError| format!("Writing the backup archive failed: {}", e);

    let file = std::fs::File::create(target).map_err(io)?;
    let mut zip = zip::ZipWriter::new(std::io::BufWriter::new(file));
    let deflated = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).unix_permissions(0o644);
    // Images, PDFs and archives are already compressed
    let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored).unix_permissions(0o644);

    zip.start_file("backup-info.json", deflated).map_err(zip_err)?;
    zip.write_all(&serde_json::to_vec_pretty(info).unwrap_or_default()).map_err(io)?;
    zip.start_file("RESTORE.txt", deflated).map_err(zip_err)?;
    zip.write_all(RESTORE_TXT.as_bytes()).map_err(io)?;

    let add = |zip: &mut zip::ZipWriter<_>, name: &str, path: &Path, options: SimpleFileOptions| -> Result<(), String> {
        let len = std::fs::metadata(path).map_err(io)?.len();
        zip.start_file(name, options.large_file(len >= u32::MAX as u64)).map_err(zip_err)?;
        let mut src = std::fs::File::open(path).map_err(io)?;
        std::io::copy(&mut src, zip).map_err(io)?;
        Ok(())
    };
    add(&mut zip, "database.sql", dump, deflated)?;

    let mut count = 0;
    if let Some(root) = uploads.filter(|p| p.is_dir()) {
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).map_err(io)? {
                let path = entry.map_err(io)?.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let Ok(rel) = path.strip_prefix(root) else { continue };
                let name = format!("uploads/{}", rel.to_string_lossy().replace('\\', "/"));
                add(&mut zip, &name, &path, stored)?;
                count += 1;
            }
        }
    }

    let mut writer = zip.finish().map_err(zip_err)?;
    writer.flush().map_err(io)?;
    Ok(count)
}

const RESTORE_TXT: &str = "Shop backup
===========

database.sql  complete PostgreSQL dump (orders, customers, products, settings, ...)
uploads/      uploaded images and files

Restore: in the admin under Settings -> Export & Backups -> Restore a Backup
(pick the backup from GitHub or upload this ZIP file).

Or on the command line (in the shop folder on the server):

  docker compose stop backend storefront admin
  docker compose exec -T db psql -v ON_ERROR_STOP=1 --single-transaction -U shop_user -d shop_db < database.sql
  docker compose start backend storefront admin
  docker compose cp uploads/. backend:/app/uploads/

Full guide: BACKUP.md in the shop repository.
";

pub(crate) fn human_size(bytes: u64) -> String {
    let b = bytes as f64;
    if b >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB", b / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024.0 * 1024.0 {
        format!("{:.1} MB", b / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KB", (b / 1024.0).max(1.0))
    }
}

fn parse_github_time(s: &str) -> Option<DateTime<Utc>> {
    // e.g. "2027-10-25 12:00:00 UTC" or "2027-10-25 12:00:00 +0200"
    DateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M:%S %z")
        .map(|d| d.with_timezone(&Utc))
        .ok()
        .or_else(|| chrono::NaiveDateTime::parse_from_str(s.trim().trim_end_matches(" UTC"), "%Y-%m-%d %H:%M:%S").ok().map(|n| n.and_utc()))
}

async fn notify_failure(pool: &PgPool, repo: &str, message: &str) -> bool {
    let settings = match sqlx::query_as::<_, crate::models::StoreSettings>("SELECT * FROM store_settings WHERE id = 1").fetch_one(pool).await {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut recipients: Vec<String> = sqlx::query_scalar(
        "SELECT email FROM admin_users WHERE role = 'superadmin' AND email IS NOT NULL AND TRIM(email) <> '' ORDER BY created_at",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    if recipients.is_empty() && settings.support_email.contains('@') {
        recipients.push(settings.support_email.trim().to_string());
    }
    let mut sent = false;
    for to in recipients {
        match crate::services::email::send_backup_failed_email(&settings, &to, repo, message).await {
            Ok(()) => sent = true,
            Err(e) => warn!("Could not send the backup failure email to {}: {}", to, e),
        }
    }
    sent
}

// ------------------------------------------------------------------ repository name

/// Accepts `owner/repo`, `https://github.com/owner/repo` or the `.git` clone URL.
pub fn normalize_repo(input: &str) -> Result<String, String> {
    let mut s = input.trim();
    for prefix in ["https://", "http://", "www.", "github.com/", "git@github.com:"] {
        s = s.strip_prefix(prefix).unwrap_or(s);
    }
    let s = s.trim_end_matches('/').trim_end_matches(".git");
    let valid_part = |p: &str| !p.is_empty() && p.len() <= 100 && p.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) && p != "." && p != "..";
    match s.split('/').collect::<Vec<_>>().as_slice() {
        [owner, name] if valid_part(owner) && valid_part(name) => Ok(format!("{}/{}", owner, name)),
        _ => Err("Please enter the repository as owner/name, e.g. my-account/shop-backups (or paste its GitHub link).".to_string()),
    }
}

// ------------------------------------------------------------------ GitHub API

#[derive(Clone)]
pub struct GitHub {
    client: reqwest::Client,
    repo: String,
    token: String,
}

pub struct RepoInfo {
    pub html_url: String,
    pub token_expires_at: Option<String>,
}

struct Release {
    id: u64,
    upload_url: String,
}

impl GitHub {
    pub fn new(repo: &str, token: &str) -> Self {
        GitHub { client: reqwest::Client::new(), repo: repo.to_string(), token: token.trim().to_string() }
    }

    fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client
            .request(method, url)
            .bearer_auth(&self.token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .header(reqwest::header::USER_AGENT, "rustwebshop-backup")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    fn api(&self, method: Method, path: &str) -> RequestBuilder {
        self.request(method, &format!("{}/repos/{}{}", GITHUB_API, self.repo, path))
            .timeout(std::time::Duration::from_secs(60))
    }

    async fn send(&self, req: RequestBuilder, action: &str) -> Result<(StatusCode, reqwest::header::HeaderMap, Value), String> {
        let resp = req.send().await.map_err(|e| format!("GitHub is not reachable ({}): {}", action, e))?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        Ok((status, headers, body))
    }

    fn error(&self, status: StatusCode, body: &Value, action: &str) -> String {
        let detail = body["message"].as_str().unwrap_or("");
        match status {
            StatusCode::UNAUTHORIZED => "GitHub rejected the access token — it is wrong, expired or was deleted. Create a new token and save it.".to_string(),
            StatusCode::FORBIDDEN => format!(
                "The access token may not {} in {}. Give the token access to this repository with the permission \"Contents: Read and write\". ({})",
                action, self.repo, detail
            ),
            StatusCode::NOT_FOUND => format!(
                "Repository {} was not found. Check the name and that the token was given access to exactly this repository.",
                self.repo
            ),
            _ => format!("GitHub refused to {} (HTTP {}): {}", action, status.as_u16(), detail),
        }
    }

    /// The repository must exist, be reachable with the token and be private.
    pub async fn check_repo(&self) -> Result<RepoInfo, String> {
        let (status, headers, body) = self.send(self.api(Method::GET, ""), "open the repository").await?;
        if !status.is_success() {
            return Err(self.error(status, &body, "open the repository"));
        }
        if body["private"].as_bool() != Some(true) {
            return Err(format!(
                "{} is a public repository. Backups contain customer data and must only be stored in a private repository.",
                self.repo
            ));
        }
        Ok(RepoInfo {
            html_url: body["html_url"].as_str().unwrap_or_default().to_string(),
            token_expires_at: headers
                .get("github-authentication-token-expiration")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string),
        })
    }

    /// Releases need at least one commit; a brand-new empty repository gets a README.
    pub async fn ensure_initialized(&self) -> Result<(), String> {
        let (status, _, body) = self.send(self.api(Method::GET, "/commits?per_page=1"), "read the repository").await?;
        if status.is_success() {
            return Ok(());
        }
        if status != StatusCode::CONFLICT {
            return Err(self.error(status, &body, "read the repository"));
        }
        let content = base64::engine::general_purpose::STANDARD.encode(README_MD);
        let (status, _, body) = self
            .send(
                self.api(Method::PUT, "/contents/README.md").json(&json!({ "message": "Shop backup repository", "content": content })),
                "write to the repository",
            )
            .await?;
        if !status.is_success() {
            return Err(self.error(status, &body, "write to the repository"));
        }
        Ok(())
    }

    /// Proves write access without leaving anything behind: creates and deletes a temporary tag.
    pub async fn check_write_access(&self) -> Result<(), String> {
        let (status, _, body) = self.send(self.api(Method::GET, "/commits?per_page=1"), "read the repository").await?;
        if !status.is_success() {
            return Err(self.error(status, &body, "read the repository"));
        }
        let sha = body[0]["sha"].as_str().unwrap_or_default().to_string();
        let tag = format!("refs/tags/connection-test-{}", Utc::now().timestamp());
        let (status, _, body) = self
            .send(self.api(Method::POST, "/git/refs").json(&json!({ "ref": tag, "sha": sha })), "write to the repository")
            .await?;
        if !status.is_success() {
            return Err(self.error(status, &body, "write to the repository"));
        }
        let _ = self.send(self.api(Method::DELETE, &format!("/git/{}", tag)), "clean up").await;
        Ok(())
    }

    async fn create_draft_release(&self, tag: &str, title: &str, body: &str) -> Result<Release, String> {
        let (status, _, resp) = self
            .send(
                self.api(Method::POST, "/releases")
                    .json(&json!({ "tag_name": tag, "name": title, "body": body, "draft": true, "prerelease": false })),
                "create a release",
            )
            .await?;
        if !status.is_success() {
            return Err(self.error(status, &resp, "create a release"));
        }
        Ok(Release {
            id: resp["id"].as_u64().ok_or("GitHub did not return a release id.")?,
            upload_url: resp["upload_url"].as_str().unwrap_or_default().split('{').next().unwrap_or_default().to_string(),
        })
    }

    async fn upload_asset(&self, release: &Release, file_name: &str, path: &Path, size: u64) -> Result<(), String> {
        let file = tokio::fs::File::open(path).await.map_err(|e| e.to_string())?;
        let body = reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(file));
        let req = self
            .request(Method::POST, &release.upload_url)
            .query(&[("name", file_name)])
            .header(reqwest::header::CONTENT_TYPE, "application/zip")
            .header(reqwest::header::CONTENT_LENGTH, size)
            .timeout(std::time::Duration::from_secs(3 * 3600))
            .body(body);
        let (status, _, resp) = self.send(req, "upload the backup file").await?;
        if !status.is_success() {
            return Err(self.error(status, &resp, "upload the backup file"));
        }
        Ok(())
    }

    async fn publish_release(&self, id: u64) -> Result<String, String> {
        let (status, _, resp) = self
            .send(self.api(Method::PATCH, &format!("/releases/{}", id)).json(&json!({ "draft": false })), "publish the release")
            .await?;
        if !status.is_success() {
            return Err(self.error(status, &resp, "publish the release"));
        }
        Ok(resp["html_url"].as_str().unwrap_or_default().to_string())
    }

    async fn delete_release(&self, id: u64) -> Result<(), String> {
        let (status, _, resp) = self.send(self.api(Method::DELETE, &format!("/releases/{}", id)), "delete an old backup").await?;
        if !status.is_success() && status != StatusCode::NOT_FOUND {
            return Err(self.error(status, &resp, "delete an old backup"));
        }
        Ok(())
    }

    /// Keeps the newest `keep` backups; removes older ones (release + tag) and leftover drafts.
    async fn prune(&self, keep: usize) -> Result<usize, String> {
        let mut backups = Vec::new();
        for page in 1..=50 {
            let (status, _, resp) = self
                .send(self.api(Method::GET, &format!("/releases?per_page=100&page={}", page)), "list the backups")
                .await?;
            if !status.is_success() {
                return Err(self.error(status, &resp, "list the backups"));
            }
            let items = resp.as_array().cloned().unwrap_or_default();
            if items.is_empty() {
                break;
            }
            for r in items {
                let tag = r["tag_name"].as_str().unwrap_or_default().to_string();
                if let (true, Some(id)) = (tag.starts_with(TAG_PREFIX), r["id"].as_u64()) {
                    backups.push((tag, id, r["draft"].as_bool().unwrap_or(false)));
                }
            }
        }
        // Tags carry the timestamp, so they sort chronologically
        backups.sort_by(|a, b| b.0.cmp(&a.0));
        let (published, drafts): (Vec<_>, Vec<_>) = backups.into_iter().partition(|b| !b.2);
        let mut removed = 0;
        for (tag, id, draft) in published.into_iter().skip(keep).chain(drafts) {
            self.delete_release(id).await?;
            if !draft {
                let _ = self.send(self.api(Method::DELETE, &format!("/git/refs/tags/{}", tag)), "delete an old tag").await;
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// All published backups (newest first) that carry a backup ZIP, for the restore picker.
    pub async fn list_backups(&self) -> Result<Vec<Value>, String> {
        let mut backups = Vec::new();
        for page in 1..=10 {
            let (status, _, resp) = self
                .send(self.api(Method::GET, &format!("/releases?per_page=100&page={}", page)), "list the backups")
                .await?;
            if !status.is_success() {
                return Err(self.error(status, &resp, "list the backups"));
            }
            let items = resp.as_array().cloned().unwrap_or_default();
            if items.is_empty() {
                break;
            }
            for r in items {
                let tag = r["tag_name"].as_str().unwrap_or_default();
                if !tag.starts_with(TAG_PREFIX) || r["draft"].as_bool().unwrap_or(false) {
                    continue;
                }
                let assets = r["assets"].as_array().cloned().unwrap_or_default();
                let Some(asset) = assets.iter().find(|a| a["name"].as_str().is_some_and(|n| n.ends_with(".zip"))) else { continue };
                backups.push(json!({
                    "asset_id": asset["id"],
                    "file": asset["name"],
                    "size_bytes": asset["size"],
                    "title": r["name"],
                    "tag": tag,
                    "created_at": r["published_at"],
                }));
            }
        }
        backups.sort_by(|a, b| b["tag"].as_str().cmp(&a["tag"].as_str()));
        Ok(backups)
    }

    /// Downloads one release asset (the backup ZIP) to `target`.
    pub async fn download_asset(&self, asset_id: u64, target: &Path) -> Result<u64, String> {
        // GitHub answers with a redirect to its file storage; reqwest drops the token on that cross-host redirect
        let req = self
            .client
            .get(format!("{}/repos/{}/releases/assets/{}", GITHUB_API, self.repo, asset_id))
            .bearer_auth(&self.token)
            .header(reqwest::header::ACCEPT, "application/octet-stream")
            .header(reqwest::header::USER_AGENT, "rustwebshop-backup")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .timeout(std::time::Duration::from_secs(3 * 3600));
        let mut resp = req.send().await.map_err(|e| format!("GitHub is not reachable (download the backup): {}", e))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            return Err(self.error(status, &body, "download the backup"));
        }
        let mut file = tokio::fs::File::create(target).await.map_err(|e| format!("Cannot store the download: {}", e))?;
        let mut size = 0u64;
        while let Some(chunk) = resp.chunk().await.map_err(|e| format!("Downloading the backup was interrupted: {}", e))? {
            size += chunk.len() as u64;
            if size > MAX_ASSET_BYTES {
                return Err("The download is larger than any backup can be.".to_string());
            }
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await.map_err(|e| format!("Cannot store the download: {}", e))?;
        }
        tokio::io::AsyncWriteExt::flush(&mut file).await.map_err(|e| format!("Cannot store the download: {}", e))?;
        Ok(size)
    }
}

const README_MD: &str = "# Shop backups

This private repository stores the automatic backups of the web shop.
Every backup is a **release** (see *Releases* on the right) with one ZIP file containing the
complete database dump (`database.sql`) and the uploaded files (`uploads/`).

Keep this repository **private** — the backups contain customer data.
To restore a backup open the shop admin → *Settings → Export & Backups* → *Restore a Backup*
(details in `BACKUP.md` of the shop software).
";

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn slots_follow_the_frequency() {
        // 2026-10-25 is a Sunday
        assert_eq!(last_slot("daily", 2, t("2026-10-25T01:00:00Z")), t("2026-10-24T02:00:00Z"));
        assert_eq!(last_slot("daily", 2, t("2026-10-25T02:00:00Z")), t("2026-10-25T02:00:00Z"));
        assert_eq!(last_slot("weekly", 2, t("2026-10-28T12:00:00Z")), t("2026-10-25T02:00:00Z"));
        assert_eq!(last_slot("weekly", 2, t("2026-10-25T01:00:00Z")), t("2026-10-18T02:00:00Z"));
        assert_eq!(last_slot("monthly", 2, t("2026-10-25T12:00:00Z")), t("2026-10-01T02:00:00Z"));
        assert_eq!(last_slot("monthly", 2, t("2026-10-01T01:00:00Z")), t("2026-09-01T02:00:00Z"));
        assert_eq!(next_slot("monthly", 2, t("2026-12-15T00:00:00Z")), t("2027-01-01T02:00:00Z"));
        assert_eq!(next_slot("weekly", 2, t("2026-10-28T12:00:00Z")), t("2026-11-01T02:00:00Z"));
    }

    #[test]
    fn due_only_once_per_slot_and_retries_hourly() {
        let mut s = BackupSettings {
            enabled: true,
            github_repo: "me/backups".into(),
            github_token: "token".into(),
            frequency: "daily".into(),
            backup_hour_utc: 2,
            keep_count: 30,
            include_media: true,
            token_expires_at: None,
            last_attempt_at: None,
            last_success_at: None,
            last_status: None,
            last_message: None,
            last_file: None,
            last_size_bytes: None,
            last_url: None,
            failure_notified: false,
        };
        let now = t("2026-10-25T02:05:00Z");
        assert!(s.is_due(now), "never backed up");
        s.last_success_at = Some(t("2026-10-25T02:01:00Z"));
        s.last_attempt_at = s.last_success_at;
        s.last_status = Some("success".into());
        assert!(!s.is_due(now));
        assert!(s.is_due(t("2026-10-26T02:00:00Z")));
        s.last_status = Some("failed".into());
        s.last_attempt_at = Some(t("2026-10-26T02:00:00Z"));
        assert!(!s.is_due(t("2026-10-26T02:30:00Z")));
        assert!(s.is_due(t("2026-10-26T03:00:00Z")));
        s.enabled = false;
        assert!(!s.is_due(t("2026-10-27T03:00:00Z")));
    }

    #[test]
    fn repository_names_are_normalized() {
        assert_eq!(normalize_repo("me/shop-backups").unwrap(), "me/shop-backups");
        assert_eq!(normalize_repo(" https://github.com/me/shop-backups.git ").unwrap(), "me/shop-backups");
        assert_eq!(normalize_repo("github.com/me/shop.backups/").unwrap(), "me/shop.backups");
        assert!(normalize_repo("shop-backups").is_err());
        assert!(normalize_repo("me/../x").is_err());
        assert!(normalize_repo("me/back ups").is_err());
    }

    #[test]
    fn archive_contains_dump_and_uploads() {
        let dir = std::env::temp_dir().join(format!("backup-test-{}", crate::services::auth::random_token(6)));
        let _cleanup = TempDir(dir.clone());
        std::fs::create_dir_all(dir.join("uploads/sub")).unwrap();
        std::fs::write(dir.join("database.sql"), "CREATE TABLE x();").unwrap();
        std::fs::write(dir.join("uploads/logo.png"), [1u8, 2, 3]).unwrap();
        std::fs::write(dir.join("uploads/sub/manual.pdf"), b"pdf").unwrap();

        let zip_path = dir.join("out.zip");
        let count = write_archive(&zip_path, &dir.join("database.sql"), Some(&dir.join("uploads")), &json!({ "format_version": 1 })).unwrap();
        assert_eq!(count, 2);

        let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip_path).unwrap()).unwrap();
        let mut names: Vec<String> = archive.file_names().map(str::to_string).collect();
        names.sort();
        assert_eq!(names, ["RESTORE.txt", "backup-info.json", "database.sql", "uploads/logo.png", "uploads/sub/manual.pdf"]);
        let mut sql = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("database.sql").unwrap(), &mut sql).unwrap();
        assert_eq!(sql, "CREATE TABLE x();");
    }

    #[test]
    fn github_expiry_header_is_parsed() {
        assert_eq!(parse_github_time("2027-10-25 12:00:00 UTC"), Some(t("2027-10-25T12:00:00Z")));
        assert_eq!(parse_github_time("2027-10-25 12:00:00 +0200"), Some(t("2027-10-25T10:00:00Z")));
    }
}
