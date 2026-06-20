use actix_web::{http::StatusCode, web, Error, HttpRequest, HttpResponse};
use awc::Client;
use std::{env, time::Duration};
use url::Url;

const X_FORWARDED_FOR: &str = "x-forwarded-for";
const FORWARD_URL: &str = "http://localhost:8080";

fn with_forward_headers(
    mut forward_req: awc::ClientRequest,
    forward_url: &Url,
    req: &HttpRequest,
) -> awc::ClientRequest {
    forward_req.headers_mut().remove("host");
    forward_req = forward_req.insert_header((
        "Host",
        forward_url.host_str().unwrap_or("localhost"),
    ));

    if let Some(addr) = req.head().peer_addr {
        forward_req = forward_req.insert_header((X_FORWARDED_FOR, format!("{}", addr.ip())));
    }

    forward_req
}

pub async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let query = match req.uri().query() {
        Some(query) => format!("?{}", query),
        None => String::new(),
    };
    let forward_url_str =
        env::var("YEW_FULLSTACK_FORWARD_FRONTEND_URL").unwrap_or(String::from(FORWARD_URL));
    let forward_url = match Url::parse(forward_url_str.as_str()) {
        Ok(url) => url,
        Err(err) => {
            warn!("An error occured while parsing the forward URI: {}", err);
            return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal server error"));
        }
    };

    debug!("forward_url is: {}", forward_url_str);
    let forward_uri = format!("{}{}{}", forward_url_str, req.uri().path(), query);

    let forward_req = client
        .request_from(forward_uri.clone(), req.head())
        .timeout(Duration::from_secs(120))
        .no_decompress();
    let forward_req = with_forward_headers(forward_req, &forward_url, &req);

    debug!("forwarded request to {}", forward_uri);

    let mut res = forward_req
        .send_body(body.clone())
        .await
        .map_err(|e| Error::from(actix_web::error::ErrorBadGateway(e)))?;

    if res.status() == StatusCode::NOT_FOUND {
        info!("Alternatively requesting index.html because of 404");
        let forward_uri = format!("{}{}{}", forward_url_str, "/index.html", query);

        let forward_req = client
            .request_from(forward_uri.clone(), req.head())
            .timeout(Duration::from_secs(120))
            .no_decompress();
        let forward_req = with_forward_headers(forward_req, &forward_url, &req);

        res = forward_req
            .send_body(body)
            .await
            .map_err(|e| Error::from(actix_web::error::ErrorBadGateway(e)))?;
    }

    let mut client_resp = HttpResponse::build(res.status());
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.append_header((header_name.clone(), header_value.clone()));
    }

    let body: web::Bytes = res.body().limit(104_857_600).await?;

    Ok(client_resp.body(body))
}
