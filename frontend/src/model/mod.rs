use serde::{Deserialize, Serialize};

mod auth;

pub use auth::Auth;
pub use auth::Login;
pub use auth::Signup;
pub use auth::User;

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
