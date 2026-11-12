use crate::{context::AuthContext, services::auth_header};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use web_sys::HtmlSelectElement;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SetupConfig {
    feed_provider: String,
    immich_url: String,
    immich_api_key_set: bool,
    immich_api_key_hint: String,
    immich_album_id: String,
    immich_shared_key_set: bool,
    nextcloud_url: String,
    nextcloud_user: String,
    nextcloud_app_password_set: bool,
    nextcloud_app_password_hint: String,
    nextcloud_folder: String,
    feed_static_json: String,
    contact_email: String,
    public_url: String,
    from_db: bool,
    needs_setup: bool,
    restart_required: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupUpdate {
    feed_provider: String,
    immich_url: String,
    immich_api_key: String,
    immich_album_id: String,
    immich_shared_key: String,
    nextcloud_url: String,
    nextcloud_user: String,
    nextcloud_app_password: String,
    nextcloud_folder: String,
    feed_static_json: String,
    contact_email: String,
    public_url: String,
}

#[derive(Clone, PartialEq)]
struct FormState {
    feed_provider: String,
    immich_url: String,
    immich_api_key: String,
    immich_album_id: String,
    immich_shared_key: String,
    nextcloud_url: String,
    nextcloud_user: String,
    nextcloud_app_password: String,
    nextcloud_folder: String,
    feed_static_json: String,
    contact_email: String,
    public_url: String,
    immich_hint: String,
    nextcloud_hint: String,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            feed_provider: "static".into(),
            immich_url: String::new(),
            immich_api_key: String::new(),
            immich_album_id: String::new(),
            immich_shared_key: String::new(),
            nextcloud_url: String::new(),
            nextcloud_user: String::new(),
            nextcloud_app_password: String::new(),
            nextcloud_folder: String::new(),
            feed_static_json: String::new(),
            contact_email: String::new(),
            public_url: String::new(),
            immich_hint: String::new(),
            nextcloud_hint: String::new(),
        }
    }
}

fn from_config(cfg: &SetupConfig) -> FormState {
    FormState {
        feed_provider: cfg.feed_provider.clone(),
        immich_url: cfg.immich_url.clone(),
        immich_api_key: String::new(),
        immich_album_id: cfg.immich_album_id.clone(),
        immich_shared_key: String::new(),
        nextcloud_url: cfg.nextcloud_url.clone(),
        nextcloud_user: cfg.nextcloud_user.clone(),
        nextcloud_app_password: String::new(),
        nextcloud_folder: cfg.nextcloud_folder.clone(),
        feed_static_json: cfg.feed_static_json.clone(),
        contact_email: cfg.contact_email.clone(),
        public_url: cfg.public_url.clone(),
        immich_hint: if cfg.immich_api_key_set {
            cfg.immich_api_key_hint.clone()
        } else {
            String::new()
        },
        nextcloud_hint: if cfg.nextcloud_app_password_set {
            cfg.nextcloud_app_password_hint.clone()
        } else {
            String::new()
        },
    }
}

