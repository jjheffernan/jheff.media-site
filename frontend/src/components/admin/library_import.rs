use crate::{components::ui::Text, components::ui::TextTone, context::AuthContext, services::auth_header};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct LibraryCandidate {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct SourceList {
    sources: Vec<String>,
}

#[derive(Deserialize)]
struct ImportResult {
    copied: usize,
}

#[function_component(LibraryImport)]
pub fn library_import() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let sources = use_state(Vec::<String>::new);
    let source = use_state(String::new);
    let items = use_state(Vec::<LibraryCandidate>::new);
    let selected = use_state(Vec::<String>::new);
    let kind = use_state(|| "gallery".to_string());
    let collection_id = use_state(String::new);
    let message = use_state(|| None::<String>);
    let ready = use_state(|| false);

    {
        let auth_ctx = auth_ctx.clone();
        let sources = sources.clone();
        let source = source.clone();
        let message = message.clone();
        let ready = ready.clone();
        use_effect_with(auth_ctx.auth.as_ref().map(|auth| auth.jwt.clone()), move |jwt| {
            if let Some(jwt) = jwt.clone() {
            let sources = sources.clone();
            let source = source.clone();
            let message = message.clone();
            let ready = ready.clone();
            spawn_local(async move {
                let result = Request::get("/api/admin/library")
                    .header("Authorization", &auth_header(&jwt))
                    .send()
                    .await;
                match result {
                    Ok(resp) if resp.ok() => {
                        if let Ok(body) = resp.json::<crate::model::ServerResponse<SourceList>>().await {
                            if let Some(first) = body.data.sources.first() {
                                source.set(first.clone());
                            }
                            sources.set(body.data.sources);
                            ready.set(true);
                        }
                    }
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_default();
                        message.set(Some(text));
                    }
                    Err(err) => message.set(Some(err.to_string())),
                }
            });
            }
            || ()
        });
    }

    let on_source = {
        let source = source.clone();
        let items = items.clone();
        let selected = selected.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<web_sys::HtmlSelectElement>();
            source.set(input.value());
            items.set(Vec::new());
            selected.set(Vec::new());
        })
    };
    let on_kind = {
        let kind = kind.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<web_sys::HtmlSelectElement>();
            kind.set(input.value());
        })
    };
    let on_collection = {
        let collection_id = collection_id.clone();
        Callback::from(move |event: InputEvent| {
            let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
            collection_id.set(input.value());
        })
    };

    let on_load = {
        let auth_ctx = auth_ctx.clone();
        let source = source.clone();
        let items = items.clone();
        let selected = selected.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let source = (*source).clone();
            let items = items.clone();
            let selected = selected.clone();
            let message = message.clone();
            spawn_local(async move {
                selected.set(Vec::new());
                let result = Request::get(&format!("/api/admin/library/{source}"))
                    .header("Authorization", &auth_header(&jwt))
                    .send()
                    .await;
                match result {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<crate::model::ServerResponse<Vec<LibraryCandidate>>>().await {
                            Ok(body) => {
                                message.set(Some(format!("{} files ready to choose.", body.data.len())));
                                items.set(body.data);
                            }
                            Err(_) => message.set(Some("Could not read the library list.".into())),
                        }
                    }
                    Ok(resp) => message.set(Some(error_message(resp).await)),
                    Err(err) => message.set(Some(err.to_string())),
                }
            });
        })
    };

    let on_import = {
        let auth_ctx = auth_ctx.clone();
        let source = source.clone();
        let selected = selected.clone();
        let kind = kind.clone();
        let collection_id = collection_id.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let Some(auth) = &auth_ctx.auth else {
                return;
            };
            let jwt = auth.jwt.clone();
            let source = (*source).clone();
            let ids = (*selected).clone();
            let kind = (*kind).clone();
            let collection_id = (*collection_id).clone();
            let message = message.clone();
            spawn_local(async move {
                let body = serde_json::json!({
                    "kind": kind,
                    "collectionId": collection_id.trim(),
                    "assetIds": ids,
                });
                let result = Request::post(&format!("/api/admin/library/{source}"))
                    .header("Authorization", &auth_header(&jwt))
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .expect("json body")
                    .send()
                    .await;
                match result {
                    Ok(resp) if resp.ok() => {
                        match resp.json::<crate::model::ServerResponse<ImportResult>>().await {
                            Ok(body) => message.set(Some(format!(
                                "Copied {} file(s) into the draft. Change its status when it should be public.",
                                body.data.copied
                            ))),
                            Err(_) => message.set(Some("Import finished, but the reply could not be read.".into())),
                        }
                    }
                    Ok(resp) => message.set(Some(error_message(resp).await)),
                    Err(err) => message.set(Some(err.to_string())),
                }
            });
        })
    };

    let toggle = {
        let selected = selected.clone();
        Callback::from(move |id: String| {
            let mut next = (*selected).clone();
            if let Some(index) = next.iter().position(|item| item == &id) {
                next.remove(index);
            } else if next.len() < 6 {
                next.push(id);
            }
            selected.set(next);
        })
    };

    html! {
        <div class="space-y-3">
            <Text tone={TextTone::Muted}>
                { "Library import. Admin only. Pick up to 6 files from Immich or Nextcloud and copy them into a draft gallery or shoot. Credentials stay on the server." }
            </Text>
            if !*ready {
                <p class="muted">{ "Checking which libraries are configured." }</p>
            } else if sources.is_empty() {
                <p class="muted">{ "No library is configured. Set the Immich or Nextcloud env vars on the server." }</p>
            } else {
                <select class="field" onchange={on_source}>
                    { for sources.iter().map(|name| html! {
                        <option value={name.clone()} selected={*source == *name}>{ name.clone() }</option>
                    }) }
                </select>
                <button type="button" class="btn btn-quiet" onclick={on_load}>{ "Show recent files" }</button>
                <ul class="space-y-1">
                    { for items.iter().map(|item| {
                        let checked = selected.iter().any(|id| id == &item.id);
                        let id = item.id.clone();
                        let on_toggle = toggle.clone();
                        html! {
                            <li key={item.id.clone()}>
                                <label class="menu-action">
                                    <input
                                        type="checkbox"
                                        checked={checked}
                                        onchange={Callback::from(move |_| on_toggle.emit(id.clone()))}
                                    />
                                    <span>{ item.name.clone() }</span>
                                </label>
                            </li>
                        }
                    }) }
                </ul>
                <select class="field" onchange={on_kind}>
                    <option value="gallery" selected={*kind == "gallery"}>{ "Gallery" }</option>
                    <option value="shoot" selected={*kind == "shoot"}>{ "Shoot" }</option>
                </select>
                <input
                    type="text"
                    class="field"
                    placeholder="Draft collection ID"
                    value={(*collection_id).clone()}
                    oninput={on_collection}
                />
                <button
                    type="button"
                    class="btn btn-primary"
                    disabled={selected.is_empty() || collection_id.trim().is_empty()}
                    onclick={on_import}
                >
                    { "Copy selected into draft" }
                </button>
            }
            if let Some(msg) = &*message {
                <p class="muted">{ msg }</p>
            }
        </div>
    }
}

async fn error_message(resp: gloo_net::http::Response) -> String {
    if let Ok(body) = resp.json::<crate::model::ServerResponse<String>>().await {
        return match body.message.as_str() {
            "NOT_DRAFT" => "That collection is not a draft. Set its status to draft before copying.".into(),
            "NOT_CONFIGURED" => "That library is not configured on the server.".into(),
            "TOO_MANY" => "Choose at most 6 files.".into(),
            "COLLECTION_NOT_FOUND" => "No Mongo collection with that id.".into(),
            "UPSTREAM" => "The library refused the request.".into(),
            "FILE_TOO_LARGE" => "A file is over the upload size limit.".into(),
            "UNSUPPORTED_MEDIA_TYPE" => "A file is not a JPEG, PNG, WebP, or GIF.".into(),
            other => other.to_string(),
        };
    }
    "Library request failed.".into()
}
