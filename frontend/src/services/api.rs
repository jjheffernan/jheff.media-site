//! Shared JSON fetch helpers over gloo-net.
//! Consolidates auth header + ok/error envelope parsing used across pages.

use gloo_net::http::{Method, RequestBuilder, Response};
use serde::{de::DeserializeOwned, Serialize};

pub use gloo_net::http::Method as HttpMethod;

pub fn auth_header(jwt: &str) -> String {
    format!("bearer {}", jwt)
}

/// Read `message` from the API JSON envelope (`ServerResponse`), else `fallback`.
pub async fn error_message(resp: Response, fallback: &str) -> String {
    if let Ok(body) = resp.json::<serde_json::Value>().await {
        body.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or(fallback)
            .to_string()
    } else {
        fallback.to_string()
    }
}

async fn send(
    method: Method,
    url: &str,
    jwt: Option<&str>,
    body: Option<String>,
) -> Result<Response, String> {
    let mut builder = RequestBuilder::new(url).method(method);
    if let Some(jwt) = jwt {
        builder = builder.header("Authorization", &auth_header(jwt));
    }
    let resp = if let Some(body) = body {
        builder
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?
    } else {
        builder.send().await.map_err(|e| e.to_string())?
    };
    Ok(resp)
}

/// Send a request and deserialize JSON on success; on failure parse the API `message` envelope.
pub async fn json<T: DeserializeOwned>(
    method: Method,
    url: &str,
    jwt: Option<&str>,
    body: Option<String>,
) -> Result<T, String> {
    let resp = send(method, url, jwt, body).await?;
    if resp.ok() {
        resp.json()
            .await
            .map_err(|_| "Could not parse response.".into())
    } else {
        Err(error_message(resp, "Request failed.").await)
    }
}

/// Send a request; succeed on HTTP ok without reading a body.
pub async fn ok(
    method: Method,
    url: &str,
    jwt: Option<&str>,
    body: Option<String>,
) -> Result<(), String> {
    let resp = send(method, url, jwt, body).await?;
    if resp.ok() {
        Ok(())
    } else {
        Err(error_message(resp, "Request failed.").await)
    }
}

pub async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    json(Method::GET, url, None, None).await
}

pub async fn get_json_auth<T: DeserializeOwned>(url: &str, jwt: &str) -> Result<T, String> {
    json(Method::GET, url, Some(jwt), None).await
}

/// Unauthenticated JSON POST (e.g. contact form). Kept for call sites that need it.
#[allow(dead_code)]
pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    url: &str,
    body: &B,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|e| e.to_string())?;
    json(Method::POST, url, None, Some(body)).await
}

pub async fn post_json_auth<B: Serialize, T: DeserializeOwned>(
    url: &str,
    jwt: &str,
    body: &B,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|e| e.to_string())?;
    json(Method::POST, url, Some(jwt), Some(body)).await
}

pub async fn put_json_auth<B: Serialize, T: DeserializeOwned>(
    url: &str,
    jwt: &str,
    body: &B,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|e| e.to_string())?;
    json(Method::PUT, url, Some(jwt), Some(body)).await
}

pub async fn patch_json_auth<B: Serialize, T: DeserializeOwned>(
    url: &str,
    jwt: &str,
    body: &B,
) -> Result<T, String> {
    let body = serde_json::to_string(body).map_err(|e| e.to_string())?;
    json(Method::PATCH, url, Some(jwt), Some(body)).await
}

pub async fn patch_ok_auth<B: Serialize>(url: &str, jwt: &str, body: &B) -> Result<(), String> {
    let body = serde_json::to_string(body).map_err(|e| e.to_string())?;
    ok(Method::PATCH, url, Some(jwt), Some(body)).await
}

pub async fn delete_ok_auth(url: &str, jwt: &str) -> Result<(), String> {
    ok(Method::DELETE, url, Some(jwt), None).await
}
