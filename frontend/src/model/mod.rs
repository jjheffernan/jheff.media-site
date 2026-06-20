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
    BookingConfig, ContactSubmission, ContentCollection, ContentCollectionSummary,
    ContentListResponse, MediaItem, OtherSite, OtherSitesResponse,
};
pub use feed::{FeedItem, FeedResponse};
pub use media::{
    ChangePassword, EmailChange, InstagramFeatured, MediaFeedItem, MediaFeedResponse,
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
            r#"{"id":"a","thumbnailUrl":"/api/feed/thumbnail/a","title":"shot"}"#,
        )
        .unwrap();
        assert_eq!(item.id, "a");
        assert_eq!(item.thumbnail_url, "/api/feed/thumbnail/a");
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
}
