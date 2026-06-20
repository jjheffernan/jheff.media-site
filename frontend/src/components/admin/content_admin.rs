use crate::{
    components::ui::{Heading, HeadingLevel, Text, TextTone},
    context::AuthContext,
    model::ContentCollection,
    services::auth_header,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component(ContentAdmin)]
pub fn content_admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let message = use_state(|| None::<String>);
    let gallery_id = use_state(String::new);
    let gallery_title = use_state(String::new);
    let shoot_id = use_state(String::new);
    let shoot_title = use_state(String::new);

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
                                    message.set(Some("Gallery created.".into()));
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
                                    message.set(Some("Shoot created.".into()));
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

    html! {
        <section class="space-y-6 rounded-2xl border border-border bg-surface-elevated p-6">
            <Heading level={HeadingLevel::H2} subtitle="Admin only — create new gallery or shoot collections.">
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
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
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
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
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
                        class="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-zinc-950"
                        onclick={on_create_gallery}
                    >
                        { "Create gallery" }
                    </button>
                </div>

                <div class="space-y-3">
                    <Text tone={TextTone::Muted}>{ "New shoot" }</Text>
                    <input
                        type="text"
                        placeholder="ID (slug)"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
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
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
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
                        class="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-zinc-950"
                        onclick={on_create_shoot}
                    >
                        { "Create shoot" }
                    </button>
                </div>
            </div>
        </section>
    }
}
