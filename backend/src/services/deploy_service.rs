//! Admin deploy: host `jheffmedia --update` via control dir (or local binary).
//!
//! App container has no docker.sock — production path is a bind-mounted control
//! dir + systemd path unit on the LXC host. Local `cargo run` → 503.

use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
};
use uuid::Uuid;

const CTL_ENV: &str = "YEW_FULLSTACK_DEPLOY_CTL";
const DEFAULT_CTL: &str = "/data/deploy-ctl";
const LOG_TAIL_BYTES: usize = 48 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployChannel {
    Main,
    Testing,
}

impl DeployChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Testing => "testing",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "main" | "prod" | "production" => Some(Self::Main),
            "testing" | "dev" => Some(Self::Testing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployStatus {
    pub build_id: String,
    pub channel_hint: String,
    pub cli_available: bool,
    pub control_dir: bool,
    pub available: bool,
    pub autoupdate_enabled: Option<bool>,
    pub autoupdate_channel: Option<String>,
    pub active_job_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployJobView {
    pub id: String,
    pub channel: String,
    pub state: String,
    pub log_tail: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone)]
struct LocalJob {
    id: String,
    channel: String,
    state: String,
    log: String,
    exit_code: Option<i32>,
}

static LOCAL: OnceLock<Mutex<Option<LocalJob>>> = OnceLock::new();

fn local_slot() -> &'static Mutex<Option<LocalJob>> {
    LOCAL.get_or_init(|| Mutex::new(None))
}

pub fn control_dir() -> PathBuf {
    PathBuf::from(env::var(CTL_ENV).unwrap_or_else(|_| DEFAULT_CTL.into()))
}

