//! Shop updater: checks GitHub (git tags) for new releases, updates the installation and applies
//! domain settings. Stable installations follow `vX.Y.Z` tags; with `UPDATE_CHANNEL=beta` in `.env`
//! they also get beta tags `vX.Y.Z-beta.N`. It runs next to the shop with access to the project checkout and the Docker
//! daemon, has no published port and only accepts requests from the backend (shared token).

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{process::Command, sync::Mutex};
use tracing::{error, info};

/// Services rebuilt on update. The updater never restarts itself mid-job.
const APP_SERVICES: &[&str] = &["backend", "storefront", "admin"];

/// SSH public keys allowed to sign release tags (git `allowed_signers` format). Always read from the
/// *installed* commit, so a new release can only be installed when a key the current version trusts signed it.
const ALLOWED_SIGNERS_PATH: &str = "deploy/updater/allowed_signers";

/// Pre-update database dumps kept in `backups/` (older ones are deleted).
const KEEP_PRE_UPDATE_BACKUPS: usize = 10;

/// How long the updated backend may take to become healthy (migrations included) before rolling back.
const HEALTH_TIMEOUT_SECS: u64 = 300;

#[derive(Default, Clone, Serialize)]
struct Job {
    kind: String,
    running: bool,
    success: Option<bool>,
    started_at: Option<String>,
    finished_at: Option<String>,
    log: Vec<String>,
}

struct AppState {
    repo: PathBuf,
    token: String,
    remote: String,
    job: Mutex<Job>,
}

type Shared = Arc<AppState>;
type ApiResult = Result<Json<JsonValue>, (StatusCode, String)>;

// ---------------------------------------------------------------- versions

/// `X.Y.Z` or the beta `X.Y.Z-beta.N` (`.3` is the beta number; a release sorts after its betas).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Version(u64, u64, u64, Option<u64>);

impl Version {
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('v');
        let (core, beta) = match s.split_once('-') {
            Some((core, pre)) => (core, Some(pre.strip_prefix("beta.")?.parse::<u64>().ok()?)),
            None => (s, None),
        };
        let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
        let v = Version(parts.next()??, parts.next()??, parts.next()??, beta);
        parts.next().is_none().then_some(v)
    }

    fn is_beta(&self) -> bool {
        self.3.is_some()
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0, self.1, self.2)
            .cmp(&(other.0, other.1, other.2))
            .then_with(|| match (self.3, other.3) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(a), Some(b)) => a.cmp(&b),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)?;
        match self.3 {
            Some(n) => write!(f, "-beta.{}", n),
            None => Ok(()),
        }
    }
}

/// Which releases an installation follows (`UPDATE_CHANNEL` in `.env`, stable when missing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    Stable,
    Beta,
}

impl Channel {
    fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "" | "stable" => Some(Channel::Stable),
            "beta" => Some(Channel::Beta),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Beta => "beta",
        }
    }
}

fn latest_tag(tags: &str, channel: Channel) -> Option<Version> {
    tags.lines()
        .filter_map(Version::parse)
        .filter(|v| channel == Channel::Beta || !v.is_beta())
        .max()
}

// ---------------------------------------------------------------- .env handling

