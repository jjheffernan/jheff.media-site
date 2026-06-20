use crate::{
    error::{ServiceError, ServiceResult},
    models::user::{ChangePasswordDTO, EmailChangeDTO, PublicUserDTO, TotpVerifyDTO, User},
    utils::auth::AuthUser,
};
use actix_web::http::StatusCode;
use bcrypt::{hash, verify, DEFAULT_COST};
use data_encoding::BASE32;
use mongodb::Database;
use rand::RngCore;
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};

pub async fn me(auth: &AuthUser, db: &Database) -> PublicUserDTO {
    if let Some(user) = User::find_by_id(&auth.id, db).await {
        return PublicUserDTO::from(user);
    }
    PublicUserDTO {
        email: auth.email.clone(),
        username: auth.username.clone(),
        role: auth.role.clone(),
        totp_enabled: auth.totp_enabled,
    }
}

pub async fn change_password(
    auth: &AuthUser,
    dto: ChangePasswordDTO,
    db: &Database,
) -> ServiceResult<String> {
    if dto.new_password.len() < 8 {
        return Err(ServiceError::new(
            StatusCode::BAD_REQUEST,
            "Password must be at least 8 characters.",
        ));
    }

    let user = User::find_by_id(&auth.id, db)
        .await
        .ok_or_else(|| ServiceError::new(StatusCode::NOT_FOUND, "USER_NOT_FOUND"))?;

    if !verify(&dto.current_password, &user.password).unwrap_or(false) {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "BAD_PASSWORD"));
    }

    let hashed = hash(&dto.new_password, DEFAULT_COST)
        .map_err(|_| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, "HASH_ERROR"))?;

    if !User::update_password(&auth.id, &hashed, db).await {
        return Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UPDATE_FAILED",
        ));
    }

    Ok(String::from("Password updated."))
}

pub async fn request_email_change(
    auth: &AuthUser,
    dto: EmailChangeDTO,
    db: &Database,
) -> ServiceResult<String> {
    let new_email = dto.new_email.trim().to_lowercase();
    if new_email.is_empty() || !new_email.contains('@') {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_EMAIL"));
    }

    if User::find_by_email_or_username(&new_email, db).await.is_some() {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "EMAIL_IN_USE"));
    }

    if !User::set_pending_email(&auth.id, &new_email, db).await {
        return Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UPDATE_FAILED",
        ));
    }

    info!(
        "Email change requested for {} → {} (delivery not configured)",
        auth.email, new_email
    );

    Ok(format!(
        "Change link requested for {}. Check your inbox when email delivery is configured.",
        new_email
    ))
}

pub async fn totp_enroll(auth: &AuthUser, db: &Database) -> ServiceResult<(String, String)> {
    let mut secret_bytes = [0u8; 20];
    rand::thread_rng().fill_bytes(&mut secret_bytes);
    let secret_base32 = BASE32.encode(&secret_bytes);

    if !User::set_totp_secret(&auth.id, &secret_base32, db).await {
        return Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UPDATE_FAILED",
        ));
    }

    let otpauth = format!(
        "otpauth://totp/jheff.media:{}?secret={}&issuer=jheff.media",
        auth.email, secret_base32
    );

    Ok((secret_base32, otpauth))
}

pub async fn totp_confirm(
    auth: &AuthUser,
    dto: TotpVerifyDTO,
    db: &Database,
) -> ServiceResult<String> {
    let user = User::find_by_id(&auth.id, db)
        .await
        .ok_or_else(|| ServiceError::new(StatusCode::NOT_FOUND, "USER_NOT_FOUND"))?;

    let secret = user
        .totp_secret
        .as_ref()
        .ok_or_else(|| ServiceError::new(StatusCode::BAD_REQUEST, "TOTP_NOT_STARTED"))?;

    if !verify_totp_code(secret, &dto.code) {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_TOTP"));
    }

    if !User::enable_totp(&auth.id, db).await {
        return Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UPDATE_FAILED",
        ));
    }

    Ok(String::from("Two-factor authentication enabled."))
}

pub async fn totp_disable(
    auth: &AuthUser,
    dto: TotpVerifyDTO,
    db: &Database,
) -> ServiceResult<String> {
    let user = User::find_by_id(&auth.id, db)
        .await
        .ok_or_else(|| ServiceError::new(StatusCode::NOT_FOUND, "USER_NOT_FOUND"))?;

    if user.totp_enabled {
        let secret = user
            .totp_secret
            .as_ref()
            .ok_or_else(|| ServiceError::new(StatusCode::BAD_REQUEST, "TOTP_NOT_CONFIGURED"))?;
        if !verify_totp_code(secret, &dto.code) {
            return Err(ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_TOTP"));
        }
    }

    if !User::disable_totp(&auth.id, db).await {
        return Err(ServiceError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UPDATE_FAILED",
        ));
    }

    Ok(String::from("Two-factor authentication disabled."))
}

fn verify_totp_code(secret_base32: &str, code: &str) -> bool {
    let secret_bytes = BASE32.decode(secret_base32.to_uppercase().as_bytes()).unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expected = totp_custom::<Sha1>(DEFAULT_STEP, 6, &secret_bytes, now);
    expected == code.trim()
}