#[function_component(SetupAdmin)]
pub fn setup_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let form = use_state(FormState::default);
    let note = use_state(|| None::<String>);
    let restart = use_state(|| false);
    let loading = use_state(|| true);
    let jwt = auth_ctx
        .auth
        .as_ref()
        .filter(|auth| auth.user.role == "admin")
        .map(|auth| auth.jwt.clone())
        .unwrap_or_default();

    {
        let form = form.clone();
        let note = note.clone();
        let loading = loading.clone();
        use_effect_with(jwt.clone(), move |jwt| {
            if !jwt.is_empty() {
                let jwt = jwt.clone();
                spawn_local(async move {
                    let auth_val = auth_header(&jwt);
                    match Request::get("/api/admin/setup")
                        .header("Authorization", auth_val.as_str())
                        .send()
                        .await
                    {
                        Ok(resp) if resp.ok() => match resp.json::<SetupConfig>().await {
                            Ok(cfg) => {
                                form.set(from_config(&cfg));
                                if cfg.needs_setup {
                                    note.set(Some(
                                        "First-run: save setup here so feed settings live in Mongo (env is the bootstrap default)."
                                            .into(),
                                    ));
                                }
                            }
                            Err(_) => note.set(Some("Could not read setup.".into())),
                        },
                        Ok(_) => note.set(Some("Could not load setup.".into())),
                        Err(err) => note.set(Some(err.to_string())),
                    }
                    loading.set(false);
                });
            } else {
                loading.set(false);
            }
            || ()
        });
    }

    let on_provider = {
        let form = form.clone();
        Callback::from(move |e: Event| {
            let value = e
                .target_dyn_into::<HtmlSelectElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            let mut next = (*form).clone();
            next.feed_provider = value;
            form.set(next);
        })
    };

    let set_input = |field: &'static str| {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let value = e
                .target_dyn_into::<HtmlInputElement>()
                .map(|el| el.value())
                .or_else(|| {
                    e.target_dyn_into::<HtmlTextAreaElement>()
                        .map(|el| el.value())
                })
                .unwrap_or_default();
            let mut next = (*form).clone();
            match field {
                "immich_url" => next.immich_url = value,
                "immich_api_key" => next.immich_api_key = value,
                "immich_album_id" => next.immich_album_id = value,
                "immich_shared_key" => next.immich_shared_key = value,
                "nextcloud_url" => next.nextcloud_url = value,
                "nextcloud_user" => next.nextcloud_user = value,
                "nextcloud_app_password" => next.nextcloud_app_password = value,
                "nextcloud_folder" => next.nextcloud_folder = value,
                "feed_static_json" => next.feed_static_json = value,
                "contact_email" => next.contact_email = value,
                "public_url" => next.public_url = value,
                _ => {}
            }
            form.set(next);
        })
    };

    let on_save = {
        let form = form.clone();
        let note = note.clone();
        let restart = restart.clone();
        let jwt = jwt.clone();
        Callback::from(move |_| {
            if jwt.is_empty() {
                return;
            }
            let jwt = jwt.clone();
            let note = note.clone();
            let restart = restart.clone();
            let form = form.clone();
            let f = (*form).clone();
            spawn_local(async move {
                let body = SetupUpdate {
                    feed_provider: f.feed_provider,
                    immich_url: f.immich_url,
                    immich_api_key: f.immich_api_key,
                    immich_album_id: f.immich_album_id,
                    immich_shared_key: f.immich_shared_key,
                    nextcloud_url: f.nextcloud_url,
                    nextcloud_user: f.nextcloud_user,
                    nextcloud_app_password: f.nextcloud_app_password,
                    nextcloud_folder: f.nextcloud_folder,
                    feed_static_json: f.feed_static_json,
                    contact_email: f.contact_email,
                    public_url: f.public_url,
                };
                let Ok(payload) = serde_json::to_string(&body) else {
                    note.set(Some("Could not encode setup.".into()));
                    return;
                };
                let auth_val = auth_header(&jwt);
                let sent = Request::put("/api/admin/setup")
                    .header("Authorization", auth_val.as_str())
                    .header("Content-Type", "application/json")
                    .body(payload);
                match sent {
                    Ok(req) => match req.send().await {
                        Ok(resp) if resp.ok() => match resp.json::<SetupConfig>().await {
                            Ok(cfg) => {
                                form.set(from_config(&cfg));
                                restart.set(cfg.restart_required);
                                if cfg.restart_required {
                                    note.set(Some(
                                        "Saved. Provider changed — restart may be needed to repopulate images (`jheffmedia --restart`)."
                                            .into(),
                                    ));
                                } else {
                                    note.set(Some(
                                        "Saved. Feed reads Mongo on each request; restart only if images look stale."
                                            .into(),
                                    ));
                                }
                            }
                            Err(_) => note.set(Some("Saved, but reply could not be read.".into())),
                        },
                        Ok(resp) => {
                            let message = resp
                                .json::<serde_json::Value>()
                                .await
                                .ok()
                                .and_then(|body| {
                                    body.get("message")
                                        .and_then(|m| m.as_str())
                                        .map(str::to_string)
                                })
                                .unwrap_or_else(|| "Could not save setup.".into());
                            note.set(Some(message));
                        }
                        Err(err) => note.set(Some(err.to_string())),
                    },
                    Err(err) => note.set(Some(err.to_string())),
                }
            });
        })
    };

    let provider = form.feed_provider.clone();

    html! {
        <section class="card space-y-4">
            <div>
                <h2 class="title-sm">{ "Site setup" }</h2>
                <p class="muted">{ "Feed provider and contact defaults. Saved to Mongo — no redeploy for provider switches." }</p>
            </div>
            if *loading {
                <p class="muted">{ "Loading…" }</p>
            } else {
                <label class="stack-gap-xs">
                    <span class="label-xs">{ "Photo feed provider" }</span>
                    <select class="field" value={provider.clone()} onchange={on_provider}>
                        <option value="none" selected={provider == "none"}>{ "None" }</option>
                        <option value="static" selected={provider == "static"}>{ "Static / demo" }</option>
                        <option value="immich" selected={provider == "immich"}>{ "Immich" }</option>
                        <option value="nextcloud" selected={provider == "nextcloud"}>{ "Nextcloud" }</option>
                    </select>
                </label>

                if provider == "immich" {
                    <div class="space-y-3">
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "Immich URL" }</span>
                            <input class="field" type="url" placeholder="https://photos.example.com" value={form.immich_url.clone()} oninput={set_input("immich_url")} />
                        </label>
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "API key" }</span>
                            <input class="field" type="password" autocomplete="off" placeholder={if form.immich_hint.is_empty() { "API key".into() } else { format!("Stored {}", form.immich_hint) }} value={form.immich_api_key.clone()} oninput={set_input("immich_api_key")} />
                        </label>
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "Album ID" }</span>
                            <input class="field" type="text" value={form.immich_album_id.clone()} oninput={set_input("immich_album_id")} />
                        </label>
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "Shared key (optional)" }</span>
                            <input class="field" type="password" autocomplete="off" value={form.immich_shared_key.clone()} oninput={set_input("immich_shared_key")} />
                        </label>
                    </div>
                }

                if provider == "nextcloud" {
                    <div class="space-y-3">
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "Nextcloud URL" }</span>
                            <input class="field" type="url" placeholder="https://cloud.example.com" value={form.nextcloud_url.clone()} oninput={set_input("nextcloud_url")} />
                        </label>
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "User" }</span>
                            <input class="field" type="text" value={form.nextcloud_user.clone()} oninput={set_input("nextcloud_user")} />
                        </label>
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "App password" }</span>
                            <input class="field" type="password" autocomplete="off" placeholder={if form.nextcloud_hint.is_empty() { "App password".into() } else { format!("Stored {}", form.nextcloud_hint) }} value={form.nextcloud_app_password.clone()} oninput={set_input("nextcloud_app_password")} />
                        </label>
                        <label class="stack-gap-xs">
                            <span class="label-xs">{ "Folder" }</span>
                            <input class="field" type="text" placeholder="Photos/Site" value={form.nextcloud_folder.clone()} oninput={set_input("nextcloud_folder")} />
                        </label>
                    </div>
                }

                if provider == "static" {
                    <label class="stack-gap-xs">
                        <span class="label-xs">{ "Static JSON (optional — demo URLs if empty)" }</span>
                        <textarea class="field" rows="4" value={form.feed_static_json.clone()} oninput={set_input("feed_static_json")} />
                    </label>
                }

                <div class="space-y-3">
                    <h3 class="title-sm">{ "Site" }</h3>
                    <label class="stack-gap-xs">
                        <span class="label-xs">{ "Contact email" }</span>
                        <input class="field" type="email" value={form.contact_email.clone()} oninput={set_input("contact_email")} />
                    </label>
                    <label class="stack-gap-xs">
                        <span class="label-xs">{ "Public URL" }</span>
                        <input class="field" type="url" placeholder="https://jheff.media" value={form.public_url.clone()} oninput={set_input("public_url")} />
                    </label>
                    <p class="muted">{ "CORS origins stay in env (`YEW_FULLSTACK_CORS_ORIGINS`) and need a process restart to change." }</p>
                </div>

                if *restart {
                    <p class="admin-warn" role="status">
                        { "Restart required to repopulate images: " }
                        <code>{ "jheffmedia --restart" }</code>
                    </p>
                }

                <div class="row-actions">
                    <button type="button" class="btn btn-primary" onclick={on_save}>{ "Save setup" }</button>
                </div>
            }
            if let Some(msg) = &*note {
                <p class="muted" role="status">{ msg }</p>
            }
        </section>
    }
}
