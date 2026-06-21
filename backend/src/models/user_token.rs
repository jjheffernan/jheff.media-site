use crate::models::user::LoginInfoDTO;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

pub static KEY: [u8; 32] = *include_bytes!("../secret.key");
static ONE_WEEK: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // data
    pub user: String,
    pub login_session: String,
}

impl UserToken {
    pub fn generate_token(login: LoginInfoDTO) -> String {
        let now = Utc::now().timestamp();
        let payload = UserToken {
            iat: now,
            exp: now + ONE_WEEK,
            user: login.username,
            login_session: login.login_session,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&KEY),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::LoginInfoDTO;
    use crate::utils::token::decode_token;

    #[test]
    fn generate_token_roundtrip_decodes_claims() {
        let login = LoginInfoDTO {
            email: "a@example.com".into(),
            username: "alice".into(),
            login_session: "session-abc".into(),
            role: "user".into(),
            totp_enabled: false,
        };
        let token = UserToken::generate_token(login.clone());
        let decoded = decode_token(token).expect("token should decode");
        assert_eq!(decoded.claims.user, login.username);
        assert_eq!(decoded.claims.login_session, login.login_session);
        assert!(decoded.claims.exp > decoded.claims.iat);
    }
}
