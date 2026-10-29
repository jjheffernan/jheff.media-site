use crate::{
    models::{
        response::ResponseBody,
        user::{ChangePasswordDTO, EmailChangeDTO, LoginDTO, TotpVerifyDTO, User},
    },
    services::{account_service, account_settings_service},
    utils::auth::require_auth,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;

fn client_ip(req: &HttpRequest) -> String {
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string()
}

// POST api/auth/signup
pub async fn signup(
    req: HttpRequest,
    user_dto: web::Json<User>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match account_service::signup(user_dto.0, &client_ip(&req), &db).await {
        Ok(data) => Ok(HttpResponse::Ok().json(ResponseBody::new("SUCCESS", &data))),
        Err(err) => Ok(err.response()),
    }
}

// POST api/auth/login
pub async fn login(
    req: HttpRequest,
    login_dto: web::Json<LoginDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    match account_service::login(&login_dto.0, &client_ip(&req), &db).await {
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
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    Ok(HttpResponse::Ok().json(account_settings_service::me(&auth, &db).await))
}

pub async fn change_password(
    req: HttpRequest,
    body: web::Json<ChangePasswordDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match account_settings_service::change_password(&auth, body.0, &db).await {
        Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("PASSWORD_UPDATED", msg))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn request_email_change(
    req: HttpRequest,
    body: web::Json<EmailChangeDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match account_settings_service::request_email_change(&auth, body.0, &db).await {
        Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("EMAIL_CHANGE_REQUESTED", msg))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn totp_enroll(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match account_settings_service::totp_enroll(&auth, &db).await {
        Ok((secret, otpauth)) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "secret": secret,
            "otpauthUrl": otpauth
        }))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn totp_confirm(
    req: HttpRequest,
    body: web::Json<TotpVerifyDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match account_settings_service::totp_confirm(&auth, body.0, &db).await {
        Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("TOTP_ENABLED", msg))),
        Err(err) => Ok(err.response()),
    }
}

pub async fn totp_disable(
    req: HttpRequest,
    body: web::Json<TotpVerifyDTO>,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let auth = match require_auth(&req, &db).await {
        Ok(auth) => auth,
        Err(resp) => return Ok(resp),
    };
    match account_settings_service::totp_disable(&auth, body.0, &db).await {
        Ok(msg) => Ok(HttpResponse::Ok().json(ResponseBody::new("TOTP_DISABLED", msg))),
        Err(err) => Ok(err.response()),
    }
}
