use crate::{
    models::{
        response::ResponseBody,
        user::{ChangePasswordDTO, EmailChangeDTO, LoginDTO, TotpVerifyDTO, User},
    },
    services::{account_service, account_settings_service},
    utils::auth::extract_auth_user,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

// POST api/auth/signup
pub async fn signup(user_dto: web::Json<User>, db: web::Data<Database>) -> Result<HttpResponse> {
    match account_service::signup(user_dto.0, &db).await {
        Ok(data) => Ok(HttpResponse::Ok().json(ResponseBody::new("SUCCESS", &data))),
        Err(err) => Ok(err.response()),
    }
}

// POST api/auth/login
pub async fn login(
    login_dto: web::Json<LoginDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match account_service::login(&login_dto.0, &db).await {
        Ok(token_res) => Ok(HttpResponse::Ok().json(ResponseBody::new("LOGIN_SUCCESS", token_res))),
        Err(err) => Ok(err.response()),
    }
}

// POST api/auth/logout
pub async fn logout(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    if let Some(authen_header) = req.headers().get("Authorization") {
        match account_service::logout(authen_header, &db).await {
            Ok(_) => Ok(HttpResponse::Ok().json(ResponseBody::new("LOGOUT_SUCCESS", ""))),
            Err(err) => Ok(err.response()),
        }
    } else {
        Ok(HttpResponse::BadRequest().json(ResponseBody::new("MISSING_TOKEN", "")))
    }
}

pub async fn me(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => Ok(HttpResponse::Ok().json(account_settings_service::me(&auth, &db).await)),
        None => Ok(HttpResponse::Unauthorized().json(ResponseBody::new("UNAUTHORIZED", ""))),
    }
}

pub async fn change_password(
    req: HttpRequest,
    body: web::Json<ChangePasswordDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => match account_settings_service::change_password(&auth, body.0, &db).await {
            Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("PASSWORD_UPDATED", msg))),
            Err(err) => Ok(err.response()),
        },
        None => Ok(HttpResponse::Unauthorized().json(ResponseBody::new("UNAUTHORIZED", ""))),
    }
}

pub async fn request_email_change(
    req: HttpRequest,
    body: web::Json<EmailChangeDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => {
            match account_settings_service::request_email_change(&auth, body.0, &db).await {
                Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("EMAIL_CHANGE_REQUESTED", msg))),
                Err(err) => Ok(err.response()),
            }
        }
        None => Ok(HttpResponse::Unauthorized().json(ResponseBody::new("UNAUTHORIZED", ""))),
    }
}

pub async fn totp_enroll(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => match account_settings_service::totp_enroll(&auth, &db).await {
            Ok((secret, otpauth)) => Ok(HttpResponse::Ok().json(serde_json::json!({
                "secret": secret,
                "otpauthUrl": otpauth
            }))),
            Err(err) => Ok(err.response()),
        },
        None => Ok(HttpResponse::Unauthorized().json(ResponseBody::new("UNAUTHORIZED", ""))),
    }
}

pub async fn totp_confirm(
    req: HttpRequest,
    body: web::Json<TotpVerifyDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => match account_settings_service::totp_confirm(&auth, body.0, &db).await {
            Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("TOTP_ENABLED", msg))),
            Err(err) => Ok(err.response()),
        },
        None => Ok(HttpResponse::Unauthorized().json(ResponseBody::new("UNAUTHORIZED", ""))),
    }
}

pub async fn totp_disable(
    req: HttpRequest,
    body: web::Json<TotpVerifyDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match extract_auth_user(&req, &db).await {
        Some(auth) => match account_settings_service::totp_disable(&auth, body.0, &db).await {
            Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("TOTP_DISABLED", msg))),
            Err(err) => Ok(err.response()),
        },
        None => Ok(HttpResponse::Unauthorized().json(ResponseBody::new("UNAUTHORIZED", ""))),
    }
}
