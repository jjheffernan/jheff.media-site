use crate::{context::AuthContext, services::auth_header};
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ContactItem {
    id: String,
    kind: String,
    name: String,
    email: String,
    subject: String,
    message: String,
    created_at: String,
    status: String,
}

async fn api_error(resp: gloo_net::http::Response, fallback: &str) -> String {
    if let Ok(body) = resp.json::<serde_json::Value>().await {
        body.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or(fallback)
            .to_string()
    } else {
        fallback.to_string()
    }
}

async fn load_contacts(jwt: &str) -> Result<Vec<ContactItem>, String> {
    let auth_val = auth_header(jwt);
    let resp = Request::get("/api/admin/contacts")
        .header("Authorization", auth_val.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(api_error(resp, "Could not load inbox.").await)
    }
}

async fn patch_status(jwt: &str, id: &str, status: &str) -> Result<(), String> {
    let auth_val = auth_header(jwt);
    let body = serde_json::json!({ "status": status }).to_string();
    let resp = Request::patch(&format!("/api/admin/contacts/{id}"))
        .header("Authorization", auth_val.as_str())
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else {
        Err(api_error(resp, "Could not update status.").await)
    }
}

async fn delete_contact(jwt: &str, id: &str) -> Result<(), String> {
    let auth_val = auth_header(jwt);
    let resp = Request::delete(&format!("/api/admin/contacts/{id}"))
        .header("Authorization", auth_val.as_str())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else {
        Err(api_error(resp, "Could not delete message.").await)
    }
}

#[function_component(ContactAdmin)]
pub fn contact_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let items = use_state(Vec::<ContactItem>::new);
    let message = use_state(|| None::<String>);
    let loaded = use_state(|| false);
    let tick = use_state(|| 0u32);
    let jwt = auth_ctx
        .auth
        .as_ref()
        .filter(|auth| auth.user.role == "admin")
        .map(|auth| auth.jwt.clone());

    {
        let items = items.clone();
        let message = message.clone();
        let loaded = loaded.clone();
        let jwt = jwt.clone();
        use_effect_with((*tick, jwt), move |(_, jwt)| {
            if let Some(jwt) = jwt.clone() {
                let items = items.clone();
                let message = message.clone();
                let loaded = loaded.clone();
                spawn_local(async move {
                    match load_contacts(&jwt).await {
                        Ok(list) => {
                            items.set(list);
                            message.set(None);
                            loaded.set(true);
                        }
                        Err(err) => message.set(Some(err)),
                    }
                });
            }
            || ()
        });
    }

    let Some(jwt) = jwt else {
        return html! {};
    };

    let rows = items.iter().map(|item| {
        let on_status = {
            let jwt = jwt.clone();
            let id = item.id.clone();
            let tick = tick.clone();
            let message = message.clone();
            Callback::from(move |e: Event| {
                let status = e.target_unchecked_into::<HtmlInputElement>().value();
                let jwt = jwt.clone();
                let id = id.clone();
                let tick = tick.clone();
                let message = message.clone();
                spawn_local(async move {
                    match patch_status(&jwt, &id, &status).await {
                        Ok(()) => tick.set(*tick + 1),
                        Err(err) => message.set(Some(err)),
                    }
                });
            })
        };
        let on_delete = {
            let jwt = jwt.clone();
            let id = item.id.clone();
            let tick = tick.clone();
            let message = message.clone();
            Callback::from(move |_| {
                let jwt = jwt.clone();
                let id = id.clone();
                let tick = tick.clone();
                let message = message.clone();
                spawn_local(async move {
                    match delete_contact(&jwt, &id).await {
                        Ok(()) => tick.set(*tick + 1),
                        Err(err) => message.set(Some(err)),
                    }
                });
            })
        };
        let known = item.status == "new" || item.status == "read" || item.status == "archived";
        html! {
            <div key={item.id.clone()} class="space-y-2">
                <p>
                    <strong>{ item.name.clone() }</strong>
                    { " " }
                    <span class="muted">{ item.email.clone() }</span>
                </p>
                <p class="muted">{ format!("{} · {} · {}", item.kind, item.status, item.created_at) }</p>
                if !item.subject.is_empty() {
                    <p>{ item.subject.clone() }</p>
                }
                <p>{ item.message.clone() }</p>
                <select class="field" value={item.status.clone()} onchange={on_status}>
                    <option value="new">{ "new" }</option>
                    <option value="read">{ "read" }</option>
                    <option value="archived">{ "archived" }</option>
                    if !known {
                        <option value={item.status.clone()}>{ item.status.clone() }</option>
                    }
                </select>
                <button type="button" class="btn btn-quiet" onclick={on_delete}>{ "Delete" }</button>
            </div>
        }
    });

    html! {
        <section class="card space-y-4">
            <h2 class="title-sm">{ "Inbox" }</h2>
            <p class="muted">{ "Contact and booking messages, newest first." }</p>
            if let Some(msg) = &*message {
                <p class="muted">{ msg }</p>
            }
            if *loaded && items.is_empty() && message.is_none() {
                <p class="muted">{ "No messages." }</p>
            }
            { for rows }
        </section>
    }
}
