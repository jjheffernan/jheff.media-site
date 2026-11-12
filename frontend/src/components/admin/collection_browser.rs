use crate::{
    components::ui::DatePicker,
    context::AuthContext,
    model::MediaItem,
    services::auth_header,
};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{FormData, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BrowserCollection {
    id: String,
    title: String,
    summary: Option<String>,
    cover_url: Option<String>,
    status: Option<String>,
    date: Option<String>,
    location: Option<String>,
    #[serde(default)]
    media: Vec<MediaItem>,
    #[serde(default)]
    locked: bool,
}

#[derive(Deserialize)]
struct ListBody {
    items: Vec<BrowserCollection>,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub kind: &'static str,
    pub noun: &'static str,
    pub plural: &'static str,
}

fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn cover_of(item: &BrowserCollection) -> Option<String> {
    item.cover_url
        .as_ref()
        .filter(|url| !url.is_empty())
        .cloned()
        .or_else(|| item.media.first().map(|media| media.thumbnail_url.clone()))
}

fn status_of(item: &BrowserCollection) -> String {
    item.status.clone().unwrap_or_else(|| "draft".into())
}

async fn api(method: &str, url: &str, jwt: &str, body: Option<String>) -> Result<(), String> {
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
            .map_err(|err| err.to_string())?
            .send()
            .await
            .map_err(|err| err.to_string())?
    } else {
        builder.send().await.map_err(|err| err.to_string())?
    };
    if resp.ok() {
        Ok(())
    } else if let Ok(body) = resp.json::<serde_json::Value>().await {
        Err(body
            .get("message")
            .and_then(|message| message.as_str())
            .unwrap_or("Request failed.")
            .to_string())
    } else {
        Err("Request failed.".into())
    }
}

