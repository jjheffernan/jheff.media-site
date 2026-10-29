use crate::{
    components::admin::library_import::LibraryImport,
    components::ui::{Heading, HeadingLevel, Text, TextTone},
    context::AuthContext,
    model::{AdminPrintOrder, ContentCollection},
    services::auth_header,
};
use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{FormData, HtmlInputElement};
use yew::prelude::*;

fn starts_at_with_offset(value: &str) -> String {
    let value = value.trim();
    if value.len() < 16 || value.contains('Z') || value.contains('+') || value[10..].contains('-') {
        return value.to_string();
    }
    let date = js_sys::Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.to_string();
    }
    let offset_min = date.get_timezone_offset() as i32;
    let sign = if offset_min > 0 { '-' } else { '+' };
    let abs = offset_min.unsigned_abs();
    let stamp = if value.len() == 16 {
        format!("{value}:00")
    } else {
        value.to_string()
    };
    format!("{stamp}{sign}{:02}:{:02}", abs / 60, abs % 60)
}

fn opt_str(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

async fn admin_json(
    method: &str,
    url: &str,
    jwt: &str,
    body: Option<String>,
) -> Result<(), String> {
    let auth_val = auth_header(jwt);
    let builder = match method {
        "PATCH" => Request::patch(url),
        "PUT" => Request::put(url),
        "DELETE" => Request::delete(url),
        "POST" => Request::post(url),
        _ => return Err("Unsupported method.".into()),
    }
    .header("Authorization", auth_val.as_str());
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
    if resp.ok() {
        Ok(())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Request failed.")
            .to_string())
    } else {
        Err("Request failed.".into())
    }
}

async fn create_schedule_event(jwt: &str, payload: serde_json::Value) -> Result<String, String> {
    let auth_val = auth_header(jwt);
    let req = Request::post("/api/admin/schedule")
        .header("Authorization", auth_val.as_str())
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        let body = resp
            .json::<serde_json::Value>()
            .await
            .map_err(|_| "Could not read response.".to_string())?;
        body.get("data")
            .and_then(|data| data.get("id"))
            .and_then(|id| id.as_str())
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .ok_or_else(|| "Failed to add date.".to_string())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Failed to add date.")
            .to_string())
    } else {
        Err("Failed to add date.".into())
    }
}

fn schedule_payload(
    title: &str,
    starts: &str,
    location: &str,
    kind: &str,
) -> Result<serde_json::Value, String> {
    let title = title.trim().to_string();
    let starts_at = starts_at_with_offset(starts);
    let location = location.trim().to_string();
    let kind = if kind.trim().is_empty() {
        "shoot".to_string()
    } else {
        kind.trim().to_string()
    };
    if title.is_empty() || starts_at.is_empty() {
        return Err("Title and start time are required.".into());
    }
    let mut payload = serde_json::json!({
        "title": title,
        "startsAt": starts_at,
        "kind": kind,
    });
    if !location.is_empty() {
        payload["location"] = serde_json::Value::String(location);
    }
    Ok(payload)
}

async fn update_schedule_event(
    jwt: &str,
    id: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Enter a date ID.".into());
    }
    let url = format!("/api/admin/schedule/{id}");
    let auth_val = auth_header(jwt);
    let req = Request::patch(&url)
        .header("Authorization", auth_val.as_str())
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Failed to update date.")
            .to_string())
    } else {
        Err("Failed to update date.".into())
    }
}

async fn delete_schedule_event(jwt: &str, id: &str) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Enter a date ID.".into());
    }
    admin_json("DELETE", &format!("/api/admin/schedule/{id}"), jwt, None).await
}

async fn upload_collection_media(
    jwt: &str,
    kind: &str,
    collection_id: &str,
    files: Vec<web_sys::File>,
    set_as_cover: bool,
    featured: bool,
) -> Result<usize, String> {
    if collection_id.trim().is_empty() {
        return Err("Enter a collection ID first.".into());
    }
    if files.is_empty() {
        return Err("Choose at least one file.".into());
    }

    let form_data = FormData::new().map_err(|_| "Could not build upload form.".to_string())?;
    for file in &files {
        form_data
            .append_with_blob("file", file)
            .map_err(|_| "Could not attach file.".to_string())?;
    }
    if set_as_cover {
        form_data
            .append_with_str("setAsCover", "true")
            .map_err(|_| "Could not attach cover flag.".to_string())?;
    }
    if featured {
        form_data
            .append_with_str("featured", "true")
            .map_err(|_| "Could not attach featured flag.".to_string())?;
    }

    let url = format!("/api/admin/{kind}/{collection_id}/media");
    let auth_val = auth_header(jwt);
    let req = Request::post(&url)
        .header("Authorization", auth_val.as_str())
        .body(JsValue::from(form_data))
        .map_err(|e| e.to_string())?;

    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(files.len())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Upload failed.")
            .to_string())
    } else {
        Err("Upload failed.".into())
    }
}

