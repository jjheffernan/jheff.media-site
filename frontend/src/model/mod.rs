use serde::{Deserialize, Serialize};

mod auth;
mod content;
mod feed;
mod media;
mod social;

pub use auth::Auth;
pub use auth::Login;
pub use auth::Signup;
pub use auth::User;
pub use content::{
    ContactSubmission, ContentCollection, ContentCollectionSummary, ContentHubResponse,
    ContentListResponse, MediaItem, OtherSite, OtherSitesResponse,
};
pub use feed::{FeedItem, FeedResponse};
pub use media::{
    AdminPrintOrder, ChangePassword, EmailChange, InstagramFeatured, MediaFeedItem,
    MediaFeedResponse, PrintOrder,
    PrintSelection, PrintSelectionInput, TotpEnrollResponse, TotpVerify,
};
pub use social::{SocialAccount, SocialHubResponse, SocialPost};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerResponse<T> {
    pub message: String,
    pub data: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_deserializes_camel_case_fields() {
        let login: Login =
            serde_json::from_str(r#"{"emailOrUsername":"u","password":"p"}"#).unwrap();
        assert_eq!(login.email_or_username, "u");
        assert_eq!(login.password, "p");
    }

    #[test]
    fn feed_item_deserializes_camel_case() {
        let item: FeedItem = serde_json::from_str(
            r#"{"id":"a","thumbnailUrl":"/api/media/thumbnail/a","title":"shot"}"#,
        )
        .unwrap();
        assert_eq!(item.id, "a");
        assert_eq!(item.thumbnail_url, "/api/media/thumbnail/a");
    }

    #[test]
    fn server_response_matches_api_envelope() {
        let resp = ServerResponse {
            message: "LOGIN_SUCCESS".into(),
            data: "payload".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["message"], "LOGIN_SUCCESS");
        assert_eq!(json["data"], "payload");
    }

    #[test]
    fn media_feed_item_deserializes_unified_feed_shape() {
        let item: MediaFeedItem = serde_json::from_str(
            r#"{
                "id":"x","source":"instagram","mediaType":"image",
                "thumbnailUrl":"/t","featured":false
            }"#,
        )
        .unwrap();
        assert_eq!(item.source, "instagram");
        assert!(item.title.is_none());
        assert!(!item.featured);
    }

    #[test]
    fn contact_submission_serializes_preferred_at_when_set() {
        let mut body = ContactSubmission {
            name: "N".into(),
            email: "e".into(),
            subject: "S".into(),
            message: "M".into(),
            website: String::new(),
            kind: "booking".into(),
            preferred_at: "2026-11-02T15:45".into(),
        };
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v["kind"], "booking");
        assert_eq!(v["preferredAt"], "2026-11-02T15:45");
        body.preferred_at.clear();
        let v2 = serde_json::to_value(&body).unwrap();
        assert!(v2.get("preferredAt").is_none());
    }
}
