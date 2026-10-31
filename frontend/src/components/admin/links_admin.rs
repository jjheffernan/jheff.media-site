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
    let note = use_state(|| None::<String>);
    let jwt = auth_ctx
        .auth
        .as_ref()
        .filter(|auth| auth.user.role == "admin")
        .map(|auth| auth.jwt.clone());

    {
        let links = links.clone();
        let note = note.clone();
        let jwt = jwt.clone();
        use_effect_with(jwt, move |jwt| {
            if jwt.is_some() {
                let links = links.clone();
                let note = note.clone();
                spawn_local(async move {
                    match Request::get("/api/links").send().await {
                        Ok(resp) if resp.ok() => match resp.json::<LinksResponse>().await {
                            Ok(body) => links.set(body.links),
                            Err(_) => note.set(Some("Could not read links.".into())),
                        },
                        Ok(_) => note.set(Some("Could not load links.".into())),
                        Err(err) => note.set(Some(err.to_string())),
                    }
                });
            }
            || ()
        });
    }

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
                            .and_then(|body| body.get("message").and_then(|m| m.as_str()).map(str::to_string))
                            .unwrap_or_else(|| "Could not save links.".into());
                        note.set(Some(message));
                    }
                    Err(err) => note.set(Some(err.to_string())),
                }
            });
        })
    };

    html! {
        <section class="card space-y-3">
            <h2 class="title-sm">{ "Social links" }</h2>
            <p class="muted">{ "These buttons are the public link page. Use https addresses." }</p>
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
    }
}