/// Sets `KEY=value` lines in a dotenv file, keeping every other line untouched.
fn upsert_env(content: &str, updates: &[(&str, String)]) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    for (key, value) in updates {
        let line = format!("{}={}", key, value);
        match lines.iter().position(|l| l.trim_start().starts_with(&format!("{}=", key))) {
            Some(i) => lines[i] = line,
            None => lines.push(line),
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn read_env_value(content: &str, key: &str) -> String {
    content
        .lines()
        .find_map(|l| l.trim().strip_prefix(&format!("{}=", key)))
        .map(|v| v.trim().trim_matches('"').to_string())
        .unwrap_or_default()
}

fn valid_domain(d: &str) -> bool {
    !d.is_empty()
        && d.len() <= 253
        && d.contains('.')
        && d.split('.').all(|l| !l.is_empty() && !l.starts_with('-') && !l.ends_with('-'))
        && d.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

// ---------------------------------------------------------------- command helpers

async fn run(repo: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo)
        .output()
        .await
        .map_err(|e| format!("could not start {}: {}", program, e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(format!("{}{}", stdout, stderr))
    } else {
        Err(format!("{} {} failed: {}{}", program, args.join(" "), stderr.trim(), stdout.trim()))
    }
}

fn compose_args<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["compose", "--project-directory", ".", "-f", "docker-compose.yml"];
    args.extend_from_slice(extra);
    args
}

async fn log(state: &Shared, line: impl Into<String>) {
    let line = format!("[{}] {}", chrono::Utc::now().format("%H:%M:%S"), line.into());
    info!("{}", line);
    let mut job = state.job.lock().await;
    job.log.push(line);
    if job.log.len() > 400 {
        job.log.remove(0);
    }
}

async fn step(state: &Shared, title: &str, program: &str, args: &[&str]) -> Result<String, String> {
    log(state, format!("→ {}", title)).await;
    let out = run(&state.repo, program, args).await;
    match &out {
        Ok(text) => {
            let tail: Vec<&str> = text.lines().rev().take(15).collect();
            for l in tail.into_iter().rev() {
                log(state, format!("   {}", l)).await;
            }
        }
        Err(e) => log(state, format!("   ERROR: {}", e)).await,
    }
    out
}

async fn current_channel(repo: &Path) -> Channel {
    let env = tokio::fs::read_to_string(repo.join(".env")).await.unwrap_or_default();
    Channel::parse(&read_env_value(&env, "UPDATE_CHANNEL")).unwrap_or(Channel::Stable)
}

/// Non-comment lines of an `allowed_signers` file.
fn signer_lines(content: &str) -> Vec<&str> {
    content.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

fn env_flag(content: &str, key: &str) -> bool {
    matches!(read_env_value(content, key).to_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

/// Refuses a release tag that is not signed by a key the installed version trusts.
/// Without trusted keys the tag is accepted with a warning, unless `UPDATE_REQUIRE_SIGNED=true` in `.env`.
async fn verify_release_tag(state: &Shared, tag: &str) -> Result<(), String> {
    let signers = run(&state.repo, "git", &["show", &format!("HEAD:{}", ALLOWED_SIGNERS_PATH)]).await.unwrap_or_default();
    let signers = signer_lines(&signers);
    if signers.is_empty() {
        let env = tokio::fs::read_to_string(state.repo.join(".env")).await.unwrap_or_default();
        if env_flag(&env, "UPDATE_REQUIRE_SIGNED") {
            return Err(format!(
                "UPDATE_REQUIRE_SIGNED is on, but the installed version has no trusted signing keys ({}). Nothing was changed.",
                ALLOWED_SIGNERS_PATH
            ));
        }
        log(state, format!("   WARNING: {} is not signature-checked (no trusted keys in {})", tag, ALLOWED_SIGNERS_PATH)).await;
        return Ok(());
    }
    let file = std::env::temp_dir().join(format!("allowed_signers-{}", uuid::Uuid::new_v4().simple()));
    tokio::fs::write(&file, format!("{}\n", signers.join("\n"))).await.map_err(|e| e.to_string())?;
    let signers_cfg = format!("gpg.ssh.allowedSignersFile={}", file.display());
    let result = step(state, &format!("Verifying the signature of {}", tag), "git", &["-c", "gpg.format=ssh", "-c", &signers_cfg, "verify-tag", tag]).await;
    let _ = tokio::fs::remove_file(&file).await;
    result.map(|_| ()).map_err(|_| format!("{} is not signed by a trusted key — refusing to install it. Nothing was changed.", tag))
}

/// Waits until the backend answers `/health` (database reachable, migrations done).
async fn wait_for_backend(state: &Shared) -> bool {
    log(state, "→ Waiting for the shop to come back up").await;
    let started = std::time::Instant::now();
    while started.elapsed().as_secs() < HEALTH_TIMEOUT_SECS {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let probe = compose_args(&["exec", "-T", "backend", "curl", "-fsS", "-o", "/dev/null", "http://127.0.0.1:8000/health"]);
        if run(&state.repo, "docker", &probe).await.is_ok() {
            log(state, format!("   backend healthy after {} s", started.elapsed().as_secs())).await;
            return true;
        }
    }
    false
}

/// Writes the dump readable by its owner only and removes the oldest pre-update dumps.
async fn store_backup(dir: &Path, file: &Path, data: &[u8]) -> Result<(), String> {
    tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).await;
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut out = options.open(file).await.map_err(|e| format!("could not write backup: {}", e))?;
    tokio::io::AsyncWriteExt::write_all(&mut out, data).await.map_err(|e| format!("could not write backup: {}", e))?;

    let mut dumps = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("pre-update-") && name.ends_with(".sql") {
                let modified = entry.metadata().await.and_then(|m| m.modified()).ok();
                dumps.push((modified, entry.path()));
            }
        }
    }
    dumps.sort();
    let excess = dumps.len().saturating_sub(KEEP_PRE_UPDATE_BACKUPS);
    for (_, path) in dumps.into_iter().take(excess) {
        let _ = tokio::fs::remove_file(path).await;
    }
    Ok(())
}

async fn current_version(repo: &Path) -> Option<Version> {
    tokio::fs::read_to_string(repo.join("VERSION")).await.ok().and_then(|v| Version::parse(&v))
}

async fn start_job(state: &Shared, kind: &str) -> Result<(), (StatusCode, String)> {
    let mut job = state.job.lock().await;
    if job.running {
        return Err((StatusCode::CONFLICT, format!("Another task ({}) is still running", job.kind)));
    }
    *job = Job {
        kind: kind.to_string(),
        running: true,
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };
    Ok(())
}

async fn finish_job(state: &Shared, result: Result<(), String>) {
    if let Err(e) = &result {
        error!("Job failed: {}", e);
        log(state, format!("✗ Failed: {}", e)).await;
    } else {
        log(state, "✓ Finished successfully").await;
    }
    let mut job = state.job.lock().await;
    job.running = false;
    job.success = Some(result.is_ok());
    job.finished_at = Some(chrono::Utc::now().to_rfc3339());
}

// ---------------------------------------------------------------- handlers

fn authorize(state: &Shared, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    let provided = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default();
    // Constant-time comparison
    let ok = provided.len() == state.token.len()
        && provided.bytes().zip(state.token.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0;
    if ok { Ok(()) } else { Err((StatusCode::UNAUTHORIZED, "invalid updater token".to_string())) }
}

async fn status(State(state): State<Shared>, headers: HeaderMap) -> ApiResult {
    authorize(&state, &headers)?;
    let commit = run(&state.repo, "git", &["rev-parse", "--short", "HEAD"]).await.unwrap_or_default();
    let job = state.job.lock().await.clone();
    Ok(Json(json!({
        "current_version": current_version(&state.repo).await.map(|v| v.to_string()),
        "current_commit": commit.trim(),
        "channel": current_channel(&state.repo).await.as_str(),
        "remote": state.remote,
        "job": job
    })))
}

/// Fetches tags from GitHub and reports whether a newer release exists.
async fn check(State(state): State<Shared>, headers: HeaderMap) -> ApiResult {
    authorize(&state, &headers)?;
    run(&state.repo, "git", &["fetch", "--tags", "--force", "--quiet", &state.remote])
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Could not reach the update server: {}", e)))?;
    let tags = run(&state.repo, "git", &["tag", "--list", "v*"]).await.unwrap_or_default();
    let current = current_version(&state.repo).await;
    let channel = current_channel(&state.repo).await;
    let latest = latest_tag(&tags, channel);
    let notes = match &latest {
        Some(v) => run(&state.repo, "git", &["tag", "-l", "--format=%(contents:subject)%0a%0a%(contents:body)", &format!("v{}", v)]).await.unwrap_or_default(),
        None => String::new(),
    };
    Ok(Json(json!({
        "current_version": current.as_ref().map(|v| v.to_string()),
        "latest_version": latest.as_ref().map(|v| v.to_string()),
        "update_available": matches!((&current, &latest), (Some(c), Some(l)) if l > c),
        "channel": channel.as_str(),
        "release_notes": notes.trim()
    })))
}

async fn update(State(state): State<Shared>, headers: HeaderMap) -> ApiResult {
    authorize(&state, &headers)?;
    start_job(&state, "update").await?;
    let job_state = state.clone();
    tokio::spawn(async move {
        let result = perform_update(&job_state).await;
        finish_job(&job_state, result).await;
    });
    Ok(Json(json!({ "started": true })))
}

async fn perform_update(state: &Shared) -> Result<(), String> {
    step(state, "Fetching releases", "git", &["fetch", "--tags", "--force", &state.remote]).await?;
    let tags = run(&state.repo, "git", &["tag", "--list", "v*"]).await?;
    let current = current_version(&state.repo).await.ok_or("VERSION file missing or invalid")?;
    let channel = current_channel(&state.repo).await;
    let target = latest_tag(&tags, channel).ok_or("No release tags (vX.Y.Z) found")?;
    if target <= current {
        log(state, format!("Already up to date ({}, {} channel)", current, channel.as_str())).await;
        return Ok(());
    }
    log(state, format!("Updating {} → {} ({} channel)", current, target, channel.as_str())).await;

    let dirty = run(&state.repo, "git", &["status", "--porcelain", "--untracked-files=no"]).await?;
    if !dirty.trim().is_empty() {
        return Err(format!("The installation has local code changes, refusing to overwrite them:\n{}", dirty.trim()));
    }
    let previous = run(&state.repo, "git", &["rev-parse", "HEAD"]).await?.trim().to_string();
    let tag = format!("v{}", target);
    verify_release_tag(state, &tag).await?;

    // Database backup before anything changes
    let backup_dir = state.repo.join("backups");
    let backup = backup_dir.join(format!("pre-update-{}-{}.sql", current, chrono::Utc::now().format("%Y%m%d-%H%M%S")));
    log(state, "→ Backing up the database").await;
    let dump = Command::new("docker")
        .args(compose_args(&["exec", "-T", "db", "pg_dump", "-U", "shop_user", "shop_db"]))
        .current_dir(&state.repo)
        .output()
        .await
        .map_err(|e| format!("backup failed: {}", e))?;
    if !dump.status.success() || dump.stdout.is_empty() {
        return Err(format!("Database backup failed: {}", String::from_utf8_lossy(&dump.stderr)));
    }
    store_backup(&backup_dir, &backup, &dump.stdout).await?;
    log(state, format!("   saved {}", backup.display())).await;

    if let Err(e) = step(state, &format!("Switching code to {}", tag), "git", &["-c", "advice.detachedHead=false", "checkout", "--force", &tag]).await {
        return Err(format!("could not check out {}: {}", tag, e));
    }

    let mut args = vec!["up", "-d", "--build"];
    args.extend_from_slice(APP_SERVICES);
    if let Err(e) = step(state, "Building and restarting the shop (this can take several minutes)", "docker", &compose_args(&args)).await {
        log(state, "Build failed — restoring the previous code; the running shop was not replaced").await;
        let _ = step(state, "Rolling back", "git", &["-c", "advice.detachedHead=false", "checkout", "--force", &previous]).await;
        return Err(e);
    }
    if !wait_for_backend(state).await {
        // Each migration is transactional, so a failed one left the database unchanged; the previous code runs on it again.
        log(state, format!("The new version did not become healthy within {} s — going back to {}", HEALTH_TIMEOUT_SECS, current)).await;
        step(state, "Restoring the previous code", "git", &["-c", "advice.detachedHead=false", "checkout", "--force", &previous]).await?;
        step(state, "Rebuilding the previous version", "docker", &compose_args(&args)).await?;
        let healthy = wait_for_backend(state).await;
        return Err(format!(
            "Update to {} failed and was rolled back to {}{}. The database backup from before the update is {}.",
            target,
            current,
            if healthy { "" } else { " (the shop is still not healthy — check `docker compose logs backend`)" },
            backup.display()
        ));
    }
    log(state, format!("Shop updated to {}. Database migrations ran automatically on start.", target)).await;
    Ok(())
}

async fn job(State(state): State<Shared>, headers: HeaderMap) -> ApiResult {
    authorize(&state, &headers)?;
    let job = state.job.lock().await.clone();
    Ok(Json(serde_json::to_value(job).unwrap_or_default()))
}

#[derive(Deserialize)]
struct ChannelConfig {
    channel: String,
}

/// Switches between stable releases and beta releases (stored as `UPDATE_CHANNEL` in `.env`).
/// Nothing is installed here; the next check/update uses the new channel.
async fn put_channel(State(state): State<Shared>, headers: HeaderMap, Json(cfg): Json<ChannelConfig>) -> ApiResult {
    authorize(&state, &headers)?;
    let channel = Channel::parse(&cfg.channel)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "The update channel is either stable or beta".to_string()))?;
    if state.job.lock().await.running {
        return Err((StatusCode::CONFLICT, "Wait until the running task has finished".to_string()));
    }
    let path = state.repo.join(".env");
    let env = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    tokio::fs::write(&path, upsert_env(&env, &[("UPDATE_CHANNEL", channel.as_str().to_string())]))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("could not write .env: {}", e)))?;
    info!("Update channel set to {}", channel.as_str());
    Ok(Json(json!({ "channel": channel.as_str() })))
}

