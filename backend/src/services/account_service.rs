use crate::{
    error::ServiceError,
    models::user::{LoginDTO, PublicUserDTO, User},
    models::user_token::UserToken,
    utils::token as token_utils,
};
use actix_web::{
    http::{header::HeaderValue, StatusCode},
    web,
};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const AUTH_RATE_LIMIT: usize = 8;
const AUTH_RATE_WINDOW: Duration = Duration::from_secs(10 * 60);

fn allow(hits: &Mutex<Option<HashMap<String, Vec<Instant>>>>, ip: &str) -> bool {
    let mut guard = hits.lock().unwrap_or_else(|err| err.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let now = Instant::now();
    let recent = map.entry(ip.to_string()).or_default();
    recent.retain(|at| now.duration_since(*at) < AUTH_RATE_WINDOW);
    if recent.len() >= AUTH_RATE_LIMIT {
        return false;
    }
    recent.push(now);
    true
}

fn allow_login(ip: &str) -> bool {
    static HITS: Mutex<Option<HashMap<String, Vec<Instant>>>> = Mutex::new(None);
    allow(&HITS, ip)
}

fn allow_signup(ip: &str) -> bool {
    static HITS: Mutex<Option<HashMap<String, Vec<Instant>>>> = Mutex::new(None);
    allow(&HITS, ip)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenBodyResponse {
    pub token: String,
    pub token_type: String,
    pub user: PublicUserDTO,
}

pub async fn signup(
    user: User,
    client_ip: &str,
    db: &web::Data<Database>,
) -> Result<String, ServiceError> {
    if !allow_signup(client_ip) {
        return Err(ServiceError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts. Try again later.",
        ));
    }
    if user.password.len() < 8 {
        return Err(ServiceError::new(
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters.",
        ));
    }
    match User::signup(user, db).await {
        Ok(id) => Ok(id.to_hex()),
        Err(err) => match err.as_str() {
            "USER_ALREADY_EXISTS" => Err(ServiceError::new(StatusCode::BAD_REQUEST, err)),
            _ => Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, err)),
        },
    }
}

pub async fn login(
    login: &LoginDTO,
    client_ip: &str,
    db: &web::Data<Database>,
) -> Result<TokenBodyResponse, ServiceError> {
    if !allow_login(client_ip) {
        return Err(ServiceError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts. Try again later.",
        ));
    }
    let logged_user = User::login(&login, db).await?;
    let public_user: PublicUserDTO = logged_user.clone().into();
    match serde_json::from_value(
        json!({ "token": UserToken::generate_token(logged_user), "token_type": "bearer", "user": public_user }),
    ) {
        Ok(token_res) => Ok(token_res),
        Err(_) => Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR".to_string(),
        )),
    }
}

pub async fn logout(authen_header: &HeaderValue, db: &web::Data<Database>) -> Result<(), ServiceError> {
    if let Ok(authen_str) = authen_header.to_str() {
        let authen_str = authen_str.trim();
        if authen_str.len() > 7 && authen_str[..7].eq_ignore_ascii_case("bearer ") {
            let token = authen_str[7..].trim();
            if token.is_empty() {
                return Err(ServiceError::new(
                    StatusCode::BAD_REQUEST,
                    "MISSING_TOKEN".to_string(),
                ));
            }
            if let Ok(token_data) = token_utils::decode_token(token.to_string()) {
                if let Ok(username) = token_utils::verify_token(&token_data, &**db).await {
                    if let Some(user) = User::find_by_email_or_username(&username, db).await {
                        User::logout(user.id.unwrap(), db).await;
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(ServiceError::new(
        StatusCode::UNAUTHORIZED,
        "INVALID_TOKEN".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{allow_login, allow_signup};

    #[test]
    fn rate_limit_is_eight_and_separate_per_action() {
        let ip = format!("auth-test-{}", std::process::id());
        for _ in 0..8 {
            assert!(allow_login(&ip));
            assert!(allow_signup(&ip));
        }
        assert!(!allow_login(&ip));
        assert!(!allow_signup(&ip));
    }
}
