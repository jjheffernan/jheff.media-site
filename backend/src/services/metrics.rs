use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    Error,
};

// ponytail: counters live in this process. Prometheus scrapes them; a restart starts at zero.
struct Hits {
    http: BTreeMap<(String, u16), u64>,
    telemetry: u64,
}

static HITS: LazyLock<Mutex<Hits>> = LazyLock::new(|| {
    Mutex::new(Hits {
        http: BTreeMap::new(),
        telemetry: 0,
    })
});

pub fn record_http(method: &str, status: u16) {
    let mut hits = HITS.lock().unwrap_or_else(|err| err.into_inner());
    *hits.http.entry((method.to_string(), status)).or_insert(0) += 1;
}

pub fn record_telemetry() {
    let mut hits = HITS.lock().unwrap_or_else(|err| err.into_inner());
    hits.telemetry += 1;
}

pub fn exposition() -> String {
    let hits = HITS.lock().unwrap_or_else(|err| err.into_inner());
    let mut out = String::from(
        "# TYPE http_requests_total counter\n# TYPE telemetry_events_total counter\n",
    );
    for ((method, status), n) in &hits.http {
        out.push_str(&format!(
            "http_requests_total{{method=\"{method}\",status=\"{status}\"}} {n}\n"
        ));
    }
    out.push_str(&format!("telemetry_events_total {}\n", hits.telemetry));
    out
}

pub async fn track(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let method = req.method().as_str().to_string();
    let skip = req.path() == "/metrics";
    let res = next.call(req).await?;
    if !skip {
        record_http(&method, res.status().as_u16());
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposition_lists_a_request_and_a_page_view() {
        record_http("TEST", 204);
        record_telemetry();
        let text = exposition();
        assert!(text.contains("http_requests_total{method=\"TEST\",status=\"204\"} "));
        assert!(text.contains("telemetry_events_total "));
    }
}
