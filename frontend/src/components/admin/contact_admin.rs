use crate::{
    components::ui::{Badge, BadgeVariant, ConfirmDialog, Empty, Table},
    context::AuthContext,
    services::{delete_ok_auth, get_json_auth, patch_ok_auth},
};
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
    #[serde(default)]
    preferred_at: String,
    created_at: String,
    status: String,
}

async fn load_contacts(jwt: &str) -> Result<Vec<ContactItem>, String> {
    get_json_auth("/api/admin/contacts", jwt).await
}

async fn patch_status(jwt: &str, id: &str, status: &str) -> Result<(), String> {
    patch_ok_auth(
        &format!("/api/admin/contacts/{id}"),
        jwt,
        &serde_json::json!({ "status": status }),
    )
    .await
}

async fn delete_contact(jwt: &str, id: &str) -> Result<(), String> {
    delete_ok_auth(&format!("/api/admin/contacts/{id}"), jwt).await
}

#[function_component(ContactAdmin)]
pub fn contact_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let items = use_state(Vec::<ContactItem>::new);
    let message = use_state(|| None::<String>);
    let loaded = use_state(|| false);
    let tick = use_state(|| 0u32);
    let pending_delete = use_state(|| None::<String>);
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

    let on_cancel_delete = {
        let pending_delete = pending_delete.clone();
        Callback::from(move |_: ()| pending_delete.set(None))
    };
    let on_confirm_delete = {
        let jwt = jwt.clone();
        let tick = tick.clone();
        let message = message.clone();
        let pending_delete = pending_delete.clone();
        Callback::from(move |_: ()| {
            let Some(id) = (*pending_delete).clone() else {
                return;
            };
            pending_delete.set(None);
            let jwt = jwt.clone();
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
        let on_ask_delete = {
            let pending_delete = pending_delete.clone();
            let id = item.id.clone();
            Callback::from(move |_| pending_delete.set(Some(id.clone())))
        };
        let known = item.status == "new" || item.status == "read" || item.status == "archived";
        let badge = match item.status.as_str() {
            "new" => BadgeVariant::Default,
            "archived" => BadgeVariant::Outline,
            _ => BadgeVariant::Secondary,
        };
        html! {
            <tr key={item.id.clone()}>
                <td>
                    <div class="stack-gap-xs">
                        <strong>{ item.name.clone() }</strong>
                        <span class="muted">{ item.email.clone() }</span>
                        <Badge variant={badge}>{ html! { { item.status.clone() } } }</Badge>
                    </div>
                </td>
                <td>
                    <div class="stack-gap-xs">
                        <span class="muted">{ format!("{} · {}", item.kind, item.created_at) }</span>
                        if !item.subject.is_empty() {
                            <p>{ item.subject.clone() }</p>
                        }
                        <p>{ item.message.clone() }</p>
                        if !item.preferred_at.is_empty() {
                            <p class="muted">{ format!("Preferred: {}", item.preferred_at) }</p>
                        }
                    </div>
                </td>
                <td>
                    <div class="row-actions">
                        <select class="field select" value={item.status.clone()} onchange={on_status}>
                            <option value="new">{ "new" }</option>
                            <option value="read">{ "read" }</option>
                            <option value="archived">{ "archived" }</option>
                            if !known {
                                <option value={item.status.clone()}>{ item.status.clone() }</option>
                            }
                        </select>
                        <button type="button" class="btn btn-destructive btn-sm" onclick={on_ask_delete}>
                            { "Delete" }
                        </button>
                    </div>
                </td>
            </tr>
        }
    });

    html! {
        <section class="card space-y-4">
            <ConfirmDialog
                open={pending_delete.is_some()}
                title={"Delete message"}
                message={"Remove this inbox message permanently?"}
                confirm_label={"Delete"}
                destructive=true
                on_confirm={on_confirm_delete}
                on_cancel={on_cancel_delete}
            />
            <h2 class="title-sm">{ "Inbox" }</h2>
            <p class="muted">{ "Contact and booking messages, newest first." }</p>
            if let Some(msg) = &*message {
                <p class="muted">{ msg }</p>
            }
            if *loaded && items.is_empty() && message.is_none() {
                <Empty
                    title={"No messages"}
                    description={"Contact and booking forms land here."}
                />
            } else if !items.is_empty() {
                <Table>
                    <thead>
                        <tr>
                            <th>{ "From" }</th>
                            <th>{ "Message" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </Table>
            }
        </section>
    }
}