#[derive(Deserialize)]
struct DomainConfig {
    shop_domain: String,
    admin_domain: String,
    https_proxy: bool,
}

async fn get_config(State(state): State<Shared>, headers: HeaderMap) -> ApiResult {
    authorize(&state, &headers)?;
    let env = tokio::fs::read_to_string(state.repo.join(".env")).await.unwrap_or_default();
    Ok(Json(json!({
        "shop_domain": read_env_value(&env, "SHOP_DOMAIN"),
        "admin_domain": read_env_value(&env, "ADMIN_DOMAIN"),
        "https_proxy": read_env_value(&env, "COMPOSE_PROFILES").split(',').any(|p| p.trim() == "proxy"),
        "shop_public_url": read_env_value(&env, "SHOP_PUBLIC_URL"),
        "admin_public_url": read_env_value(&env, "ADMIN_PUBLIC_URL"),
    })))
}

/// Writes the domains into `.env` and recreates the services so the new URLs take effect.
async fn put_config(State(state): State<Shared>, headers: HeaderMap, Json(cfg): Json<DomainConfig>) -> ApiResult {
    authorize(&state, &headers)?;
    let shop = cfg.shop_domain.trim().to_lowercase();
    let admin = cfg.admin_domain.trim().to_lowercase();
    if !valid_domain(&shop) || !valid_domain(&admin) {
        return Err((StatusCode::BAD_REQUEST, "Enter both domains as plain host names, e.g. shop.example.com and admin.example.com".to_string()));
    }
    if shop == admin {
        return Err((StatusCode::BAD_REQUEST, "Shop and admin need different (sub)domains".to_string()));
    }
    start_job(&state, "apply-domains").await?;

    let job_state = state.clone();
    tokio::spawn(async move {
        let result = async {
            let path = job_state.repo.join(".env");
            let env = tokio::fs::read_to_string(&path).await.unwrap_or_default();
            let updates = [
                ("SHOP_DOMAIN", shop.clone()),
                ("ADMIN_DOMAIN", admin.clone()),
                ("SHOP_PUBLIC_URL", format!("https://{}", shop)),
                ("ADMIN_PUBLIC_URL", format!("https://{}", admin)),
                ("COMPOSE_PROFILES", if cfg.https_proxy { "proxy".to_string() } else { String::new() }),
                // With the built-in proxy the app ports only listen locally
                ("PUBLISH_ADDR", if cfg.https_proxy { "127.0.0.1".to_string() } else { "0.0.0.0".to_string() }),
            ];
            tokio::fs::write(&path, upsert_env(&env, &updates)).await.map_err(|e| format!("could not write .env: {}", e))?;
            log(&job_state, format!("Saved domains: shop={} admin={} (HTTPS proxy: {})", shop, admin, cfg.https_proxy)).await;

            let mut services: Vec<&str> = APP_SERVICES.to_vec();
            if cfg.https_proxy {
                services.push("proxy");
            }
            let mut args = vec!["up", "-d", "--build"];
            args.extend(services);
            step(&job_state, "Restarting services with the new domains", "docker", &compose_args(&args)).await?;
            if !cfg.https_proxy {
                let _ = step(&job_state, "Stopping the built-in proxy", "docker", &compose_args(&["stop", "proxy"])).await;
            }
            Ok(())
        }
        .await;
        finish_job(&job_state, result).await;
    });
    Ok(Json(json!({ "started": true })))
}

