//! Runtime DB probes + production readiness hints for admin diagnostics.
use bson::doc;
use mongodb::Database;
use serde::Serialize;
use std::env;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DbTestResult {
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DbTests {
    pub ping: DbTestResult,
    pub list_collections: DbTestResult,
    pub users_readable: DbTestResult,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProdWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSnapshot {
    pub db_reachable: bool,
    pub db_tests: DbTests,
    pub feed_provider: String,
    pub rust_log: String,
    pub uptime_secs: u64,
    pub version: String,
    pub build_id: String,
    pub compress_enabled: bool,
    pub warnings: Vec<ProdWarning>,
}

pub async fn run_db_tests(db: &Database) -> DbTests {
    let ping = match db.run_command(doc! { "ping": 1 }).await {
        Ok(_) => DbTestResult {
            ok: true,
            detail: "ping ok".into(),
        },
        Err(err) => DbTestResult {
            ok: false,
            detail: shorten_err(&err.to_string()),
        },
    };

    let list_collections = match db.list_collection_names().await {
        Ok(names) => DbTestResult {
            ok: true,
            detail: format!("{} collection(s)", names.len()),
        },
        Err(err) => DbTestResult {
            ok: false,
            detail: shorten_err(&err.to_string()),
        },
    };

    let users = db.collection::<bson::Document>("users");
    let users_readable = match users.estimated_document_count().await {
        Ok(n) => DbTestResult {
            ok: true,
            detail: format!("users≈{n}"),
        },
        Err(err) => DbTestResult {
            ok: false,
            detail: shorten_err(&err.to_string()),
        },
    };

    DbTests {
        ping,
        list_collections,
        users_readable,
    }
}

pub fn production_warnings(build_id: &str, compress_enabled: bool) -> Vec<ProdWarning> {
    let mut out = Vec::new();
    if build_id.trim().is_empty() {
        out.push(ProdWarning {
            code: "NO_BUILD_ID".into(),
            message: "YEW_FULLSTACK_BUILD_ID empty — set git SHA on deploy for release identity."
                .into(),
        });
    }
    if !compress_enabled {
        out.push(ProdWarning {
            code: "NO_COMPRESSION".into(),
            message: "HTTP response compression is disabled.".into(),
        });
    }
    if env::var("YEW_FULLSTACK_CORS_ORIGINS")
        .map(|s| s.trim().is_empty())
        .unwrap_or(true)
    {
        out.push(ProdWarning {
            code: "CORS_DEFAULT".into(),
            message:
                "YEW_FULLSTACK_CORS_ORIGINS unset — using default; set public origin(s) in prod."
                    .into(),
        });
    }
    out
}

pub fn version_mismatch_warning(frontend: &str, backend: &str) -> Option<ProdWarning> {
    let fe = frontend.trim();
    let be = backend.trim();
    if fe.is_empty() || be.is_empty() || fe == be {
        return None;
    }
    Some(ProdWarning {
        code: "VERSION_MISMATCH".into(),
        message: format!("Frontend {fe} ≠ backend {be} — stale WASM or mixed deploy."),
    })
}

fn shorten_err(s: &str) -> String {
    // Never echo connection strings / passwords if the driver embeds them.
    let scrubbed = s
        .split("mongodb://")
        .next()
        .unwrap_or(s)
        .chars()
        .take(160)
        .collect::<String>();
    if scrubbed.contains("Authentication") || scrubbed.contains("auth") {
        "authentication failed (check MONGO_PASSWORD vs volume)".into()
    } else if scrubbed.contains("server selection") || scrubbed.contains("timed out") {
        "server selection / timeout".into()
    } else {
        scrubbed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_warnings_flag_empty_build_id_and_no_compress() {
        // Ensure CORS warning path does not panic; env may or may not be set in CI.
        let w = production_warnings("", false);
        assert!(w.iter().any(|x| x.code == "NO_BUILD_ID"));
        assert!(w.iter().any(|x| x.code == "NO_COMPRESSION"));
    }

    #[test]
    fn version_mismatch_only_when_both_set_and_differ() {
        assert!(version_mismatch_warning("", "0.0.1").is_none());
        assert!(version_mismatch_warning("0.0.1", "0.0.1").is_none());
        let w = version_mismatch_warning("0.0.1", "0.0.2").unwrap();
        assert_eq!(w.code, "VERSION_MISMATCH");
    }

    #[test]
    fn shorten_err_maps_auth() {
        assert!(shorten_err("Authentication failed for user root").contains("MONGO_PASSWORD"));
    }
}
