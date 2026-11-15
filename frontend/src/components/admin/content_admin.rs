use crate::{
    components::admin::collection_browser::CollectionBrowser,
    components::admin::library_import::LibraryImport,
    components::ui::{
        ConfirmDialog, DatePicker, Field, Heading, HeadingLevel, TabItem, Tabs, Text, TextTone,
    },
    context::AuthContext,
    model::AdminPrintOrder,
    services::{get_json_auth, ok, patch_json_auth, patch_ok_auth, post_json_auth, HttpMethod},
    utils::with_local_offset,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

fn admin_method(method: &str) -> Result<HttpMethod, String> {
    match method {
        "PATCH" => Ok(HttpMethod::PATCH),
        "PUT" => Ok(HttpMethod::PUT),
        "DELETE" => Ok(HttpMethod::DELETE),
        "POST" => Ok(HttpMethod::POST),
        _ => Err("Unsupported method.".into()),
    }
}

async fn admin_json(
    method: &str,
    url: &str,
    jwt: &str,
    body: Option<String>,
) -> Result<(), String> {
    ok(admin_method(method)?, url, Some(jwt), body).await
}

async fn create_schedule_event(jwt: &str, payload: serde_json::Value) -> Result<String, String> {
    let body = post_json_auth::<_, serde_json::Value>("/api/admin/schedule", jwt, &payload).await?;
    body.get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "Failed to add date.".to_string())
}

fn schedule_payload(
    title: &str,
    starts: &str,
    location: &str,
    kind: &str,
) -> Result<serde_json::Value, String> {
    let title = title.trim().to_string();
    let starts_at = with_local_offset(starts);
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
    patch_ok_auth(&format!("/api/admin/schedule/{id}"), jwt, &payload).await
}

async fn delete_schedule_event(jwt: &str, id: &str) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Enter a date ID.".into());
    }
    admin_json("DELETE", &format!("/api/admin/schedule/{id}"), jwt, None).await
}


fn bind_input(state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        state.set(e.target_unchecked_into::<HtmlInputElement>().value());
    })
}

fn split_starts(value: &str) -> (String, String) {
    let value = value.trim();
    if value.len() >= 16 && value.as_bytes().get(10) == Some(&b'T') {
        let day = value[..10].to_string();
        let time = value[11..].chars().take(5).collect();
        (day, time)
    } else if value.len() == 10 {
        (value.to_string(), "12:00".into())
    } else {
        (String::new(), "12:00".into())
    }
}

fn combine_starts(day: &str, time: &str) -> String {
    let day = day.trim();
    let time = time.trim();
    if day.len() != 10 {
        return String::new();
    }
    let time = if time.len() >= 5 {
        time.chars().take(5).collect::<String>()
    } else {
        "12:00".into()
    };
    format!("{day}T{time}")
}

/// Soft slug for Advanced preview; real Mongo id is filled after Add.
fn schedule_slug(title: &str, starts: &str) -> String {
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
    let day = starts.trim().chars().take(10).collect::<String>();
    if day.len() == 10 {
        if !out.is_empty() {
            out.push('-');
        }
        out.push_str(&day);
    }
    let s = out.trim_matches('-').to_string();
    if s.is_empty() {
        "date".into()
    } else {
        s
    }
}

