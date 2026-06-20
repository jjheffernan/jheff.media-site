use crate::models::user::User;
use crate::utils::token::{decode_token, verify_token};
use actix_web::{http::header, HttpRequest};
use bson::oid::ObjectId;
use mongodb::Database;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    pub role: String,
    pub totp_enabled: bool,
}

pub async fn extract_auth_user(req: &HttpRequest, db: &Database) -> Option<AuthUser> {
    let header = req.headers().get(header::AUTHORIZATION)?;
    let authen_str = header.to_str().ok()?;
    if !authen_str.to_lowercase().starts_with("bearer ") {
        return None;
    }
    let token = authen_str[7..].trim();
    let token_data = decode_token(token.to_string()).ok()?;
    let username = verify_token(&token_data, db).await.ok()?;
    let user = User::find_by_email_or_username(&username, db).await?;
    let id = user.id?;
    Some(AuthUser {
        id,
        username: user.username,
        email: user.email,
        role: user.role.clone(),
        totp_enabled: user.totp_enabled,
    })
}

pub fn is_admin(user: &AuthUser) -> bool {
    user.role == "admin"
}