fn collect_files(list: &web_sys::FileList) -> Vec<web_sys::File> {
    let mut out = Vec::new();
    for i in 0..list.length() {
        if let Some(f) = list.get(i) {
            out.push(f);
        }
    }
    out
}

fn bind_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        state.set(e.target_unchecked_into::<HtmlInputElement>().value());
    })
}

fn bind_check(state: UseStateHandle<bool>) -> Callback<Event> {
    Callback::from(move |e: Event| {
        state.set(e.target_unchecked_into::<HtmlInputElement>().checked());
    })
}

#[function_component(ContentAdmin)]
pub fn content_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let message = use_state(|| None::<String>);
    let gallery_id = use_state(String::new);
    let gallery_title = use_state(String::new);
    let gallery_summary = use_state(String::new);
    let gallery_date = use_state(String::new);
    let gallery_location = use_state(String::new);
    let gallery_status = use_state(|| String::from("published"));
    let gallery_media_id = use_state(String::new);
    let gallery_order = use_state(String::new);
    let shoot_id = use_state(String::new);
    let shoot_title = use_state(String::new);
    let shoot_summary = use_state(String::new);
    let shoot_date = use_state(String::new);
    let shoot_location = use_state(String::new);
    let shoot_status = use_state(|| String::from("scheduled"));
    let shoot_media_id = use_state(String::new);
    let shoot_order = use_state(String::new);
    let date_title = use_state(String::new);
    let date_starts = use_state(String::new);
    let date_location = use_state(String::new);
    let date_kind = use_state(|| String::from("shoot"));
    let date_id = use_state(String::new);
    let gallery_cover = use_state(|| false);
    let gallery_featured = use_state(|| false);
    let shoot_cover = use_state(|| false);
    let shoot_featured = use_state(|| false);

    if auth_ctx.auth.as_ref().map(|a| a.user.role.as_str()) != Some("admin") {
        return html! {};
    }

    let on_create_gallery = {
        let auth_ctx = auth_ctx.clone();
        let gallery_id = gallery_id.clone();
        let gallery_title = gallery_title.clone();
        let gallery_summary = gallery_summary.clone();
        let gallery_date = gallery_date.clone();
        let gallery_location = gallery_location.clone();
        let gallery_status = gallery_status.clone();
        let message = message.clone();
        Callback::from(move |_| {
            if let Some(auth) = &auth_ctx.auth {
                let collection = ContentCollection {
                    id: (*gallery_id).clone(),
                    title: (*gallery_title).clone(),
                    summary: opt_str(&gallery_summary),
                    cover_url: None,
                    status: opt_str(&gallery_status).or(Some("published".into())),
                    date: opt_str(&gallery_date),
                    location: opt_str(&gallery_location),
                    media: vec![],
                };
                let jwt = auth.jwt.clone();
                let message = message.clone();
                spawn_local(async move {
                    if let Ok(body) = serde_json::to_string(&collection) {
                        match admin_json("POST", "/api/admin/galleries", &jwt, Some(body)).await {
                            Ok(()) => message.set(Some(
                                "Gallery created. You can upload images below.".into(),
                            )),
                            Err(err) => message.set(Some(err)),
                        }
                    }
                });
            }
        })
    };

    let on_create_shoot = {
        let auth_ctx = auth_ctx.clone();
        let shoot_id = shoot_id.clone();
        let shoot_title = shoot_title.clone();
        let shoot_summary = shoot_summary.clone();
        let shoot_date = shoot_date.clone();
        let shoot_location = shoot_location.clone();
        let shoot_status = shoot_status.clone();
        let message = message.clone();
        Callback::from(move |_| {
            if let Some(auth) = &auth_ctx.auth {
                let collection = ContentCollection {
                    id: (*shoot_id).clone(),
                    title: (*shoot_title).clone(),
                    summary: opt_str(&shoot_summary),
                    cover_url: None,
                    status: opt_str(&shoot_status).or(Some("scheduled".into())),
                    date: opt_str(&shoot_date),
                    location: opt_str(&shoot_location),
                    media: vec![],
                };
                let jwt = auth.jwt.clone();
                let message = message.clone();
                spawn_local(async move {
                    if let Ok(body) = serde_json::to_string(&collection) {
                        match admin_json("POST", "/api/admin/shoots", &jwt, Some(body)).await {
                            Ok(()) => message
                                .set(Some("Shoot created. You can upload images below.".into())),
                            Err(err) => message.set(Some(err)),
                        }
                    }
                });
            }
        })
    };

    let on_save_gallery = {
        let auth_ctx = auth_ctx.clone();
        let gallery_id = gallery_id.clone();
        let gallery_title = gallery_title.clone();
        let gallery_summary = gallery_summary.clone();
        let gallery_date = gallery_date.clone();
        let gallery_location = gallery_location.clone();
        let gallery_status = gallery_status.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let id = gallery_id.trim().to_string();
            if id.is_empty() {
                message.set(Some("Enter a gallery ID.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let patch = serde_json::json!({
                "title": gallery_title.trim(),
                "summary": gallery_summary.trim(),
                "date": gallery_date.trim(),
                "location": gallery_location.trim(),
                "status": gallery_status.trim(),
            });
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match admin_json(
                    "PATCH",
                    &format!("/api/admin/galleries/{id}"),
                    &jwt,
                    Some(patch.to_string()),
                )
                .await
                {
                    Ok(()) => message.set(Some(format!("Gallery \"{id}\" updated."))),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_save_shoot = {
        let auth_ctx = auth_ctx.clone();
        let shoot_id = shoot_id.clone();
        let shoot_title = shoot_title.clone();
        let shoot_summary = shoot_summary.clone();
        let shoot_date = shoot_date.clone();
        let shoot_location = shoot_location.clone();
        let shoot_status = shoot_status.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let id = shoot_id.trim().to_string();
            if id.is_empty() {
                message.set(Some("Enter a shoot ID.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let patch = serde_json::json!({
                "title": shoot_title.trim(),
                "summary": shoot_summary.trim(),
                "date": shoot_date.trim(),
                "location": shoot_location.trim(),
                "status": shoot_status.trim(),
            });
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match admin_json(
                    "PATCH",
                    &format!("/api/admin/shoots/{id}"),
                    &jwt,
                    Some(patch.to_string()),
                )
                .await
                {
                    Ok(()) => message.set(Some(format!("Shoot \"{id}\" updated."))),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_delete_gallery = {
        let auth_ctx = auth_ctx.clone();
        let gallery_id = gallery_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let id = gallery_id.trim().to_string();
            if id.is_empty() {
                message.set(Some("Enter a gallery ID.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match admin_json("DELETE", &format!("/api/admin/galleries/{id}"), &jwt, None).await
                {
                    Ok(()) => message.set(Some(format!("Gallery \"{id}\" deleted."))),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_delete_shoot = {
        let auth_ctx = auth_ctx.clone();
        let shoot_id = shoot_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let id = shoot_id.trim().to_string();
            if id.is_empty() {
                message.set(Some("Enter a shoot ID.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match admin_json("DELETE", &format!("/api/admin/shoots/{id}"), &jwt, None).await {
                    Ok(()) => message.set(Some(format!("Shoot \"{id}\" deleted."))),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_delete_gallery_media = {
        let auth_ctx = auth_ctx.clone();
        let gallery_id = gallery_id.clone();
        let gallery_media_id = gallery_media_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let cid = gallery_id.trim().to_string();
            let aid = gallery_media_id.trim().to_string();
            if cid.is_empty() || aid.is_empty() {
                message.set(Some("Enter gallery ID and media ID.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let message = message.clone();
            let gallery_media_id = gallery_media_id.clone();
            spawn_local(async move {
                match admin_json(
                    "DELETE",
                    &format!("/api/admin/galleries/{cid}/media/{aid}"),
                    &jwt,
                    None,
                )
                .await
                {
                    Ok(()) => {
                        gallery_media_id.set(String::new());
                        message.set(Some("Media removed.".into()));
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_delete_shoot_media = {
        let auth_ctx = auth_ctx.clone();
        let shoot_id = shoot_id.clone();
        let shoot_media_id = shoot_media_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let cid = shoot_id.trim().to_string();
            let aid = shoot_media_id.trim().to_string();
            if cid.is_empty() || aid.is_empty() {
                message.set(Some("Enter shoot ID and media ID.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let message = message.clone();
            let shoot_media_id = shoot_media_id.clone();
            spawn_local(async move {
                match admin_json(
                    "DELETE",
                    &format!("/api/admin/shoots/{cid}/media/{aid}"),
                    &jwt,
                    None,
                )
                .await
                {
                    Ok(()) => {
                        shoot_media_id.set(String::new());
                        message.set(Some("Media removed.".into()));
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_reorder_gallery = {
        let auth_ctx = auth_ctx.clone();
        let gallery_id = gallery_id.clone();
        let gallery_order = gallery_order.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let cid = gallery_id.trim().to_string();
            if cid.is_empty() {
                message.set(Some("Enter a gallery ID.".into()));
                return;
            }
            let media_ids: Vec<String> = gallery_order
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let body = serde_json::json!({ "mediaIds": media_ids }).to_string();
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match admin_json(
                    "PUT",
                    &format!("/api/admin/galleries/{cid}/media/order"),
                    &jwt,
                    Some(body),
                )
                .await
                {
                    Ok(()) => message.set(Some("Gallery media reordered.".into())),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_reorder_shoot = {
        let auth_ctx = auth_ctx.clone();
        let shoot_id = shoot_id.clone();
        let shoot_order = shoot_order.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let cid = shoot_id.trim().to_string();
            if cid.is_empty() {
                message.set(Some("Enter a shoot ID.".into()));
                return;
            }
            let media_ids: Vec<String> = shoot_order
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let body = serde_json::json!({ "mediaIds": media_ids }).to_string();
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match admin_json(
                    "PUT",
                    &format!("/api/admin/shoots/{cid}/media/order"),
                    &jwt,
                    Some(body),
                )
                .await
                {
                    Ok(()) => message.set(Some("Shoot media reordered.".into())),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_gallery_files = {
        let auth_ctx = auth_ctx.clone();
        let gallery_id = gallery_id.clone();
        let message = message.clone();
        let gallery_cover = gallery_cover.clone();
        let gallery_featured = gallery_featured.clone();
        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            let Some(file_list) = input.files() else {
                return;
            };
            let files = collect_files(&file_list);
            if files.is_empty() {
                return;
            }
            if let Some(auth) = &auth_ctx.auth {
                let jwt = auth.jwt.clone();
                let collection_id = (*gallery_id).clone();
                let message = message.clone();
                let set_as_cover = *gallery_cover;
                let featured = *gallery_featured;
                let gallery_cover = gallery_cover.clone();
                let gallery_featured = gallery_featured.clone();
                spawn_local(async move {
                    match upload_collection_media(
                        &jwt,
                        "galleries",
                        &collection_id,
                        files,
                        set_as_cover,
                        featured,
                    )
                    .await
                    {
                        Ok(n) => {
                            gallery_cover.set(false);
                            gallery_featured.set(false);
                            message.set(Some(format!(
                                "Uploaded {n} file(s) to gallery \"{collection_id}\"."
                            )));
                        }
                        Err(err) => message.set(Some(err)),
                    }
                });
            }
            input.set_value("");
        })
    };

    let on_shoot_files = {
        let auth_ctx = auth_ctx.clone();
        let shoot_id = shoot_id.clone();
        let message = message.clone();
        let shoot_cover = shoot_cover.clone();
        let shoot_featured = shoot_featured.clone();
        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            let Some(file_list) = input.files() else {
                return;
            };
            let files = collect_files(&file_list);
            if files.is_empty() {
                return;
            }
            if let Some(auth) = &auth_ctx.auth {
                let jwt = auth.jwt.clone();
                let collection_id = (*shoot_id).clone();
                let message = message.clone();
                let set_as_cover = *shoot_cover;
                let featured = *shoot_featured;
                let shoot_cover = shoot_cover.clone();
                let shoot_featured = shoot_featured.clone();
                spawn_local(async move {
                    match upload_collection_media(
                        &jwt,
                        "shoots",
                        &collection_id,
                        files,
                        set_as_cover,
                        featured,
                    )
                    .await
                    {
                        Ok(n) => {
                            shoot_cover.set(false);
                            shoot_featured.set(false);
                            message.set(Some(format!(
                                "Uploaded {n} file(s) to shoot \"{collection_id}\"."
                            )));
                        }
                        Err(err) => message.set(Some(err)),
                    }
                });
            }
            input.set_value("");
        })
    };

    let on_add_date = {
        let auth_ctx = auth_ctx.clone();
        let date_title = date_title.clone();
        let date_starts = date_starts.clone();
        let date_location = date_location.clone();
        let date_kind = date_kind.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let title = date_title.trim().to_string();
            let starts_at = starts_at_with_offset(&date_starts);
            let location = date_location.trim().to_string();
            let kind = if date_kind.trim().is_empty() {
                "shoot".to_string()
            } else {
                date_kind.trim().to_string()
            };
            if title.is_empty() || starts_at.is_empty() {
                message.set(Some("Title and start time are required.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let mut payload = serde_json::json!({
                "title": title,
                "startsAt": starts_at,
                "kind": kind,
            });
            if !location.is_empty() {
                payload["location"] = serde_json::Value::String(location);
            }
            let jwt = auth.jwt.clone();
            let message = message.clone();
            let date_title = date_title.clone();
            let date_starts = date_starts.clone();
            let date_location = date_location.clone();
            spawn_local(async move {
                match create_schedule_event(&jwt, payload).await {
                    Ok(id) => {
                        date_title.set(String::new());
                        date_starts.set(String::new());
                        date_location.set(String::new());
                        message.set(Some(format!("Date added: {id}")));
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_kind = {
        let date_kind = date_kind.clone();
        Callback::from(move |e: Event| {
            date_kind.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_save_date = {
        let auth_ctx = auth_ctx.clone();
        let date_title = date_title.clone();
        let date_starts = date_starts.clone();
        let date_location = date_location.clone();
        let date_kind = date_kind.clone();
        let date_id = date_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let payload = match schedule_payload(&date_title, &date_starts, &date_location, &date_kind)
            {
                Ok(payload) => payload,
                Err(err) => {
                    message.set(Some(err));
                    return;
                }
            };
            let id = (*date_id).clone();
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let message = message.clone();
            spawn_local(async move {
                match update_schedule_event(&jwt, &id, payload).await {
                    Ok(()) => message.set(Some("Date updated.".into())),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_delete_date = {
        let auth_ctx = auth_ctx.clone();
        let date_id = date_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let id = (*date_id).clone();
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let message = message.clone();
            let date_id = date_id.clone();
            spawn_local(async move {
                match delete_schedule_event(&jwt, &id).await {
                    Ok(()) => {
                        date_id.set(String::new());
                        message.set(Some("Date removed.".into()));
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    html! {
        <section class="card space-y-6">
            <Heading level={HeadingLevel::H2} subtitle="Admin only — create, edit, upload, reorder, delete. Mongo collections only.">
                { "Content management" }
            </Heading>

            if let Some(msg) = &*message {
                <p class="text-sm text-accent">{ msg }</p>
            }

            <div class="grid gap-6 sm:grid-cols-2">
                <div class="space-y-3">
                    <Text tone={TextTone::Muted}>{ "Gallery" }</Text>
                    <input type="text" placeholder="ID (slug)" class="field" value={(*gallery_id).clone()} oninput={bind_input(gallery_id.clone())} />
                    <input type="text" placeholder="Title" class="field" value={(*gallery_title).clone()} oninput={bind_input(gallery_title.clone())} />
                    <input type="text" placeholder="Summary" class="field" value={(*gallery_summary).clone()} oninput={bind_input(gallery_summary.clone())} />
                    <input type="text" placeholder="Date" class="field" value={(*gallery_date).clone()} oninput={bind_input(gallery_date.clone())} />
                    <input type="text" placeholder="Location" class="field" value={(*gallery_location).clone()} oninput={bind_input(gallery_location.clone())} />
                    <input type="text" placeholder="Status" class="field" value={(*gallery_status).clone()} oninput={bind_input(gallery_status.clone())} />
                    <div class="flex flex-wrap gap-2">
                        <button type="button" class="btn btn-primary" onclick={on_create_gallery}>{ "Create" }</button>
                        <button type="button" class="btn btn-quiet" onclick={on_save_gallery}>{ "Save" }</button>
                        <button type="button" class="btn btn-quiet" onclick={on_delete_gallery}>{ "Delete" }</button>
                    </div>
                    <label class="muted">
                        <input type="checkbox" checked={*gallery_cover} onchange={bind_check(gallery_cover.clone())} />
                        { " Set as cover (first file only)" }
                    </label>
                    <label class="muted">
                        <input type="checkbox" checked={*gallery_featured} onchange={bind_check(gallery_featured.clone())} />
                        { " Featured (all files in upload)" }
                    </label>
                    <label class="block space-y-1">
                        <Text tone={TextTone::Muted}>{ "Upload images (uses ID above)" }</Text>
                        <input
                            type="file"
                            multiple=true
                            accept="image/jpeg,image/png,image/webp,image/gif"
                            class="block w-full text-sm text-muted file:mr-3 file:rounded-lg file:border-0 file:bg-surface file:px-3 file:py-2"
                            onchange={on_gallery_files}
                        />
                    </label>
                    <input type="text" placeholder="Media asset ID" class="field" value={(*gallery_media_id).clone()} oninput={bind_input(gallery_media_id.clone())} />
                    <button type="button" class="btn btn-quiet" onclick={on_delete_gallery_media}>{ "Delete media" }</button>
                    <input type="text" placeholder="Reorder: id1, id2, …" class="field" value={(*gallery_order).clone()} oninput={bind_input(gallery_order.clone())} />
                    <button type="button" class="btn btn-quiet" onclick={on_reorder_gallery}>{ "Reorder media" }</button>
                </div>

                <div class="space-y-3">
                    <Text tone={TextTone::Muted}>{ "Shoot" }</Text>
                    <input type="text" placeholder="ID (slug)" class="field" value={(*shoot_id).clone()} oninput={bind_input(shoot_id.clone())} />
                    <input type="text" placeholder="Title" class="field" value={(*shoot_title).clone()} oninput={bind_input(shoot_title.clone())} />
                    <input type="text" placeholder="Summary" class="field" value={(*shoot_summary).clone()} oninput={bind_input(shoot_summary.clone())} />
                    <input type="text" placeholder="Date" class="field" value={(*shoot_date).clone()} oninput={bind_input(shoot_date.clone())} />
                    <input type="text" placeholder="Location" class="field" value={(*shoot_location).clone()} oninput={bind_input(shoot_location.clone())} />
                    <input type="text" placeholder="Status" class="field" value={(*shoot_status).clone()} oninput={bind_input(shoot_status.clone())} />
                    <div class="flex flex-wrap gap-2">
                        <button type="button" class="btn btn-primary" onclick={on_create_shoot}>{ "Create" }</button>
                        <button type="button" class="btn btn-quiet" onclick={on_save_shoot}>{ "Save" }</button>
                        <button type="button" class="btn btn-quiet" onclick={on_delete_shoot}>{ "Delete" }</button>
                    </div>
                    <label class="muted">
                        <input type="checkbox" checked={*shoot_cover} onchange={bind_check(shoot_cover.clone())} />
                        { " Set as cover (first file only)" }
                    </label>
                    <label class="muted">
                        <input type="checkbox" checked={*shoot_featured} onchange={bind_check(shoot_featured.clone())} />
                        { " Featured (all files in upload)" }
                    </label>
                    <label class="block space-y-1">
                        <Text tone={TextTone::Muted}>{ "Upload images (uses ID above)" }</Text>
                        <input
                            type="file"
                            multiple=true
                            accept="image/jpeg,image/png,image/webp,image/gif"
                            class="block w-full text-sm text-muted file:mr-3 file:rounded-lg file:border-0 file:bg-surface file:px-3 file:py-2"
                            onchange={on_shoot_files}
                        />
                    </label>
                    <input type="text" placeholder="Media asset ID" class="field" value={(*shoot_media_id).clone()} oninput={bind_input(shoot_media_id.clone())} />
                    <button type="button" class="btn btn-quiet" onclick={on_delete_shoot_media}>{ "Delete media" }</button>
                    <input type="text" placeholder="Reorder: id1, id2, …" class="field" value={(*shoot_order).clone()} oninput={bind_input(shoot_order.clone())} />
                    <button type="button" class="btn btn-quiet" onclick={on_reorder_shoot}>{ "Reorder media" }</button>
                </div>
            </div>

            <LibraryImport />

            <div class="space-y-3">
                <Text tone={TextTone::Muted}>{ "Add date" }</Text>
                <input
                    type="text"
                    placeholder="Title"
                    class="field"
                    value={(*date_title).clone()}
                    oninput={bind_input(date_title.clone())}
                />
                <label class="block space-y-1">
                    <Text tone={TextTone::Muted}>{ "Starts" }</Text>
                    <input
                        type="datetime-local"
                        class="field"
                        value={(*date_starts).clone()}
                        oninput={bind_input(date_starts.clone())}
                    />
                </label>
                <input
                    type="text"
                    placeholder="Location"
                    class="field"
                    value={(*date_location).clone()}
                    oninput={bind_input(date_location.clone())}
                />
                <label class="block space-y-1">
                    <Text tone={TextTone::Muted}>{ "Kind" }</Text>
                    <select
                        class="field"
                        value={(*date_kind).clone()}
                        onchange={on_kind}
                    >
                        <option value="shoot">{ "Shoot" }</option>
                        <option value="instruction">{ "Instruction" }</option>
                        <option value="other">{ "Other" }</option>
                    </select>
                </label>
                <button
                    type="button"
                    class="btn btn-primary"
                    onclick={on_add_date}
                >
                    { "Add date" }
                </button>
                <input
                    type="text"
                    placeholder="Date ID"
                    class="field"
                    value={(*date_id).clone()}
                    oninput={bind_input(date_id.clone())}
                />
                <button
                    type="button"
                    class="btn btn-primary"
                    onclick={on_save_date}
                >
                    { "Save" }
                </button>
                <button
                    type="button"
                    class="btn btn-quiet"
                    onclick={on_delete_date}
                >
                    { "Delete" }
                </button>
            </div>

            <PrintOrdersAdmin />
        </section>
    }
}

fn order_who(order: &AdminPrintOrder) -> String {
    match (&order.email, &order.username) {
        (Some(email), Some(username)) => format!("{email} ({username})"),
        (Some(email), None) => email.clone(),
        (None, Some(username)) => username.clone(),
        (None, None) => order.user_id.clone(),
    }
}

async fn load_print_orders(jwt: &str) -> Result<Vec<AdminPrintOrder>, String> {
    let auth_val = auth_header(jwt);
    let resp = Request::get("/api/admin/prints/orders")
        .header("Authorization", auth_val.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json().await.map_err(|_| "Could not read orders.".to_string())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Could not load orders.")
            .to_string())
    } else {
        Err("Could not load orders.".into())
    }
}

async fn patch_print_order(
    jwt: &str,
    id: &str,
    status: &str,
    payment_status: &str,
) -> Result<AdminPrintOrder, String> {
    let url = format!("/api/admin/prints/orders/{id}");
    let payload = serde_json::json!({
        "status": status,
        "paymentStatus": payment_status,
    });
    let auth_val = auth_header(jwt);
    let req = Request::patch(&url)
        .header("Authorization", auth_val.as_str())
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .map_err(|e| e.to_string())?;
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json().await.map_err(|_| "Could not read order.".to_string())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Could not update order.")
            .to_string())
    } else {
        Err("Could not update order.".into())
    }
}

#[function_component(PrintOrdersAdmin)]
fn print_orders_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let orders = use_state(|| None::<Vec<AdminPrintOrder>>);
    let note = use_state(|| None::<String>);
    let jwt = auth_ctx
        .auth
        .as_ref()
        .map(|auth| auth.jwt.clone())
        .unwrap_or_default();

    {
        let orders = orders.clone();
        let note = note.clone();
        use_effect_with(jwt.clone(), move |jwt| {
            if !jwt.is_empty() {
                let jwt = jwt.clone();
                spawn_local(async move {
                    match load_print_orders(&jwt).await {
                        Ok(items) => orders.set(Some(items)),
                        Err(err) => {
                            orders.set(Some(Vec::new()));
                            note.set(Some(err));
                        }
                    }
                });
            }
            || ()
        });
    }

    let list = match &*orders {
        None => {
            return html! {
                <div class="space-y-3">
                    <Text tone={TextTone::Muted}>{ "Print orders" }</Text>
                    <p class="text-sm text-muted">{ "Loading orders…" }</p>
                </div>
            };
        }
        Some(list) => list.clone(),
    };

    html! {
        <div class="space-y-3">
            <Text tone={TextTone::Muted}>{ "Print orders" }</Text>
            if let Some(msg) = &*note {
                <p class="text-sm text-accent">{ msg }</p>
            }
            if list.is_empty() {
                <p class="text-sm text-muted">{ "No print orders." }</p>
            }
            { for list.iter().map(|order| {
                let id = order.id.clone();
                let on_status = {
                    let id = id.clone();
                    let orders = orders.clone();
                    Callback::from(move |e: Event| {
                        let value = e.target_unchecked_into::<HtmlInputElement>().value();
                        let mut next = (*orders).clone().unwrap_or_default();
                        if let Some(order) = next.iter_mut().find(|order| order.id == id) {
                            order.status = value;
                        }
                        orders.set(Some(next));
                    })
                };
                let on_payment = {
                    let id = id.clone();
                    let orders = orders.clone();
                    Callback::from(move |e: Event| {
                        let value = e.target_unchecked_into::<HtmlInputElement>().value();
                        let mut next = (*orders).clone().unwrap_or_default();
                        if let Some(order) = next.iter_mut().find(|order| order.id == id) {
                            order.payment_status = value;
                        }
                        orders.set(Some(next));
                    })
                };
                let on_save = {
                    let id = id.clone();
                    let orders = orders.clone();
                    let jwt = jwt.clone();
                    let note = note.clone();
                    Callback::from(move |_| {
                        let Some(current) = (*orders).clone() else { return };
                        let Some(order) = current.iter().find(|order| order.id == id) else { return };
                        let status = order.status.clone();
                        let payment = order.payment_status.clone();
                        let id = id.clone();
                        let jwt = jwt.clone();
                        let orders = orders.clone();
                        let note = note.clone();
                        spawn_local(async move {
                            match patch_print_order(&jwt, &id, &status, &payment).await {
                                Ok(updated) => {
                                    let mut next = (*orders).clone().unwrap_or_default();
                                    if let Some(slot) = next.iter_mut().find(|order| order.id == updated.id) {
                                        *slot = updated;
                                    }
                                    orders.set(Some(next));
                                    note.set(Some("Order updated.".into()));
                                }
                                Err(err) => note.set(Some(err)),
                            }
                        });
                    })
                };
                let photos = {
                    let titles: Vec<&str> = order
                        .items
                        .iter()
                        .filter_map(|item| item.title.as_deref().filter(|title| !title.is_empty()))
                        .collect();
                    if titles.is_empty() {
                        format!("{} photos", order.item_count)
                    } else {
                        titles.join(", ")
                    }
                };
                html! {
                    <div key={order.id.clone()} class="space-y-2">
                        <p class="text-sm text-muted">
                            { format!(
                                "{} · {} · ${} · {}",
                                order_who(order),
                                order.size,
                                order.total_cents / 100,
                                photos
                            ) }
                        </p>
                        <select class="field" value={order.status.clone()} onchange={on_status}>
                            <option value="requested" selected={order.status == "requested"}>{ "requested" }</option>
                            <option value="fulfilled" selected={order.status == "fulfilled"}>{ "fulfilled" }</option>
                            <option value="cancelled" selected={order.status == "cancelled"}>{ "cancelled" }</option>
                        </select>
                        <select class="field" value={order.payment_status.clone()} onchange={on_payment}>
                            <option value="unpaid" selected={order.payment_status == "unpaid"}>{ "unpaid" }</option>
                            <option value="invoiced" selected={order.payment_status == "invoiced"}>{ "invoiced" }</option>
                            <option value="paid" selected={order.payment_status == "paid"}>{ "paid" }</option>
                        </select>
                        <button type="button" class="btn btn-quiet" onclick={on_save}>{ "Save" }</button>
                    </div>
                }
            }) }
        </div>
    }
}