// ---------------------------------------------------------------- startup

/// Token shared with the backend through a private volume (generated once).
async fn load_or_create_token(dir: &Path) -> std::io::Result<String> {
    let file = dir.join("updater_token");
    if let Ok(t) = tokio::fs::read_to_string(&file).await {
        if t.trim().len() >= 32 {
            return Ok(t.trim().to_string());
        }
    }
    tokio::fs::create_dir_all(dir).await?;
    let token: String = (0..3).map(|_| uuid::Uuid::new_v4().simple().to_string()).collect();
    tokio::fs::write(&file, &token).await?;
    Ok(token)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let repo = PathBuf::from(std::env::var("REPO_DIR").unwrap_or_else(|_| "/repo".to_string()));
    let secrets = PathBuf::from(std::env::var("SECRETS_DIR").unwrap_or_else(|_| "/secrets".to_string()));
    let token = load_or_create_token(&secrets).await.expect("cannot create updater token");
    let remote = std::env::var("UPDATE_REMOTE").unwrap_or_else(|_| "origin".to_string());

    // The checkout belongs to the host user; allow git to operate on it
    let _ = run(&repo, "git", &["config", "--global", "--add", "safe.directory", &repo.to_string_lossy()]).await;
    // Windows checkouts may contain CRLF line endings: treat them as unchanged instead of blocking updates
    let _ = run(&repo, "git", &["config", "--global", "core.autocrlf", "input"]).await;

    let state = Arc::new(AppState { repo, token, remote, job: Mutex::new(Job::default()) });
    let app = Router::new()
        .route("/status", get(status))
        .route("/check", post(check))
        .route("/update", post(update))
        .route("/job", get(job))
        .route("/config", get(get_config).put(put_config))
        .route("/channel", axum::routing::put(put_channel))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 9000));
    info!("Updater listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind 9000");
    axum::serve(listener, app).await.expect("server error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_highest_semver_tag() {
        assert_eq!(latest_tag("v1.0.0\nv1.10.0\nv1.9.3\nfoo\nv2.0\n", Channel::Stable), Some(Version(1, 10, 0, None)));
        assert!(Version(1, 10, 0, None) > Version(1, 9, 9, None));
        assert_eq!(Version::parse("1.2.3\n"), Some(Version(1, 2, 3, None)));
        assert_eq!(Version::parse("1.2"), None);
    }

    #[test]
    fn beta_tags_only_on_the_beta_channel() {
        let tags = "v1.4.2\nv1.5.0-beta.1\nv1.5.0-beta.10\nv1.5.0-beta.2\nv1.5.0-rc.1\n";
        assert_eq!(latest_tag(tags, Channel::Stable), Some(Version(1, 4, 2, None)));
        assert_eq!(latest_tag(tags, Channel::Beta), Some(Version(1, 5, 0, Some(10))));
        // The release follows its betas, so beta installations move on to it
        let released = format!("{}v1.5.0\n", tags);
        assert_eq!(latest_tag(&released, Channel::Beta), Some(Version(1, 5, 0, None)));
        assert!(Version(1, 5, 0, Some(3)) > Version(1, 4, 9, None));
        assert_eq!(Version::parse("v1.5.0-beta.2").unwrap().to_string(), "1.5.0-beta.2");
        assert_eq!(Version::parse("1.5.0-beta"), None);
        assert_eq!(Channel::parse(""), Some(Channel::Stable));
        assert_eq!(Channel::parse("Beta"), Some(Channel::Beta));
        assert_eq!(Channel::parse("nightly"), None);
    }

    #[test]
    fn env_upsert_keeps_other_lines() {
        let out = upsert_env("# c\nDB_PASSWORD=x\nSHOP_DOMAIN=old\n", &[("SHOP_DOMAIN", "new.example".into()), ("ADMIN_DOMAIN", "a.example".into())]);
        assert_eq!(out, "# c\nDB_PASSWORD=x\nSHOP_DOMAIN=new.example\nADMIN_DOMAIN=a.example\n");
        assert_eq!(read_env_value(&out, "SHOP_DOMAIN"), "new.example");
    }

    #[test]
    fn reads_trusted_signers_and_flags() {
        let file = "# trusted release keys\n\nreleases@example.com namespaces=\"git\" ssh-ed25519 AAAAC3Nza\n  # old key\n";
        assert_eq!(signer_lines(file), vec!["releases@example.com namespaces=\"git\" ssh-ed25519 AAAAC3Nza"]);
        assert!(signer_lines("# only comments\n\n").is_empty());
        assert!(env_flag("UPDATE_REQUIRE_SIGNED=true\n", "UPDATE_REQUIRE_SIGNED"));
        assert!(env_flag("UPDATE_REQUIRE_SIGNED=\"1\"\n", "UPDATE_REQUIRE_SIGNED"));
        assert!(!env_flag("UPDATE_REQUIRE_SIGNED=\n", "UPDATE_REQUIRE_SIGNED"));
        assert!(!env_flag("", "UPDATE_REQUIRE_SIGNED"));
    }

    #[tokio::test]
    async fn pre_update_backups_are_private_and_rotated() {
        let dir = std::env::temp_dir().join(format!("updater-backups-{}", uuid::Uuid::new_v4().simple()));
        for i in 0..(KEEP_PRE_UPDATE_BACKUPS + 3) {
            store_backup(&dir, &dir.join(format!("pre-update-1.0.0-2026010{:02}.sql", i)), b"dump").await.unwrap();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        tokio::fs::write(dir.join("manual.sql"), b"keep").await.unwrap();
        let mut names: Vec<String> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
        names.sort();
        assert_eq!(names.len(), KEEP_PRE_UPDATE_BACKUPS + 1);
        assert!(names.contains(&"manual.sql".to_string()));
        assert!(!names.contains(&"pre-update-1.0.0-202601000.sql".to_string()));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.join(&names[0])).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validates_domains() {
        assert!(valid_domain("shop.example.com"));
        assert!(!valid_domain("localhost"));
        assert!(!valid_domain("bad domain.com"));
        assert!(!valid_domain("x.com\nEVIL=1"));
        assert!(!valid_domain("-a.example.com"));
    }
}
