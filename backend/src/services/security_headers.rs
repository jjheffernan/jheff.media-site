use std::env;
use url::Url;

/// Content-Security-Policy for the Yew WASM SPA + known third parties.
///
/// `wasm-unsafe-eval` is required so browsers can compile WebAssembly (Trunk/Yew).
/// It is narrower than `'unsafe-eval'` and does not allow arbitrary JS eval.
/// The sha256 source is the theme bootstrap `<script>` in `frontend/index.html` —
/// recompute if that inline script changes.
pub fn content_security_policy() -> String {
    let mut frame_src = vec![
        "'self'".to_string(),
        "https://www.youtube.com".to_string(),
        "https://youtube.com".to_string(),
        "https://www.instagram.com".to_string(),
        "https://instagram.com".to_string(),
        "https://calendar.google.com".to_string(),
        "https://calendly.com".to_string(),
    ];

    if let Ok(embed) = env::var("YEW_FULLSTACK_SCHEDULE_EMBED_URL") {
        if let Some(origin) = http_origin(embed.trim()) {
            if !frame_src.iter().any(|h| h == &origin) {
                frame_src.push(origin);
            }
        }
    }

    // img/media allow https: for Instagram/YouTube CDN host churn (scontent-*.cdninstagram.com, etc.).
    format!(
        "default-src 'self'; \
         script-src 'self' 'wasm-unsafe-eval' 'sha256-gtLPZ3PAowCv48mR4XHiDVaWelN1Nl+IA1ojmNzzLCI='; \
         style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
         font-src 'self' https://fonts.gstatic.com data:; \
         img-src 'self' data: blob: https:; \
         media-src 'self' blob: https:; \
         connect-src 'self'; \
         frame-src {}; \
         object-src 'none'; \
         base-uri 'self'; \
         form-action 'self'; \
         frame-ancestors 'none'",
        frame_src.join(" ")
    )
}

fn http_origin(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return None;
    }
    let url = Url::parse(raw).ok()?;
    let host = url.host_str()?;
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    match url.port() {
        Some(port) if !matches!((scheme, port), ("http", 80) | ("https", 443)) => {
            Some(format!("{scheme}://{host}:{port}"))
        }
        _ => Some(format!("{scheme}://{host}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csp_allows_wasm_and_self() {
        let csp = content_security_policy();
        assert!(csp.contains("'wasm-unsafe-eval'"));
        assert!(csp.contains("script-src 'self'"));
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("connect-src 'self'"));
    }

    #[test]
    fn http_origin_strips_path() {
        assert_eq!(
            http_origin("https://cal.example/embed?x=1").as_deref(),
            Some("https://cal.example")
        );
        assert_eq!(
            http_origin("https://cal.example:8443/x").as_deref(),
            Some("https://cal.example:8443")
        );
        assert_eq!(http_origin("not a url"), None);
    }
}
