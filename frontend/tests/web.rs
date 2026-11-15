//! Frontend contract tests for browser / wasm-bindgen-test.
//!
//! Host CI (`cargo test -p jheffmedia-site-frontend`) runs pure-logic cases as normal `#[test]`s.
//! On wasm32, `wasm-pack test --headless --chrome` (or Firefox) also runs DOM + `api_errors` cases.

use jheffmedia_site_frontend::browser_test_api::{
    has_explicit_offset, AppRoutes, ContactSubmission, ContentCollectionSummary,
    ContentListResponse, Login, MediaFeedItem, MediaFeedResponse, ServerResponse, ThemePreference,
    User,
};
use yew_router::Routable;

#[cfg(target_arch = "wasm32")]
use jheffmedia_site_frontend::browser_test_api::api_errors;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn media_feed_response_deserializes_camel_case() {
    let feed: MediaFeedResponse = serde_json::from_str(
        r#"{
            "items": [{
                "id": "img-1",
                "source": "gallery",
                "mediaType": "image",
                "title": "Miata",
                "caption": null,
                "thumbnailUrl": "/api/media/thumb/img-1",
                "mediaUrl": "/api/media/img-1",
                "linkUrl": null,
                "publishedAt": "2026-11-01T12:00:00Z",
                "collectionId": "g1",
                "collectionKind": "gallery",
                "collectionTitle": "Autumn",
                "featured": true
            }],
            "sources": ["gallery", "instagram"]
        }"#,
    )
    .expect("MediaFeedResponse JSON");
    assert_eq!(feed.items.len(), 1);
    let item: &MediaFeedItem = &feed.items[0];
    assert_eq!(item.id, "img-1");
    assert_eq!(item.media_type, "image");
    assert_eq!(item.thumbnail_url, "/api/media/thumb/img-1");
    assert!(item.featured);
    assert_eq!(feed.sources, vec!["gallery", "instagram"]);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn login_and_user_roundtrip_camel_case() {
    let login = Login {
        email_or_username: "driver".into(),
        password: "secret".into(),
        totp_code: Some("123456".into()),
    };
    let json = serde_json::to_value(&login).unwrap();
    assert_eq!(json["emailOrUsername"], "driver");
    assert_eq!(json["totpCode"], "123456");

    let user: User = serde_json::from_str(
        r#"{"email":"a@b.c","username":"driver","role":"admin","totpEnabled":true}"#,
    )
    .unwrap();
    assert_eq!(user.role, "admin");
    assert!(user.totp_enabled);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn contact_submission_defaults_kind() {
    let body: ContactSubmission = serde_json::from_str(
        r#"{"name":"N","email":"e@x","subject":"S","message":"M"}"#,
    )
    .unwrap();
    assert_eq!(body.kind, "contact");
    assert!(body.website.is_empty());
    assert!(body.preferred_at.is_empty());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn content_list_and_server_envelope() {
    let list: ContentListResponse = serde_json::from_str(
        r#"{
            "items": [{
                "id": "s1",
                "title": "Track day",
                "summary": null,
                "coverUrl": "/c.jpg",
                "status": "published",
                "date": "2026-06-01",
                "location": "Watkins Glen",
                "mediaCount": 12,
                "hasVideo": false
            }],
            "source": "mongodb"
        }"#,
    )
    .unwrap();
    let summary: &ContentCollectionSummary = &list.items[0];
    assert_eq!(summary.media_count, 12);
    assert_eq!(summary.cover_url.as_deref(), Some("/c.jpg"));

    let envelope = ServerResponse {
        message: "OK".into(),
        data: list,
    };
    let v = serde_json::to_value(&envelope).unwrap();
    assert_eq!(v["message"], "OK");
    assert_eq!(v["data"]["source"], "mongodb");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn app_routes_recognize_and_to_path() {
    assert_eq!(AppRoutes::recognize("/"), Some(AppRoutes::Home));
    assert_eq!(AppRoutes::recognize("/content"), Some(AppRoutes::Content));
    assert_eq!(
        AppRoutes::recognize("/galleries/abc"),
        Some(AppRoutes::GalleryDetail {
            id: "abc".into()
        })
    );
    assert_eq!(
        AppRoutes::recognize("/shoots/xyz"),
        Some(AppRoutes::ShootDetail {
            id: "xyz".into()
        })
    );
    assert_eq!(AppRoutes::recognize("/legal/terms"), Some(AppRoutes::Terms));
    assert_eq!(AppRoutes::Home.to_path(), "/");
    assert_eq!(
        AppRoutes::GalleryDetail {
            id: "g1".into()
        }
        .to_path(),
        "/galleries/g1"
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn has_explicit_offset_detects_z_and_numeric() {
    assert!(has_explicit_offset("2026-11-02T15:45:00Z"));
    assert!(has_explicit_offset("2026-11-02T15:45:00-05:00"));
    assert!(has_explicit_offset("2026-11-02T15:45:00+01:00"));
    assert!(!has_explicit_offset("2026-11-02T15:45"));
    assert!(!has_explicit_offset("short"));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn theme_preference_parses_and_roundtrips() {
    assert_eq!(
        ThemePreference::from_str("light"),
        ThemePreference::Light
    );
    assert_eq!(ThemePreference::from_str("dark"), ThemePreference::Dark);
    assert_eq!(ThemePreference::from_str("auto"), ThemePreference::Auto);
    assert_eq!(ThemePreference::from_str("nope"), ThemePreference::Auto);
    assert_eq!(ThemePreference::Light.as_str(), "light");
    assert_eq!(ThemePreference::Dark.as_str(), "dark");
    assert_eq!(ThemePreference::Auto.as_str(), "auto");
}

/// Needs `js_sys::Date` — host CI skips; wasm-pack / browser runs it.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn api_errors_scrub_query_and_tokenish_paths() {
    api_errors::clear();
    api_errors::push("/api/media/feed?cursor=1", 502);
    api_errors::push("/api/auth/token/refresh", 401);
    let recent = api_errors::recent();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].path, "/api/media/feed");
    assert_eq!(recent[0].status, 502);
    assert_eq!(recent[1].path, "[redacted]");
    assert_eq!(recent[1].status, 401);
    api_errors::clear();
    assert!(api_errors::recent().is_empty());
}

/// Confirm the harness has a document to mount into.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn document_is_available_in_browser() {
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let body = document.body().expect("body");
    let el = document.create_element("div").unwrap();
    el.set_id("wasm-bindgen-smoke");
    el.set_text_content(Some("ok"));
    body.append_child(&el).unwrap();
    assert_eq!(
        document
            .get_element_by_id("wasm-bindgen-smoke")
            .unwrap()
            .text_content()
            .as_deref(),
        Some("ok")
    );
    el.remove();
}
