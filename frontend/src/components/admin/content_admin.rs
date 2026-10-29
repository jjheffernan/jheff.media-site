use crate::{
    components::ui::{Heading, HeadingLevel, Text, TextTone},
    context::AuthContext,
    model::ContentCollection,
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

async fn delete_schedule_event(jwt: &str, id: &str) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Enter a date ID.".into());
    }
    let url = format!("/api/admin/schedule/{id}");
    let auth_val = auth_header(jwt);
    let resp = Request::delete(&url)
        .header("Authorization", auth_val.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Failed to delete date.")
            .to_string())
    } else {
        Err("Failed to delete date.".into())
    }
}

async fn upload_collection_media(
    jwt: &str,
    kind: &str,
    collection_id: &str,
    file: web_sys::File,
    set_as_cover: bool,
    featured: bool,
) -> Result<(), String> {
    if collection_id.trim().is_empty() {
        return Err("Enter a collection ID first.".into());
    }

    let form_data = FormData::new().map_err(|_| "Could not build upload form.".to_string())?;
    form_data
        .append_with_blob("file", &file)
        .map_err(|_| "Could not attach file.".to_string())?;
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
        Ok(())
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

#[function_component(ContentAdmin)]
pub fn content_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let message = use_state(|| None::<String>);
    let gallery_id = use_state(String::new);
    let gallery_title = use_state(String::new);
    let shoot_id = use_state(String::new);
    let shoot_title = use_state(String::new);
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
        let message = message.clone();
        Callback::from(move |_| {
            if let Some(auth) = &auth_ctx.auth {
                let collection = ContentCollection {
                    id: (*gallery_id).clone(),
                    title: (*gallery_title).clone(),
                    summary: None,
                    cover_url: None,
                    status: Some("published".into()),
                    date: None,
                    location: None,
                    media: vec![],
                };
                let jwt = auth.jwt.clone();
                let message = message.clone();
                spawn_local(async move {
                    if let Ok(body) = serde_json::to_string(&collection) {
                        let auth_val = auth_header(&jwt);
                        let result = Request::post("/api/admin/galleries")
                            .header("Authorization", auth_val.as_str())
                            .header("Content-Type", "application/json")
                            .body(body);
                        if let Ok(req) = result {
                            if let Ok(resp) = req.send().await {
                                if resp.ok() {
                                    message.set(Some(
                                        "Gallery created. You can upload images below.".into(),
                                    ));
                                } else {
                                    message.set(Some("Failed to create gallery.".into()));
                                }
                            }
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
        let message = message.clone();
        Callback::from(move |_| {
            if let Some(auth) = &auth_ctx.auth {
                let collection = ContentCollection {
                    id: (*shoot_id).clone(),
                    title: (*shoot_title).clone(),
                    summary: None,
                    cover_url: None,
                    status: Some("scheduled".into()),
                    date: None,
                    location: None,
                    media: vec![],
                };
                let jwt = auth.jwt.clone();
                let message = message.clone();
                spawn_local(async move {
                    if let Ok(body) = serde_json::to_string(&collection) {
                        let auth_val = auth_header(&jwt);
                        let result = Request::post("/api/admin/shoots")
                            .header("Authorization", auth_val.as_str())
                            .header("Content-Type", "application/json")
                            .body(body);
                        if let Ok(req) = result {
                            if let Ok(resp) = req.send().await {
                                if resp.ok() {
                                    message.set(Some(
                                        "Shoot created. You can upload images below.".into(),
                                    ));
                                } else {
                                    message.set(Some("Failed to create shoot.".into()));
                                }
                            }
                        }
                    }
                });
            }
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
            let Some(files) = input.files() else {
                return;
            };
            if files.length() == 0 {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };
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
                        file,
                        set_as_cover,
                        featured,
                    )
                    .await
                    {
                        Ok(()) => {
                            gallery_cover.set(false);
                            gallery_featured.set(false);
                            message.set(Some(format!(
                                "Uploaded image to gallery \"{collection_id}\"."
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
            let Some(files) = input.files() else {
                return;
            };
            if files.length() == 0 {
                return;
            };
            let Some(file) = files.get(0) else {
                return;
            };
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
                        file,
                        set_as_cover,
                        featured,
                    )
                    .await
                    {
                        Ok(()) => {
                            shoot_cover.set(false);
                            shoot_featured.set(false);
                            message.set(Some(format!(
                                "Uploaded image to shoot \"{collection_id}\"."
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
            <Heading level={HeadingLevel::H2} subtitle="Admin only — create collections and upload images.">
                { "Content management" }
            </Heading>

            if let Some(msg) = &*message {
                <p class="text-sm text-accent">{ msg }</p>
            }

            <div class="grid gap-6 sm:grid-cols-2">
                <div class="space-y-3">
                    <Text tone={TextTone::Muted}>{ "New gallery" }</Text>
                    <input
                        type="text"
                        placeholder="ID (slug)"
                        class="field"
                        value={(*gallery_id).clone()}
                        oninput={{
                            let gallery_id = gallery_id.clone();
                            Callback::from(move |e: InputEvent| {
                                gallery_id.set(e.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                    <input
                        type="text"
                        placeholder="Title"
                        class="field"
                        value={(*gallery_title).clone()}
                        oninput={{
                            let gallery_title = gallery_title.clone();
                            Callback::from(move |e: InputEvent| {
                                gallery_title.set(e.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                    <button
                        type="button"
                        class="btn btn-primary"
                        onclick={on_create_gallery}
                    >
                        { "Create gallery" }
                    </button>
                    <label class="muted">
                        <input
                            type="checkbox"
                            checked={*gallery_cover}
                            onchange={{
                                let gallery_cover = gallery_cover.clone();
                                Callback::from(move |e: Event| {
                                    gallery_cover.set(
                                        e.target_unchecked_into::<HtmlInputElement>().checked(),
                                    );
                                })
                            }}
                        />
                        { " Set as cover" }
                    </label>
                    <label class="muted">
                        <input
                            type="checkbox"
                            checked={*gallery_featured}
                            onchange={{
                                let gallery_featured = gallery_featured.clone();
                                Callback::from(move |e: Event| {
                                    gallery_featured.set(
                                        e.target_unchecked_into::<HtmlInputElement>().checked(),
                                    );
                                })
                            }}
                        />
                        { " Featured" }
                    </label>
                    <label class="block space-y-1">
                        <Text tone={TextTone::Muted}>{ "Upload images (uses ID above)" }</Text>
                        <input
                            type="file"
                            accept="image/jpeg,image/png,image/webp,image/gif"
                            class="block w-full text-sm text-muted file:mr-3 file:rounded-lg file:border-0 file:bg-surface file:px-3 file:py-2"
                            onchange={on_gallery_files}
                        />
                    </label>
                </div>

                <div class="space-y-3">
                    <Text tone={TextTone::Muted}>{ "New shoot" }</Text>
                    <input
                        type="text"
                        placeholder="ID (slug)"
                        class="field"
                        value={(*shoot_id).clone()}
                        oninput={{
                            let shoot_id = shoot_id.clone();
                            Callback::from(move |e: InputEvent| {
                                shoot_id.set(e.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                    <input
                        type="text"
                        placeholder="Title"
                        class="field"
                        value={(*shoot_title).clone()}
                        oninput={{
                            let shoot_title = shoot_title.clone();
                            Callback::from(move |e: InputEvent| {
                                shoot_title.set(e.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                    <button
                        type="button"
                        class="btn btn-primary"
                        onclick={on_create_shoot}
                    >
                        { "Create shoot" }
                    </button>
                    <label class="muted">
                        <input
                            type="checkbox"
                            checked={*shoot_cover}
                            onchange={{
                                let shoot_cover = shoot_cover.clone();
                                Callback::from(move |e: Event| {
                                    shoot_cover.set(
                                        e.target_unchecked_into::<HtmlInputElement>().checked(),
                                    );
                                })
                            }}
                        />
                        { " Set as cover" }
                    </label>
                    <label class="muted">
                        <input
                            type="checkbox"
                            checked={*shoot_featured}
                            onchange={{
                                let shoot_featured = shoot_featured.clone();
                                Callback::from(move |e: Event| {
                                    shoot_featured.set(
                                        e.target_unchecked_into::<HtmlInputElement>().checked(),
                                    );
                                })
                            }}
                        />
                        { " Featured" }
                    </label>
                    <label class="block space-y-1">
                        <Text tone={TextTone::Muted}>{ "Upload images (uses ID above)" }</Text>
                        <input
                            type="file"
                            accept="image/jpeg,image/png,image/webp,image/gif"
                            class="block w-full text-sm text-muted file:mr-3 file:rounded-lg file:border-0 file:bg-surface file:px-3 file:py-2"
                            onchange={on_shoot_files}
                        />
                    </label>
                </div>
            </div>

            <div class="space-y-3">
                <Text tone={TextTone::Muted}>{ "Add date" }</Text>
                <input
                    type="text"
                    placeholder="Title"
                    class="field"
                    value={(*date_title).clone()}
                    oninput={{
                        let date_title = date_title.clone();
                        Callback::from(move |e: InputEvent| {
                            date_title.set(e.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
                />
                <label class="block space-y-1">
                    <Text tone={TextTone::Muted}>{ "Starts" }</Text>
                    <input
                        type="datetime-local"
                        class="field"
                        value={(*date_starts).clone()}
                        oninput={{
                            let date_starts = date_starts.clone();
                            Callback::from(move |e: InputEvent| {
                                date_starts.set(e.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                </label>
                <input
                    type="text"
                    placeholder="Location"
                    class="field"
                    value={(*date_location).clone()}
                    oninput={{
                        let date_location = date_location.clone();
                        Callback::from(move |e: InputEvent| {
                            date_location.set(e.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
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
                    oninput={{
                        let date_id = date_id.clone();
                        Callback::from(move |e: InputEvent| {
                            date_id.set(e.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
                />
                <button
                    type="button"
                    class="btn btn-quiet"
                    onclick={on_delete_date}
                >
                    { "Delete" }
                </button>
            </div>
        </section>
    }
}
