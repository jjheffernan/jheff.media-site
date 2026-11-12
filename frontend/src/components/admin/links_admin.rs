use crate::{context::AuthContext, services::auth_header};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct SocialLink {
    label: String,
    url: String,
}

#[derive(Serialize)]
struct SaveBody {
    links: Vec<SocialLink>,
}

#[derive(Deserialize)]
struct LinksResponse {
    links: Vec<SocialLink>,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SocialConfig {
    instagram_handle: String,
    instagram_profile_url: String,
    instagram_token_set: bool,
    instagram_token_hint: String,
    youtube_handle: String,
    youtube_channel_url: String,
    youtube_channel_id: String,
    youtube_api_key_set: bool,
    youtube_api_key_hint: String,
    other_platform: String,
    other_handle: String,
    other_profile_url: String,
    other_api_key_set: bool,
    other_api_key_hint: String,
    from_db: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SocialAdminResponse {
    config: SocialConfig,
}

#[derive(Clone, PartialEq)]
struct IntegrationForm {
    instagram_handle: String,
    instagram_profile_url: String,
    instagram_access_token: String,
    instagram_hint: String,
    youtube_handle: String,
    youtube_channel_url: String,
    youtube_channel_id: String,
    youtube_api_key: String,
    youtube_hint: String,
    other_platform: String,
    other_handle: String,
    other_profile_url: String,
    other_api_key: String,
    other_hint: String,
}

impl Default for IntegrationForm {
    fn default() -> Self {
        Self {
            instagram_handle: String::new(),
            instagram_profile_url: String::new(),
            instagram_access_token: String::new(),
            instagram_hint: String::new(),
            youtube_handle: String::new(),
            youtube_channel_url: String::new(),
            youtube_channel_id: String::new(),
            youtube_api_key: String::new(),
            youtube_hint: String::new(),
            other_platform: String::new(),
            other_handle: String::new(),
            other_profile_url: String::new(),
            other_api_key: String::new(),
            other_hint: String::new(),
        }
    }
}

impl IntegrationForm {
    fn from_config(cfg: &SocialConfig) -> Self {
        Self {
            instagram_handle: cfg.instagram_handle.clone(),
            instagram_profile_url: cfg.instagram_profile_url.clone(),
            instagram_access_token: String::new(),
            instagram_hint: if cfg.instagram_token_set {
                cfg.instagram_token_hint.clone()
            } else {
                String::new()
            },
            youtube_handle: cfg.youtube_handle.clone(),
            youtube_channel_url: cfg.youtube_channel_url.clone(),
            youtube_channel_id: cfg.youtube_channel_id.clone(),
            youtube_api_key: String::new(),
            youtube_hint: if cfg.youtube_api_key_set {
                cfg.youtube_api_key_hint.clone()
            } else {
                String::new()
            },
            other_platform: cfg.other_platform.clone(),
            other_handle: cfg.other_handle.clone(),
            other_profile_url: cfg.other_profile_url.clone(),
            other_api_key: String::new(),
            other_hint: if cfg.other_api_key_set {
                cfg.other_api_key_hint.clone()
            } else {
                String::new()
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SocialSaveBody {
    instagram_handle: String,
    instagram_profile_url: String,
    instagram_access_token: String,
    youtube_handle: String,
    youtube_channel_url: String,
    youtube_channel_id: String,
    youtube_api_key: String,
    other_platform: String,
    other_handle: String,
    other_profile_url: String,
    other_api_key: String,
}

fn set_field(links: &UseStateHandle<Vec<SocialLink>>, index: usize, label: bool, value: String) {
    let mut next = (**links).clone();
    if let Some(link) = next.get_mut(index) {
        if label {
            link.label = value;
        } else {
            link.url = value;
        }
        links.set(next);
    }
}

#[function_component(LinksAdmin)]
pub fn links_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let links = use_state(Vec::<SocialLink>::new);
    let integrations = use_state(IntegrationForm::default);
    let note = use_state(|| None::<String>);
    let social_note = use_state(|| None::<String>);
    let jwt = auth_ctx
        .auth
        .as_ref()
        .filter(|auth| auth.user.role == "admin")
        .map(|auth| auth.jwt.clone());

    {
        let links = links.clone();
        let integrations = integrations.clone();
        let note = note.clone();
        let social_note = social_note.clone();
        let jwt = jwt.clone();
        use_effect_with(jwt, move |jwt| {
            if let Some(jwt) = jwt.clone() {
                let links = links.clone();
                let integrations = integrations.clone();
                let note = note.clone();
                let social_note = social_note.clone();
                spawn_local(async move {
                    match Request::get("/api/links").send().await {
                        Ok(resp) if resp.ok() => match resp.json::<LinksResponse>().await {
                            Ok(body) => links.set(body.links),
                            Err(_) => note.set(Some("Could not read links.".into())),
                        },
                        Ok(_) => note.set(Some("Could not load links.".into())),
                        Err(err) => note.set(Some(err.to_string())),
                    }

                    let auth_val = auth_header(&jwt);
                    match Request::get("/api/admin/social")
                        .header("Authorization", auth_val.as_str())
                        .send()
                        .await
                    {
                        Ok(resp) if resp.ok() => match resp.json::<SocialAdminResponse>().await {
                            Ok(body) => integrations.set(IntegrationForm::from_config(&body.config)),
                            Err(_) => social_note.set(Some("Could not read integrations.".into())),
                        },
                        Ok(_) => social_note.set(Some("Could not load integrations.".into())),
                        Err(err) => social_note.set(Some(err.to_string())),
                    }
                });
            }
            || ()
        });
    }

    let set_integration = |field: &'static str| {
        let integrations = integrations.clone();
        Callback::from(move |e: InputEvent| {
            let value = e
                .target_dyn_into::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            let mut next = (*integrations).clone();
            match field {
                "instagram_handle" => next.instagram_handle = value,
                "instagram_profile_url" => next.instagram_profile_url = value,
                "instagram_access_token" => next.instagram_access_token = value,
                "youtube_handle" => next.youtube_handle = value,
                "youtube_channel_url" => next.youtube_channel_url = value,
                "youtube_channel_id" => next.youtube_channel_id = value,
                "youtube_api_key" => next.youtube_api_key = value,
                "other_platform" => next.other_platform = value,
                "other_handle" => next.other_handle = value,
                "other_profile_url" => next.other_profile_url = value,
                "other_api_key" => next.other_api_key = value,
                _ => {}
            }
            integrations.set(next);
        })
    };

    let on_save_integrations = {
        let integrations = integrations.clone();
        let social_note = social_note.clone();
        let jwt = jwt.clone();
        Callback::from(move |_| {
            let Some(jwt) = jwt.clone() else {
                return;
            };
            let f = (*integrations).clone();
            let integrations = integrations.clone();
            let social_note = social_note.clone();
            spawn_local(async move {
                let body = SocialSaveBody {
                    instagram_handle: f.instagram_handle,
                    instagram_profile_url: f.instagram_profile_url,
                    instagram_access_token: f.instagram_access_token,
                    youtube_handle: f.youtube_handle,
                    youtube_channel_url: f.youtube_channel_url,
                    youtube_channel_id: f.youtube_channel_id,
                    youtube_api_key: f.youtube_api_key,
                    other_platform: f.other_platform,
                    other_handle: f.other_handle,
                    other_profile_url: f.other_profile_url,
                    other_api_key: f.other_api_key,
                };
                let Ok(payload) = serde_json::to_string(&body) else {
                    return;
                };
                let auth_val = auth_header(&jwt);
                let sent = Request::put("/api/admin/social")
                    .header("Authorization", auth_val.as_str())
                    .header("Content-Type", "application/json")
                    .body(payload);
                match sent {
                    Ok(req) => match req.send().await {
                        Ok(resp) if resp.ok() => match resp.json::<SocialConfig>().await {
                            Ok(cfg) => {
                                integrations.set(IntegrationForm::from_config(&cfg));
                                social_note.set(Some(
                                    "Integrations saved. Feeds read Mongo on each request — no restart."
                                        .into(),
                                ));
                            }
                            Err(_) => {
                                social_note.set(Some("Saved, but the reply could not be read.".into()))
                            }
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
                                .unwrap_or_else(|| "Could not save integrations.".into());
                            social_note.set(Some(message));
                        }
                        Err(err) => social_note.set(Some(err.to_string())),
                    },
                    Err(err) => social_note.set(Some(err.to_string())),
                }
            });
        })
    };

    let on_add = {
        let links = links.clone();
        Callback::from(move |_| {
            if links.len() >= 12 {
                return;
            }
            let mut next = (*links).clone();
            next.push(SocialLink {
                label: String::new(),
                url: String::new(),
            });
            links.set(next);
        })
    };

    let on_save = {
        let links = links.clone();
        let note = note.clone();
        let jwt = jwt.clone();
        Callback::from(move |_| {
            let Some(jwt) = jwt.clone() else {
                return;
            };
            let payload: Vec<SocialLink> = links
                .iter()
                .filter(|link| !link.label.trim().is_empty() || !link.url.trim().is_empty())
                .cloned()
                .collect();
            let note = note.clone();
            let links = links.clone();
            spawn_local(async move {
                let Ok(body) = serde_json::to_string(&SaveBody { links: payload }) else {
                    return;
                };
                let auth_val = auth_header(&jwt);
                let sent = Request::put("/api/admin/links")
                    .header("Authorization", auth_val.as_str())
                    .header("Content-Type", "application/json")
                    .body(body);
                let resp = match sent {
                    Ok(req) => req.send().await,
                    Err(err) => {
                        note.set(Some(err.to_string()));
                        return;
                    }
                };
                match resp {
                    Ok(resp) if resp.ok() => match resp.json::<LinksResponse>().await {
                        Ok(body) => {
                            links.set(body.links);
                            note.set(Some("Saved.".into()));
                        }
                        Err(_) => note.set(Some("Saved, but the reply could not be read.".into())),
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
                            .unwrap_or_else(|| "Could not save links.".into());
                        note.set(Some(message));
                    }
                    Err(err) => note.set(Some(err.to_string())),
                }
            });
        })
    };

    let form = (*integrations).clone();

    html! {
        <div class="space-y-4">
            <section class="card space-y-3">
                <h2 class="title-sm">{ "Social integrations" }</h2>
                <p class="muted">
                    { "Connect Instagram, YouTube, and one other platform. Tokens stay in Mongo (same as Immich). Leave secret fields blank to keep the stored value." }
                </p>

                <h3 class="label-xs">{ "Instagram" }</h3>
                <div class="space-y-2">
                    <input class="field" type="text" placeholder="Handle (@jheffmedia)" value={form.instagram_handle.clone()} oninput={set_integration("instagram_handle")} />
                    <input class="field" type="url" placeholder="Profile URL" value={form.instagram_profile_url.clone()} oninput={set_integration("instagram_profile_url")} />
                    <input
                        class="field"
                        type="password"
                        autocomplete="off"
                        placeholder={if form.instagram_hint.is_empty() { "Access token".into() } else { format!("Stored {}", form.instagram_hint) }}
                        value={form.instagram_access_token.clone()}
                        oninput={set_integration("instagram_access_token")}
                    />
                </div>

                <h3 class="label-xs">{ "YouTube" }</h3>
                <div class="space-y-2">
                    <input class="field" type="text" placeholder="Handle (@jheffmedia)" value={form.youtube_handle.clone()} oninput={set_integration("youtube_handle")} />
                    <input class="field" type="url" placeholder="Channel URL" value={form.youtube_channel_url.clone()} oninput={set_integration("youtube_channel_url")} />
                    <input class="field" type="text" placeholder="Channel ID" value={form.youtube_channel_id.clone()} oninput={set_integration("youtube_channel_id")} />
                    <input
                        class="field"
                        type="password"
                        autocomplete="off"
                        placeholder={if form.youtube_hint.is_empty() { "API key".into() } else { format!("Stored {}", form.youtube_hint) }}
                        value={form.youtube_api_key.clone()}
                        oninput={set_integration("youtube_api_key")}
                    />
                </div>

                <h3 class="label-xs">{ "Other" }</h3>
                <div class="space-y-2">
                    <input class="field" type="text" placeholder="Platform (tiktok, x, …)" value={form.other_platform.clone()} oninput={set_integration("other_platform")} />
                    <input class="field" type="text" placeholder="Handle" value={form.other_handle.clone()} oninput={set_integration("other_handle")} />
                    <input class="field" type="url" placeholder="Profile URL" value={form.other_profile_url.clone()} oninput={set_integration("other_profile_url")} />
                    <input
                        class="field"
                        type="password"
                        autocomplete="off"
                        placeholder={if form.other_hint.is_empty() { "API key (optional)".into() } else { format!("Stored {}", form.other_hint) }}
                        value={form.other_api_key.clone()}
                        oninput={set_integration("other_api_key")}
                    />
                </div>

                <button type="button" class="btn btn-primary" onclick={on_save_integrations}>{ "Save integrations" }</button>
                if let Some(text) = &*social_note {
                    <p class="muted" role="status">{ text.clone() }</p>
                }
            </section>

            <section class="card space-y-3">
                <h2 class="title-sm">{ "Social links" }</h2>
                <p class="muted">{ "Static buttons on the public link page. Integrations above appear first when connected." }</p>
                { for links.iter().enumerate().map(|(index, link)| {
                    let links_for_label = links.clone();
                    let links_for_url = links.clone();
                    let links_for_remove = links.clone();
                    html! {
                        <div class="link-row" key={index}>
                            <input
                                type="text"
                                class="field"
                                placeholder="Label"
                                value={link.label.clone()}
                                oninput={Callback::from(move |event: InputEvent| {
                                    let value = event.target_unchecked_into::<HtmlInputElement>().value();
                                    set_field(&links_for_label, index, true, value);
                                })}
                            />
                            <input
                                type="url"
                                class="field"
                                placeholder="https://"
                                value={link.url.clone()}
                                oninput={Callback::from(move |event: InputEvent| {
                                    let value = event.target_unchecked_into::<HtmlInputElement>().value();
                                    set_field(&links_for_url, index, false, value);
                                })}
                            />
                            <button
                                type="button"
                                class="btn btn-quiet"
                                onclick={Callback::from(move |_| {
                                    let mut next = (*links_for_remove).clone();
                                    if index < next.len() {
                                        next.remove(index);
                                        links_for_remove.set(next);
                                    }
                                })}
                            >
                                { "Remove" }
                            </button>
                        </div>
                    }
                }) }
                <div class="flex flex-wrap gap-2">
                    <button type="button" class="btn btn-quiet" onclick={on_add}>{ "Add a link" }</button>
                    <button type="button" class="btn btn-primary" onclick={on_save}>{ "Save links" }</button>
                </div>
                if let Some(text) = &*note {
                    <p class="muted">{ text.clone() }</p>
                }
            </section>
        </div>
    }
}
