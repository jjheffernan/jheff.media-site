//! Client-side ring buffer of failed API fetches (status + path only).
//! // ponytail: no request bodies/headers — add server-side audit when needed.

use serde::Serialize;
use std::cell::RefCell;

const CAP: usize = 24;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiFailure {
    pub path: String,
    /// HTTP status, or `0` for network / send failure.
    pub status: u16,
    pub at_ms: f64,
}

thread_local! {
    static LOG: RefCell<Vec<ApiFailure>> = RefCell::new(Vec::new());
}

fn scrub_path(path: &str) -> String {
    let bare = path.split('?').next().unwrap_or(path);
    // Drop anything that looks like a bearer/token fragment in the path.
    if bare.to_ascii_lowercase().contains("token") || bare.to_ascii_lowercase().contains("key=") {
        return "[redacted]".into();
    }
    bare.chars().take(160).collect()
}

pub fn push(path: &str, status: u16) {
    let entry = ApiFailure {
        path: scrub_path(path),
        status,
        at_ms: js_sys::Date::now(),
    };
    LOG.with(|log| {
        let mut v = log.borrow_mut();
        if v.len() >= CAP {
            v.remove(0);
        }
        v.push(entry);
    });
}

pub fn recent() -> Vec<ApiFailure> {
    LOG.with(|log| log.borrow().clone())
}

pub fn clear() {
    LOG.with(|log| log.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::scrub_path;

    #[test]
    fn scrub_path_strips_query_string() {
        assert_eq!(scrub_path("/api/media/feed?cursor=1"), "/api/media/feed");
    }

    #[test]
    fn scrub_path_redacts_tokenish_segments() {
        assert_eq!(scrub_path("/api/auth/token/refresh"), "[redacted]");
        // Query is stripped first; `key=` only redacts when present in the path body.
        assert_eq!(scrub_path("/api/links/key=secret"), "[redacted]");
        assert_eq!(scrub_path("/api/setup?key=abc"), "/api/setup");
    }

    #[test]
    fn scrub_path_truncates_long_paths() {
        let long = format!("/{}", "a".repeat(200));
        assert_eq!(scrub_path(&long).len(), 160);
    }
}