fn looks_like_object_id(id: &str) -> bool {
    let id = id.trim();
    id.len() == 24 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

#[derive(Clone, Copy, PartialEq)]
enum ContentTab {
    Galleries,
    Shoots,
    Schedule,
    Library,
    Orders,
}

#[function_component(ContentAdmin)]
pub fn content_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let message = use_state(|| None::<String>);
    let date_title = use_state(String::new);
    let date_starts = use_state(String::new);
    let date_location = use_state(String::new);
    let date_kind = use_state(|| String::from("shoot"));
    let date_id = use_state(String::new);
    let date_id_touched = use_state(|| false);
    let content_tab = use_state(|| ContentTab::Galleries);
    let confirm_delete = use_state(|| false);

    let (starts_day, starts_time) = split_starts(&date_starts);

    if auth_ctx.auth.as_ref().map(|a| a.user.role.as_str()) != Some("admin") {
        return html! {};
    }


    let on_starts_day = {
        let date_starts = date_starts.clone();
        let date_id = date_id.clone();
        let date_id_touched = date_id_touched.clone();
        let date_title = date_title.clone();
        Callback::from(move |day: String| {
            let (_, time) = split_starts(&date_starts);
            let combined = combine_starts(&day, &time);
            date_starts.set(combined.clone());
            if !*date_id_touched {
                date_id.set(schedule_slug(&date_title, &combined));
            }
        })
    };

    let on_starts_time = {
        let date_starts = date_starts.clone();
        let date_id = date_id.clone();
        let date_id_touched = date_id_touched.clone();
        let date_title = date_title.clone();
        Callback::from(move |e: InputEvent| {
            let time = e.target_unchecked_into::<HtmlInputElement>().value();
            let (day, _) = split_starts(&date_starts);
            let combined = combine_starts(&day, &time);
            date_starts.set(combined.clone());
            if !*date_id_touched {
                date_id.set(schedule_slug(&date_title, &combined));
            }
        })
    };

    let on_title_input = {
        let date_title = date_title.clone();
        let date_starts = date_starts.clone();
        let date_id = date_id.clone();
        let date_id_touched = date_id_touched.clone();
        Callback::from(move |e: InputEvent| {
            let title = e.target_unchecked_into::<HtmlInputElement>().value();
            date_title.set(title.clone());
            if !*date_id_touched {
                date_id.set(schedule_slug(&title, &date_starts));
            }
        })
    };

    let on_add_date = {
        let auth_ctx = auth_ctx.clone();
        let date_title = date_title.clone();
        let date_starts = date_starts.clone();
        let date_location = date_location.clone();
        let date_kind = date_kind.clone();
        let date_id = date_id.clone();
        let date_id_touched = date_id_touched.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let title = date_title.trim().to_string();
            let starts_at = with_local_offset(&date_starts);
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
            let date_id = date_id.clone();
            let date_id_touched = date_id_touched.clone();
            spawn_local(async move {
                match create_schedule_event(&jwt, payload).await {
                    Ok(id) => {
                        date_title.set(String::new());
                        date_starts.set(String::new());
                        date_location.set(String::new());
                        date_id.set(id.clone());
                        date_id_touched.set(true);
                        message.set(Some(format!("Date added. ID ready for edit/delete: {id}")));
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

    let on_ask_delete_date = {
        let confirm_delete = confirm_delete.clone();
        Callback::from(move |_| confirm_delete.set(true))
    };
    let on_cancel_delete = {
        let confirm_delete = confirm_delete.clone();
        Callback::from(move |_: ()| confirm_delete.set(false))
    };
    let on_delete_date = {
        let auth_ctx = auth_ctx.clone();
        let date_id = date_id.clone();
        let message = message.clone();
        let confirm_delete = confirm_delete.clone();
        Callback::from(move |_: ()| {
            confirm_delete.set(false);
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

    let tab = *content_tab;
    let tab_items = vec![
        TabItem { id: "galleries".into(), label: "Galleries".into() },
        TabItem { id: "shoots".into(), label: "Shoots".into() },
        TabItem { id: "schedule".into(), label: "Schedule".into() },
        TabItem { id: "library".into(), label: "Library".into() },
        TabItem { id: "orders".into(), label: "Orders".into() },
    ];
    let active_tab = match tab {
        ContentTab::Galleries => "galleries",
        ContentTab::Shoots => "shoots",
        ContentTab::Schedule => "schedule",
        ContentTab::Library => "library",
        ContentTab::Orders => "orders",
    }
    .to_string();
    let on_tab = {
        let content_tab = content_tab.clone();
        Callback::from(move |id: String| {
            let next = match id.as_str() {
                "shoots" => ContentTab::Shoots,
                "schedule" => ContentTab::Schedule,
                "library" => ContentTab::Library,
                "orders" => ContentTab::Orders,
                _ => ContentTab::Galleries,
            };
            content_tab.set(next);
        })
    };

    html! {
        <section class="card space-y-6">
            <ConfirmDialog
                open={*confirm_delete}
                title={"Delete date"}
                message={"Remove this schedule date? This cannot be undone."}
                confirm_label={"Delete"}
                destructive=true
                on_confirm={on_delete_date}
                on_cancel={on_cancel_delete}
            />
            <Heading level={HeadingLevel::H2} subtitle="One tool at a time.">
                { "Content" }
            </Heading>
            <Tabs items={tab_items} active={active_tab} on_change={on_tab} />

            if let Some(msg) = &*message {
                <p class="text-sm text-accent">{ msg }</p>
            }

            if tab == ContentTab::Galleries {
                <CollectionBrowser kind="galleries" noun="gallery" plural="galleries" />
            }
            if tab == ContentTab::Shoots {
                <CollectionBrowser kind="shoots" noun="shoot" plural="shoots" />
            }
            if tab == ContentTab::Library {
                <LibraryImport />
            }
            if tab == ContentTab::Schedule {
            <div class="schedule-editor">
                <Field label={"Add date"}>
                    <input
                        type="text"
                        placeholder="Title"
                        class="field"
                        value={(*date_title).clone()}
                        oninput={on_title_input}
                    />
                </Field>
                <Field label={"Starts"}>
                    <div class="starts-picker">
                        <DatePicker
                            value={starts_day}
                            on_change={on_starts_day}
                            placeholder="YYYY-MM-DD"
                        />
                        <input
                            type="time"
                            class="field starts-picker-time"
                            value={starts_time}
                            oninput={on_starts_time}
                        />
                    </div>
                </Field>
                <Field label={"Location"}>
                    <input
                        type="text"
                        placeholder="Location"
                        class="field"
                        value={(*date_location).clone()}
                        oninput={bind_input(date_location.clone())}
                    />
                </Field>
                <Field label={"Kind"}>
                    <select
                        class="field select"
                        value={(*date_kind).clone()}
                        onchange={on_kind}
                    >
                        <option value="shoot">{ "Shoot" }</option>
                        <option value="instruction">{ "Instruction" }</option>
                        <option value="other">{ "Other" }</option>
                    </select>
                </Field>
                <div class="schedule-editor-actions">
                    <button type="button" class="btn btn-primary" onclick={on_add_date}>
                        { "Add date" }
                    </button>
                </div>
                if looks_like_object_id(&date_id) {
                    <p class="muted label-xs">{ format!("Current date id: {}", date_id.trim()) }</p>
                }
                <details class="schedule-advanced">
                    <summary>{ "Advanced" }</summary>
                    <div class="schedule-advanced-body">
                        <label class="block space-y-1">
                            <Text tone={TextTone::Muted}>{ "Date ID" }</Text>
                            <input
                                type="text"
                                placeholder="Auto from title · server id after Add"
                                class="field"
                                value={(*date_id).clone()}
                                oninput={{
                                    let date_id = date_id.clone();
                                    let date_id_touched = date_id_touched.clone();
                                    Callback::from(move |e: InputEvent| {
                                        date_id_touched.set(true);
                                        date_id.set(
                                            e.target_unchecked_into::<HtmlInputElement>().value(),
                                        );
                                    })
                                }}
                            />
                            <p class="muted label-xs">
                                { "Leave alone for Add. Save/Delete need the server id (filled after Add, or paste)." }
                            </p>
                        </label>
                        <div class="schedule-editor-actions">
                            <button type="button" class="btn btn-primary" onclick={on_save_date}>
                                { "Save" }
                            </button>
                            <button type="button" class="btn btn-destructive" onclick={on_ask_delete_date}>
                                { "Delete" }
                            </button>
                        </div>
                    </div>
                </details>
            </div>
            }
            if tab == ContentTab::Orders {
                <PrintOrdersAdmin />
            }
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
    get_json_auth("/api/admin/prints/orders", jwt).await
}

async fn patch_print_order(
    jwt: &str,
    id: &str,
    status: &str,
    payment_status: &str,
) -> Result<AdminPrintOrder, String> {
    patch_json_auth(
        &format!("/api/admin/prints/orders/{id}"),
        jwt,
        &serde_json::json!({
            "status": status,
            "paymentStatus": payment_status,
        }),
    )
    .await
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