fn find_cli() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "/usr/local/bin/jheffmedia",
        "/opt/jheffmedia-site/install/common/bin/jheffmedia",
    ];
    for p in CANDIDATES {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    for key in ["SITE_ROOT", "INSTALL_DIR"] {
        if let Ok(root) = env::var(key) {
            let p = PathBuf::from(root).join("install/common/bin/jheffmedia");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn ctl_writable(ctl: &Path) -> bool {
    if !ctl.is_dir() {
        return false;
    }
    let probe = ctl.join(".w");
    match fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&probe)
    {
        Ok(mut f) => {
            let _ = f.write_all(b"1");
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn read_kv_file(path: &Path) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let Ok(text) = fs::read_to_string(path) else {
        return map;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

fn write_kv(path: &Path, pairs: &[(&str, &str)]) -> std::io::Result<()> {
    let mut body = String::new();
    for (k, v) in pairs {
        body.push_str(k);
        body.push('=');
        body.push_str(v);
        body.push('\n');
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn read_channel_hint(ctl: &Path) -> String {
    let from_ctl = read_kv_file(&ctl.join("status"))
        .get("CHANNEL")
        .cloned()
        .unwrap_or_default();
    if !from_ctl.is_empty() {
        return from_ctl;
    }
    // Host may drop last channel into install-version sibling via agent.
    let hint = ctl.join("channel-hint");
    fs::read_to_string(hint)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn read_autoupdate(ctl: &Path) -> (Option<bool>, Option<String>) {
    let map = read_kv_file(&ctl.join("autoupdate-status"));
    if map.is_empty() {
        return (None, None);
    }
    let enabled = map.get("enabled").map(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let channel = map.get("channel").cloned().filter(|s| !s.is_empty());
    (enabled, channel)
}

fn file_job_running(ctl: &Path) -> Option<String> {
    let map = read_kv_file(&ctl.join("status"));
    let state = map.get("STATE").map(String::as_str).unwrap_or("");
    if state == "running" || state == "queued" {
        map.get("JOB_ID").cloned().filter(|s| !s.is_empty())
    } else {
        None
    }
}

fn local_job_running() -> Option<String> {
    let guard = local_slot().lock().ok()?;
    let job = guard.as_ref()?;
    if job.state == "running" || job.state == "queued" {
        Some(job.id.clone())
    } else {
        None
    }
}

fn active_job_id(ctl: &Path) -> Option<String> {
    file_job_running(ctl).or_else(local_job_running)
}

fn tail_bytes(path: &Path, max: usize) -> String {
    let Ok(data) = fs::read(path) else {
        return String::new();
    };
    if data.len() <= max {
        String::from_utf8_lossy(&data).into_owned()
    } else {
        String::from_utf8_lossy(&data[data.len() - max..]).into_owned()
    }
}

pub fn status_snapshot() -> DeployStatus {
    let build_id = env::var("YEW_FULLSTACK_BUILD_ID").unwrap_or_default();
    let ctl = control_dir();
    let control = ctl_writable(&ctl);
    let cli = find_cli().is_some();
    let available = control || cli;
    let (au_en, au_ch) = read_autoupdate(&ctl);
    let message = if available {
        if control {
            "Host deploy control ready.".into()
        } else {
            "Local jheffmedia binary found.".into()
        }
    } else {
        "Deploy CLI not available — LXC/jheffmedia install with var/deploy mount required (not local cargo run)."
            .into()
    };
    DeployStatus {
        build_id,
        channel_hint: read_channel_hint(&ctl),
        cli_available: cli,
        control_dir: control,
        available,
        autoupdate_enabled: au_en,
        autoupdate_channel: au_ch,
        active_job_id: active_job_id(&ctl),
        message,
    }
}

#[derive(Debug)]
pub enum StartError {
    Unavailable(String),
    Busy(String),
    Io(String),
}

pub fn start_update(channel: DeployChannel) -> Result<String, StartError> {
    let ctl = control_dir();
    if let Some(id) = active_job_id(&ctl) {
        return Err(StartError::Busy(id));
    }

    let job_id = Uuid::new_v4().to_string();

    if ctl_writable(&ctl) {
        write_kv(
            &ctl.join("status"),
            &[
                ("JOB_ID", &job_id),
                ("CHANNEL", channel.as_str()),
                ("STATE", "queued"),
                ("EXIT", ""),
            ],
        )
        .map_err(|e| StartError::Io(e.to_string()))?;
        // Truncate log for this job.
        fs::write(ctl.join("log"), format!("[{}] queued channel={}\n", job_id, channel.as_str()))
            .map_err(|e| StartError::Io(e.to_string()))?;
        // PathChanged trigger — rewrite request atomically.
        write_kv(
            &ctl.join("request"),
            &[("JOB_ID", &job_id), ("CHANNEL", channel.as_str())],
        )
        .map_err(|e| StartError::Io(e.to_string()))?;
        return Ok(job_id);
    }

    let Some(cli) = find_cli() else {
        return Err(StartError::Unavailable(
            "Deploy CLI not available — no control dir and no jheffmedia binary.".into(),
        ));
    };

    {
        let mut guard = local_slot()
            .lock()
            .map_err(|_| StartError::Io("local job lock poisoned".into()))?;
        *guard = Some(LocalJob {
            id: job_id.clone(),
            channel: channel.as_str().into(),
            state: "running".into(),
            log: format!("[{}] starting {} {}\n", job_id, cli.display(), channel.as_str()),
            exit_code: None,
        });
    }

    let job_id_bg = job_id.clone();
    thread::spawn(move || {
        let mut cmd = Command::new(&cli);
        cmd.arg("--update");
        if channel == DeployChannel::Testing {
            cmd.arg("-d");
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let (state, exit, extra) = match cmd.output() {
            Ok(out) => {
                let mut log = String::new();
                log.push_str(&String::from_utf8_lossy(&out.stdout));
                log.push_str(&String::from_utf8_lossy(&out.stderr));
                let code = out.status.code().unwrap_or(1);
                let state = if out.status.success() { "ok" } else { "failed" };
                (state.to_string(), Some(code), log)
            }
            Err(err) => ("failed".into(), Some(1), format!("spawn failed: {err}\n")),
        };
        if let Ok(mut guard) = local_slot().lock() {
            if let Some(job) = guard.as_mut() {
                if job.id == job_id_bg {
                    job.state = state;
                    job.exit_code = exit;
                    job.log.push_str(&extra);
                    if job.log.len() > LOG_TAIL_BYTES {
                        job.log = job.log[job.log.len() - LOG_TAIL_BYTES..].to_string();
                    }
                }
            }
        }
    });

    Ok(job_id)
}

pub fn job_view(id: &str) -> Option<DeployJobView> {
    let ctl = control_dir();
    let map = read_kv_file(&ctl.join("status"));
    if map.get("JOB_ID").map(String::as_str) == Some(id) {
        return Some(DeployJobView {
            id: id.into(),
            channel: map
                .get("CHANNEL")
                .cloned()
                .unwrap_or_else(|| read_channel_hint(&ctl)),
            state: map
                .get("STATE")
                .cloned()
                .unwrap_or_else(|| "unknown".into()),
            log_tail: tail_bytes(&ctl.join("log"), LOG_TAIL_BYTES),
            exit_code: map
                .get("EXIT")
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse().ok()),
        });
    }

    let guard = local_slot().lock().ok()?;
    let job = guard.as_ref()?;
    if job.id != id {
        return None;
    }
    Some(DeployJobView {
        id: job.id.clone(),
        channel: job.channel.clone(),
        state: job.state.clone(),
        log_tail: job.log.clone(),
        exit_code: job.exit_code,
    })
}

/// Latest job if any (for UI after container restart mid-update).
pub fn latest_job() -> Option<DeployJobView> {
    let ctl = control_dir();
    let map = read_kv_file(&ctl.join("status"));
    if let Some(id) = map.get("JOB_ID").filter(|s| !s.is_empty()) {
        return job_view(id);
    }
    let guard = local_slot().lock().ok()?;
    let job = guard.as_ref()?;
    Some(DeployJobView {
        id: job.id.clone(),
        channel: job.channel.clone(),
        state: job.state.clone(),
        log_tail: job.log.clone(),
        exit_code: job.exit_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_parse_enum_only() {
        assert_eq!(DeployChannel::parse("main"), Some(DeployChannel::Main));
        assert_eq!(DeployChannel::parse("testing"), Some(DeployChannel::Testing));
        assert_eq!(DeployChannel::parse("dev"), Some(DeployChannel::Testing));
        assert!(DeployChannel::parse("rm -rf /").is_none());
        assert!(DeployChannel::parse("").is_none());
    }
}