#[function_component(CollectionBrowser)]
pub fn collection_browser(props: &Props) -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let items = use_state(Vec::<BrowserCollection>::new);
    let open_id = use_state(|| None::<String>);
    let creating = use_state(|| false);
    let tick = use_state(|| 0u32);
    let message = use_state(|| None::<String>);
    let title = use_state(String::new);
    let id_field = use_state(String::new);
    let id_touched = use_state(|| false);
    let summary = use_state(String::new);
    let date = use_state(String::new);
    let location = use_state(String::new);
    let status = use_state(|| "draft".to_string());
    let featured = use_state(|| false);
    let set_cover = use_state(|| false);
    let kind = props.kind;
    let noun = props.noun;
    let plural = props.plural;

    {
        let items = items.clone();
        let message = message.clone();
        let kind = kind.to_string();
        let jwt = auth_ctx.auth.as_ref().map(|auth| auth.jwt.clone());
        use_effect_with((jwt, *tick), move |(jwt, _)| {
            if let Some(jwt) = jwt.clone() {
            let items = items.clone();
            let message = message.clone();
            spawn_local(async move {
                let result = Request::get(&format!("/api/admin/{kind}"))
                    .header("Authorization", &auth_header(&jwt))
                    .send()
                    .await;
                match result {
                    Ok(resp) if resp.ok() => match resp.json::<ListBody>().await {
                        Ok(body) => items.set(body.items),
                        Err(_) => message.set(Some("Could not read collections.".into())),
                    },
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_else(|_| "Request failed.".into());
                        message.set(Some(text));
                    }
                    Err(err) => message.set(Some(err.to_string())),
                }
            });
            }
            || ()
        });
    }

    let open = items.iter().find(|item| Some(&item.id) == (*open_id).as_ref()).cloned();

    let fill = |item: &BrowserCollection, title: &UseStateHandle<String>, summary: &UseStateHandle<String>, date: &UseStateHandle<String>, location: &UseStateHandle<String>, status: &UseStateHandle<String>| {
        title.set(item.title.clone());
        summary.set(item.summary.clone().unwrap_or_default());
        date.set(item.date.clone().unwrap_or_default());
        location.set(item.location.clone().unwrap_or_default());
        status.set(status_of(item));
    };

    let on_open = {
        let open_id = open_id.clone();
        let creating = creating.clone();
        let message = message.clone();
        let title = title.clone();
        let summary = summary.clone();
        let date = date.clone();
        let location = location.clone();
        let status = status.clone();
        let items = items.clone();
        Callback::from(move |id: String| {
            if let Some(item) = items.iter().find(|item| item.id == id) {
                fill(item, &title, &summary, &date, &location, &status);
            }
            creating.set(false);
            message.set(None);
            open_id.set(Some(id));
        })
    };

    let on_back = {
        let open_id = open_id.clone();
        let creating = creating.clone();
        let message = message.clone();
        Callback::from(move |_| {
            open_id.set(None);
            creating.set(false);
            message.set(None);
        })
    };

    let on_new = {
        let creating = creating.clone();
        let open_id = open_id.clone();
        let title = title.clone();
        let id_field = id_field.clone();
        let id_touched = id_touched.clone();
        let summary = summary.clone();
        let date = date.clone();
        let location = location.clone();
        let status = status.clone();
        let message = message.clone();
        Callback::from(move |_| {
            open_id.set(None);
            title.set(String::new());
            id_field.set(String::new());
            id_touched.set(false);
            summary.set(String::new());
            date.set(String::new());
            location.set(String::new());
            status.set("draft".into());
            message.set(None);
            creating.set(true);
        })
    };

    let on_title = {
        let title = title.clone();
        let id_field = id_field.clone();
        let id_touched = id_touched.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            if !*id_touched {
                id_field.set(slug(&value));
            }
            title.set(value);
        })
    };
    let on_id = {
        let id_field = id_field.clone();
        let id_touched = id_touched.clone();
        Callback::from(move |event: InputEvent| {
            id_touched.set(true);
            id_field.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };
    let bind = |state: UseStateHandle<String>| {
        Callback::from(move |event: InputEvent| {
            state.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };
    let on_status = {
        let status = status.clone();
        Callback::from(move |event: Event| {
            status.set(event.target_unchecked_into::<HtmlSelectElement>().value());
        })
    };

    let on_create = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let title = title.clone();
        let id_field = id_field.clone();
        let summary = summary.clone();
        let date = date.clone();
        let location = location.clone();
        let status = status.clone();
        let message = message.clone();
        let open_id = open_id.clone();
        let creating = creating.clone();
        let tick = tick.clone();
        Callback::from(move |_| {
            let id = slug(id_field.trim());
            let title = title.trim().to_string();
            if id.is_empty() || title.is_empty() {
                message.set(Some("Title is required.".into()));
                return;
            }
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let body = serde_json::json!({
                "id": id,
                "title": title,
                "summary": summary.trim(),
                "date": date.trim(),
                "location": location.trim(),
                "status": status.trim(),
                "media": [],
            });
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let message = message.clone();
            let open_id = open_id.clone();
            let creating = creating.clone();
            let tick = tick.clone();
            spawn_local(async move {
                match api("POST", &format!("/api/admin/{kind}"), &jwt, Some(body.to_string())).await {
                    Ok(()) => {
                        message.set(Some("Collection created.".into()));
                        creating.set(false);
                        open_id.set(Some(id));
                        tick.set(*tick + 1);
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_save = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let open_id = open_id.clone();
        let title = title.clone();
        let summary = summary.clone();
        let date = date.clone();
        let location = location.clone();
        let status = status.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |_| {
            let Some(id) = (*open_id).clone() else {
                return;
            };
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let body = serde_json::json!({
                "title": title.trim(),
                "summary": summary.trim(),
                "date": date.trim(),
                "location": location.trim(),
                "status": status.trim(),
            });
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let message = message.clone();
            let tick = tick.clone();
            spawn_local(async move {
                match api("PATCH", &format!("/api/admin/{kind}/{id}"), &jwt, Some(body.to_string())).await {
                    Ok(()) => {
                        message.set(Some("Saved.".into()));
                        tick.set(*tick + 1);
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_delete = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let open_id = open_id.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |_| {
            let Some(id) = (*open_id).clone() else {
                return;
            };
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let message = message.clone();
            let open_id = open_id.clone();
            let tick = tick.clone();
            spawn_local(async move {
                match api("DELETE", &format!("/api/admin/{kind}/{id}"), &jwt, None).await {
                    Ok(()) => {
                        open_id.set(None);
                        message.set(Some("Collection removed.".into()));
                        tick.set(*tick + 1);
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_files = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let open_id = open_id.clone();
        let featured = featured.clone();
        let set_cover = set_cover.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<HtmlInputElement>();
            let Some(list) = input.files() else {
                return;
            };
            let mut files = Vec::new();
            for index in 0..list.length() {
                if let Some(file) = list.get(index) {
                    files.push(file);
                }
            }
            if files.is_empty() {
                return;
            }
            let Some(id) = (*open_id).clone() else {
                return;
            };
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let featured_state = featured.clone();
            let cover_state = set_cover.clone();
            let featured = *featured;
            let set_cover = *set_cover;
            let message = message.clone();
            let tick = tick.clone();
            spawn_local(async move {
                let form = match FormData::new() {
                    Ok(form) => form,
                    Err(_) => {
                        message.set(Some("Could not build upload form.".into()));
                        return;
                    }
                };
                for file in &files {
                    if form.append_with_blob("file", file).is_err() {
                        message.set(Some("Could not attach file.".into()));
                        return;
                    }
                }
                if set_cover {
                    let _ = form.append_with_str("setAsCover", "true");
                }
                if featured {
                    let _ = form.append_with_str("featured", "true");
                }
                let url = format!("/api/admin/{kind}/{id}/media");
                let sent = Request::post(&url)
                    .header("Authorization", &auth_header(&jwt))
                    .body(JsValue::from(form))
                    .map_err(|err| err.to_string());
                match sent {
                    Ok(req) => match req.send().await {
                        Ok(resp) if resp.ok() => {
                            featured_state.set(false);
                            cover_state.set(false);
                            message.set(Some(format!("Uploaded {} file(s).", files.len())));
                            tick.set(*tick + 1);
                        }
                        Ok(resp) => {
                            let text = resp.text().await.unwrap_or_else(|_| "Upload failed.".into());
                            message.set(Some(text));
                        }
                        Err(err) => message.set(Some(err.to_string())),
                    },
                    Err(err) => message.set(Some(err)),
                }
            });
            input.set_value("");
        })
    };

    let move_media = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let open_id = open_id.clone();
        let items = items.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |(asset_id, delta): (String, i32)| {
            let Some(id) = (*open_id).clone() else {
                return;
            };
            let Some(item) = items.iter().find(|item| item.id == id) else {
                return;
            };
            let mut ids: Vec<String> = item.media.iter().map(|media| media.id.clone()).collect();
            let Some(index) = ids.iter().position(|media_id| media_id == &asset_id) else {
                return;
            };
            let next = index as i32 + delta;
            if next < 0 || next as usize >= ids.len() {
                return;
            }
            ids.swap(index, next as usize);
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let body = serde_json::json!({ "mediaIds": ids }).to_string();
            let message = message.clone();
            let tick = tick.clone();
            spawn_local(async move {
                match api(
                    "PUT",
                    &format!("/api/admin/{kind}/{id}/media/order"),
                    &jwt,
                    Some(body),
                )
                .await
                {
                    Ok(()) => tick.set(*tick + 1),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_cover = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let open_id = open_id.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |url: String| {
            let Some(id) = (*open_id).clone() else {
                return;
            };
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let body = serde_json::json!({ "coverUrl": url }).to_string();
            let message = message.clone();
            let tick = tick.clone();
            spawn_local(async move {
                match api("PATCH", &format!("/api/admin/{kind}/{id}"), &jwt, Some(body)).await {
                    Ok(()) => {
                        message.set(Some("Cover updated.".into()));
                        tick.set(*tick + 1);
                    }
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_remove_media = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.to_string();
        let open_id = open_id.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |asset_id: String| {
            let Some(id) = (*open_id).clone() else {
                return;
            };
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let message = message.clone();
            let tick = tick.clone();
            spawn_local(async move {
                match api(
                    "DELETE",
                    &format!("/api/admin/{kind}/{id}/media/{asset_id}"),
                    &jwt,
                    None,
                )
                .await
                {
                    Ok(()) => tick.set(*tick + 1),
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_date = {
        let date = date.clone();
        Callback::from(move |value: String| date.set(value))
    };

    let fields = html! {
        <div class="collection-editor-fields">
            <input type="text" class="field" placeholder="Title" value={(*title).clone()} oninput={on_title} />
            if *creating {
                <input type="text" class="field" placeholder="ID" value={(*id_field).clone()} oninput={on_id} />
            }
            <input type="text" class="field" placeholder="Summary" value={(*summary).clone()} oninput={bind(summary.clone())} />
            <DatePicker value={(*date).clone()} on_change={on_date} placeholder="Date" />
            <input type="text" class="field" placeholder="Location" value={(*location).clone()} oninput={bind(location.clone())} />
            <select class="field" value={(*status).clone()} onchange={on_status} aria-label="Status">
                <option value="draft" selected={*status == "draft"}>{ "Draft" }</option>
                <option value="published" selected={*status == "published"}>{ "Published" }</option>
                <option value="scheduled" selected={*status == "scheduled"}>{ "Scheduled" }</option>
            </select>
        </div>
    };

    html! {
        <div class="space-y-4">
            if let Some(msg) = &*message {
                <p class="text-sm text-accent">{ msg }</p>
            }
            if let Some(item) = open {
                <button type="button" class="btn btn-quiet" onclick={on_back.clone()}>{ format!("All {plural}") }</button>
                <h3 class="font-semibold">{ item.title.clone() }</h3>
                if item.locked {
                    <p class="muted">{ "This collection is in the server config. Edit it there, not here." }</p>
                } else {
                    <div class="collection-editor">
                        { fields.clone() }
                        <div class="collection-editor-actions">
                            <button type="button" class="btn btn-primary" onclick={on_save}>{ "Save" }</button>
                            <button type="button" class="btn btn-quiet" onclick={on_delete}>{ "Delete" }</button>
                        </div>
                        <div class="collection-editor-options">
                            <label class="collection-check">
                                <input type="checkbox" checked={*set_cover} onchange={bind_check(set_cover.clone())} />
                                <span>{ "Set first upload as cover" }</span>
                            </label>
                            <label class="collection-check">
                                <input type="checkbox" checked={*featured} onchange={bind_check(featured.clone())} />
                                <span>{ "Featured" }</span>
                            </label>
                        </div>
                        <div class="collection-editor-upload">
                            <label class="btn btn-quiet collection-file-btn">
                                { "Choose files" }
                                <input
                                    type="file"
                                    multiple=true
                                    accept="image/jpeg,image/png,image/webp,image/gif"
                                    class="collection-file-input"
                                    onchange={on_files}
                                />
                            </label>
                            <p class="muted label-xs">{ "JPEG, PNG, WebP, or GIF. Multiple files OK." }</p>
                        </div>
                    </div>
                }
                <div class="collection-editor-media grid grid-cols-2 gap-3 sm:grid-cols-3">
                    { for item.media.iter().enumerate().map(|(index, media)| {
                        let earlier = move_media.clone();
                        let later = move_media.clone();
                        let cover = on_cover.clone();
                        let remove = on_remove_media.clone();
                        let earlier_id = media.id.clone();
                        let later_id = media.id.clone();
                        let remove_id = media.id.clone();
                        let thumb = media.thumbnail_url.clone();
                        let cover_url = media.thumbnail_url.clone();
                        let is_cover = item.cover_url.as_deref() == Some(thumb.as_str());
                        let last = index + 1 == item.media.len();
                        html! {
                            <figure key={media.id.clone()} class="card card-flat overflow-hidden">
                                <img src={thumb} alt={media.title.clone().unwrap_or_default()} class="aspect-[4/3] w-full object-cover bg-surface" />
                                if !item.locked {
                                    <div class="flex flex-wrap gap-1 p-2">
                                        <button type="button" class="btn btn-quiet" disabled={index == 0} onclick={Callback::from(move |_| earlier.emit((earlier_id.clone(), -1)))}>{ "Earlier" }</button>
                                        <button type="button" class="btn btn-quiet" disabled={last} onclick={Callback::from(move |_| later.emit((later_id.clone(), 1)))}>{ "Later" }</button>
                                        <button type="button" class="btn btn-quiet" disabled={is_cover} onclick={Callback::from(move |_| cover.emit(cover_url.clone()))}>{ "Cover" }</button>
                                        <button type="button" class="btn btn-quiet" onclick={Callback::from(move |_| remove.emit(remove_id.clone()))}>{ "Remove" }</button>
                                    </div>
                                }
                            </figure>
                        }
                    }) }
                </div>
            } else if *creating {
                <button type="button" class="btn btn-quiet" onclick={on_back}>{ format!("All {plural}") }</button>
                <h3 class="font-semibold">{ format!("New {noun}") }</h3>
                <div class="collection-editor">
                    { fields }
                    <div class="collection-editor-actions">
                        <button type="button" class="btn btn-primary" onclick={on_create}>{ "Create" }</button>
                    </div>
                </div>
            } else {
                <div class="flex items-center justify-between gap-2">
                    <p class="muted">{ format!("{} {}", items.len(), if items.len() == 1 { noun } else { plural }) }</p>
                    <button type="button" class="btn btn-primary" onclick={on_new}>{ format!("New {noun}") }</button>
                </div>
                if items.is_empty() {
                    <p class="muted">{ format!("No {plural} yet.") }</p>
                }
                <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
                    { for items.iter().map(|item| {
                        let id = item.id.clone();
                        let on_open = on_open.clone();
                        let cover = cover_of(item);
                        html! {
                            <button type="button" key={item.id.clone()} class="card card-flat overflow-hidden text-left" onclick={Callback::from(move |_| on_open.emit(id.clone()))}>
                                if let Some(cover) = cover {
                                    <img src={cover} alt="" class="aspect-[16/9] w-full object-cover bg-surface" />
                                } else {
                                    <div class="aspect-[16/9] bg-surface"></div>
                                }
                                <div class="space-y-1 p-3">
                                    <p class="font-semibold">{ item.title.clone() }</p>
                                    <p class="text-xs text-muted">{ format!("{} · {} {}", status_of(item), item.media.len(), if item.media.len() == 1 { "photo" } else { "photos" }) }</p>
                                </div>
                            </button>
                        }
                    }) }
                </div>
            }
        </div>
    }
}

fn bind_check(state: UseStateHandle<bool>) -> Callback<Event> {
    Callback::from(move |event: Event| {
        state.set(event.target_unchecked_into::<HtmlInputElement>().checked());
    })
}
