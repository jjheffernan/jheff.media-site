use crate::{
    components::{
        feed::PhotoGrid,
        ui::{Heading, HeadingLevel, Spinner, Text, TextTone},
    },
    context::AuthContext,
    model::{ContentCollection, FeedItem, MediaItem, ServerResponse},
    services::auth_header,
};
use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{FormData, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: String,
    pub api_path: String,
    pub back_route: crate::routes::AppRoutes,
    pub back_label: String,
    pub collection_kind: String,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Ready(ContentCollection),
    Error(String),
}

fn admin_kind(api_path: &str) -> &str {
    api_path.trim_start_matches("/api/")
}

#[function_component(ContentDetail)]
pub fn content_detail(props: &Props) -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let is_admin = auth_ctx
        .auth
        .as_ref()
        .is_some_and(|auth| auth.user.role == "admin");
    let state = use_state(|| LoadState::Loading);
    let tick = use_state(|| 0u32);
    let message = use_state(|| None::<String>);
    let set_cover = use_state(|| false);
    let api_url = format!("{}/{}", props.api_path, props.id);
    let kind = admin_kind(&props.api_path).to_string();

    {
        let state = state.clone();
        let api_url = api_url.clone();
        use_effect_with((api_url.clone(), props.id.clone(), *tick), move |_| {
            spawn_local(async move {
                let result = Request::get(&api_url).send().await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(collection) = resp.json::<ContentCollection>().await {
                            state.set(LoadState::Ready(collection));
                            return;
                        }
                        state.set(LoadState::Error("Could not parse content.".into()));
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            state.set(LoadState::Error(body.message));
                        } else {
                            state.set(LoadState::Error("Content not found.".into()));
                        }
                    }
                    Err(err) => state.set(LoadState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let on_files = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.clone();
        let id = props.id.clone();
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
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let id = id.clone();
            let cover_state = set_cover.clone();
            let set_as_cover = *set_cover;
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
                if set_as_cover {
                    let _ = form.append_with_str("setAsCover", "true");
                }
                let url = format!("/api/admin/{kind}/{id}/media");
                let sent = Request::post(&url)
                    .header("Authorization", &auth_header(&jwt))
                    .body(JsValue::from(form))
                    .map_err(|err| err.to_string());
                match sent {
                    Ok(req) => match req.send().await {
                        Ok(resp) if resp.ok() => {
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

    let on_remove = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.clone();
        let id = props.id.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |asset_id: String| {
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let id = id.clone();
            let message = message.clone();
            let tick = tick.clone();
            spawn_local(async move {
                let resp = Request::delete(&format!("/api/admin/{kind}/{id}/media/{asset_id}"))
                    .header("Authorization", &auth_header(&jwt))
                    .send()
                    .await;
                match resp {
                    Ok(resp) if resp.ok() => {
                        message.set(Some("Removed.".into()));
                        tick.set(*tick + 1);
                    }
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_else(|_| "Remove failed.".into());
                        message.set(Some(text));
                    }
                    Err(err) => message.set(Some(err.to_string())),
                }
            });
        })
    };

    let on_cover = {
        let auth_ctx = auth_ctx.clone();
        let kind = kind.clone();
        let id = props.id.clone();
        let message = message.clone();
        let tick = tick.clone();
        Callback::from(move |url: String| {
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let kind = kind.clone();
            let id = id.clone();
            let message = message.clone();
            let tick = tick.clone();
            let body = serde_json::json!({ "coverUrl": url }).to_string();
            spawn_local(async move {
                let resp = Request::patch(&format!("/api/admin/{kind}/{id}"))
                    .header("Authorization", &auth_header(&jwt))
                    .header("Content-Type", "application/json")
                    .body(body)
                    .map_err(|err| err.to_string());
                match resp {
                    Ok(req) => match req.send().await {
                        Ok(resp) if resp.ok() => {
                            message.set(Some("Cover updated.".into()));
                            tick.set(*tick + 1);
                        }
                        Ok(resp) => {
                            let text = resp.text().await.unwrap_or_else(|_| "Cover update failed.".into());
                            message.set(Some(text));
                        }
                        Err(err) => message.set(Some(err.to_string())),
                    },
                    Err(err) => message.set(Some(err)),
                }
            });
        })
    };

    let on_set_cover_toggle = {
        let set_cover = set_cover.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<HtmlInputElement>();
            set_cover.set(input.checked());
        })
    };

    let admin_bar = |media: &[MediaItem], cover_url: &Option<String>| -> Html {
        if !is_admin {
            return html! {};
        }
        html! {
            <div class="space-y-3">
                <div class="space-y-2">
                    <p class="text-sm text-muted">{ "Manage media" }</p>
                    <label class="muted text-sm">
                        <input
                            type="checkbox"
                            checked={*set_cover}
                            onchange={on_set_cover_toggle.clone()}
                        />
                        { " Set first upload as cover" }
                    </label>
                    <input
                        type="file"
                        multiple=true
                        accept="image/jpeg,image/png,image/webp,image/gif"
                        class="block w-full text-sm"
                        onchange={on_files.clone()}
                    />
                </div>
                if let Some(msg) = &*message {
                    <p class="text-sm text-accent">{ msg }</p>
                }
                if !media.is_empty() {
                    <ul class="space-y-2">
                        { for media.iter().map(|item| {
                            let remove = on_remove.clone();
                            let cover = on_cover.clone();
                            let remove_id = item.id.clone();
                            let cover_thumb = item.thumbnail_url.clone();
                            let is_cover = cover_url.as_deref() == Some(item.thumbnail_url.as_str());
                            let label = item
                                .title
                                .clone()
                                .filter(|t| !t.is_empty())
                                .unwrap_or_else(|| item.id.clone());
                            html! {
                                <li key={item.id.clone()} class="flex flex-wrap items-center gap-2">
                                    <img
                                        src={item.thumbnail_url.clone()}
                                        alt=""
                                        class="h-10 w-10 object-cover bg-surface"
                                    />
                                    <span class="text-sm text-muted flex-1">{ label }</span>
                                    <button
                                        type="button"
                                        class="btn btn-quiet"
                                        disabled={is_cover}
                                        onclick={Callback::from(move |_| cover.emit(cover_thumb.clone()))}
                                    >
                                        { if is_cover { "Cover" } else { "Set cover" } }
                                    </button>
                                    <button
                                        type="button"
                                        class="btn btn-quiet"
                                        onclick={Callback::from(move |_| remove.emit(remove_id.clone()))}
                                    >
                                        { "Remove" }
                                    </button>
                                </li>
                            }
                        }) }
                    </ul>
                }
            </div>
        }
    };

    let body = match &*state {
        LoadState::Loading => html! { <Spinner label="Loading…" /> },
        LoadState::Error(msg) => html! {
            <p class="text-sm text-muted">{ msg.clone() }</p>
        },
        LoadState::Ready(collection) => {
            let feed_items: Vec<FeedItem> = collection
                .media
                .iter()
                .map(|m| FeedItem {
                    id: m.id.clone(),
                    title: m.title.clone(),
                    thumbnail_url: m.thumbnail_url.clone(),
                    media_url: m.media_url.clone(),
                    media_type: m.media_type.clone(),
                    width: m.width,
                    height: m.height,
                    featured: m.featured,
                })
                .collect();

            let meta = {
                let mut parts = vec![];
                if let Some(date) = &collection.date {
                    parts.push(date.clone());
                }
                if let Some(location) = &collection.location {
                    parts.push(location.clone());
                }
                if let Some(status) = &collection.status {
                    parts.push(status.clone());
                }
                parts.join(" · ")
            };

            html! {
                <div class="space-y-6">
                    if let Some(summary) = &collection.summary {
                        <p class="text-sm text-muted">{ summary.clone() }</p>
                    }
                    if !meta.is_empty() {
                        <p class="text-sm text-muted">{ meta }</p>
                    }
                    if feed_items.is_empty() {
                        <div class="space-y-4">
                            <Text tone={TextTone::Muted}>
                                { "No media in this collection yet." }
                            </Text>
                            { admin_bar(&collection.media, &collection.cover_url) }
                        </div>
                    } else {
                        <p class="muted">{ "Prints are requested from your account." }</p>
                        <PhotoGrid
                            items={feed_items}
                            collection_id={collection.id.clone()}
                            collection_kind={props.collection_kind.clone()}
                            collection_title={collection.title.clone()}
                        />
                        { admin_bar(&collection.media, &collection.cover_url) }
                    }
                </div>
            }
        }
    };

    let title = match &*state {
        LoadState::Ready(c) => c.title.clone(),
        _ => props.id.clone(),
    };

    html! {
        <div class="space-y-6">
            <div class="space-y-2">
                <Link<crate::routes::AppRoutes>
                    to={props.back_route.clone()}
                    classes="footer-link"
                >
                    { format!("← Back to {}", props.back_label) }
                </Link<crate::routes::AppRoutes>>
                <Heading level={HeadingLevel::H1}>
                    { html! { { title.clone() } } }
                </Heading>
            </div>
            { body }
        </div>
    }
}
