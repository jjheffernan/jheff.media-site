use crate::models::{
    user::User,
    user_token::{UserToken, KEY},
};
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use mongodb::Database;

pub fn decode_token(token: String) -> jsonwebtoken::errors::Result<TokenData<UserToken>> {
    jsonwebtoken::decode::<UserToken>(
        &token,
        &DecodingKey::from_secret(&KEY),
        &Validation::default(),
    )
}

pub async fn verify_token(
    token_data: &TokenData<UserToken>,
    db: &Database,
) -> Result<String, String> {
    if User::is_valid_login_session(&token_data.claims, db).await {
        Ok(token_data.claims.user.to_string())
    } else {
        Err("Invalid token".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user_token::UserToken;
    use jsonwebtoken::{EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn decode_token_rejects_garbage() {
        assert!(decode_token("not-a-jwt".into()).is_err());
    }

    #[test]
    fn decode_token_accepts_valid_signed_token() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let claims = UserToken {
            iat: now,
            exp: now + 3600,
            user: "tester".into(),
            login_session: "sess".into(),
        };
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&crate::models::user_token::KEY),
        )
        .unwrap();
        let decoded = decode_token(token).unwrap();
        assert_eq!(decoded.claims.user, "tester");
    }
}
