use crate::{
    components::{
        admin::ContentAdmin,
        ui::{Heading, HeadingLevel, LazyImage, Text, TextTone},
    },
    context::AuthContext,
    model::{
        ChangePassword, EmailChange, PrintSelection, ServerResponse, TotpEnrollResponse, TotpVerify,
        User,
    },
    routes::AppRoutes,
    services::auth_header,
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
enum PrintsState {
    Loading,
    Ready(Vec<PrintSelection>),
    Error(String),
}

#[function_component(Account)]
pub fn account() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let navigator = use_navigator().unwrap();
    let user = use_state(|| None::<User>);
    let message = use_state(|| None::<String>);
    let totp_setup = use_state(|| None::<TotpEnrollResponse>);
    let prints = use_state(|| PrintsState::Loading);

    let current_password = use_state(String::new);
    let new_password = use_state(String::new);
    let new_email = use_state(String::new);
    let totp_code = use_state(String::new);

    if auth_ctx.auth.is_none() {
        navigator.push(&AppRoutes::Home);
        return html! {
            <Text tone={TextTone::Muted}>{ "Sign in to manage your account." }</Text>
        };
    }

    let auth = auth_ctx.auth.clone().unwrap();
    let jwt = auth.jwt.clone();

    {
        let user = user.clone();
        let jwt = jwt.clone();
        use_effect_with(jwt.clone(), move |jwt| {
            let jwt = jwt.clone();
            spawn_local(async move {
                let auth_val = auth_header(&jwt);
                if let Ok(resp) = Request::get("/api/auth/me")
                    .header("Authorization", auth_val.as_str())
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(me) = resp.json::<User>().await {
                            user.set(Some(me));
                        }
                    }
                }
            });
            || ()
        });
    }

    {
        let prints = prints.clone();
        let jwt = jwt.clone();
        use_effect_with(jwt.clone(), move |jwt| {
            let jwt = jwt.clone();
            spawn_local(async move {
                let auth_val = auth_header(&jwt);
                match Request::get("/api/account/prints")
                    .header("Authorization", auth_val.as_str())
                    .send()
                    .await
                {
                    Ok(resp) if resp.ok() => {
                        if let Ok(items) = resp.json::<Vec<PrintSelection>>().await {
                            prints.set(PrintsState::Ready(items));
                            return;
                        }
                        prints.set(PrintsState::Error("Could not load print selections.".into()));
                    }
                    Ok(_) => prints.set(PrintsState::Error("Print selections unavailable.".into())),
                    Err(err) => prints.set(PrintsState::Error(err.to_string())),
                }
            });
            || ()
        });
    }

    let on_change_password = {
        let jwt = jwt.clone();
        let current_password = current_password.clone();
        let new_password = new_password.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let jwt = jwt.clone();
            let dto = ChangePassword {
                current_password: (*current_password).clone(),
                new_password: (*new_password).clone(),
            };
            let message = message.clone();
            spawn_local(async move {
                if let Ok(body) = serde_json::to_string(&dto) {
                    let auth_val = auth_header(&jwt);
                    if let Ok(req) = Request::post("/api/auth/password")
                        .header("Authorization", auth_val.as_str())
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        if let Ok(resp) = req.send().await {
                            if resp.ok() {
                                message.set(Some("Password updated.".into()));
                            } else if let Ok(err) = resp.json::<ServerResponse<String>>().await {
                                message.set(Some(err.message));
                            }
                        }
                    }
                }
            });
        })
    };

    let on_request_email = {
        let jwt = jwt.clone();
        let new_email = new_email.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let jwt = jwt.clone();
            let dto = EmailChange {
                new_email: (*new_email).clone(),
            };
            let message = message.clone();
            spawn_local(async move {
                if let Ok(body) = serde_json::to_string(&dto) {
                    let auth_val = auth_header(&jwt);
                    if let Ok(req) = Request::post("/api/auth/email-change")
                        .header("Authorization", auth_val.as_str())
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        if let Ok(resp) = req.send().await {
                            if resp.ok() {
                                message.set(Some("Email change requested — check your inbox when mail is configured.".into()));
                            } else if let Ok(err) = resp.json::<ServerResponse<String>>().await {
                                message.set(Some(err.message));
                            }
                        }
                    }
                }
            });
        })
    };

    let on_totp_enroll = {
        let jwt = jwt.clone();
        let totp_setup = totp_setup.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let jwt = jwt.clone();
            let totp_setup = totp_setup.clone();
            let message = message.clone();
            spawn_local(async move {
                let auth_val = auth_header(&jwt);
                if let Ok(resp) = Request::post("/api/auth/2fa/enroll")
                    .header("Authorization", auth_val.as_str())
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<TotpEnrollResponse>().await {
                            totp_setup.set(Some(data));
                            message.set(Some("Scan the OTP URL in your authenticator app, then enter a code to confirm.".into()));
                        }
                    }
                }
            });
        })
    };

    let on_totp_confirm = {
        let jwt = jwt.clone();
        let totp_code = totp_code.clone();
        let message = message.clone();
        let user = user.clone();
        Callback::from(move |_| {
            let jwt = jwt.clone();
            let dto = TotpVerify {
                code: (*totp_code).clone(),
            };
            let message = message.clone();
            let user = user.clone();
            spawn_local(async move {
                if let Ok(body) = serde_json::to_string(&dto) {
                    let auth_val = auth_header(&jwt);
                    if let Ok(req) = Request::post("/api/auth/2fa/confirm")
                        .header("Authorization", auth_val.as_str())
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        if let Ok(resp) = req.send().await {
                            if resp.ok() {
                                message.set(Some("Two-factor authentication enabled.".into()));
                                if let Some(mut u) = (*user).clone() {
                                    u.totp_enabled = true;
                                    user.set(Some(u));
                                }
                            }
                        }
                    }
                }
            });
        })
    };

    let on_totp_disable = {
        let jwt = jwt.clone();
        let totp_code = totp_code.clone();
        let message = message.clone();
        let user = user.clone();
        Callback::from(move |_| {
            let jwt = jwt.clone();
            let dto = TotpVerify {
                code: (*totp_code).clone(),
            };
            let message = message.clone();
            let user = user.clone();
            spawn_local(async move {
                if let Ok(body) = serde_json::to_string(&dto) {
                    let auth_val = auth_header(&jwt);
                    if let Ok(req) = Request::post("/api/auth/2fa/disable")
                        .header("Authorization", auth_val.as_str())
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        if let Ok(resp) = req.send().await {
                            if resp.ok() {
                                message.set(Some("Two-factor authentication disabled.".into()));
                                if let Some(mut u) = (*user).clone() {
                                    u.totp_enabled = false;
                                    user.set(Some(u));
                                }
                            }
                        }
                    }
                }
            });
        })
    };

    let display_user = (*user).clone().unwrap_or(auth.user.clone());
    let totp_enabled = display_user.totp_enabled;

    let prints_section = match &*prints {
        PrintsState::Loading => html! { <Text tone={TextTone::Muted}>{ "Loading print selections…" }</Text> },
        PrintsState::Error(msg) => html! { <p class="text-sm text-muted">{ msg }</p> },
        PrintsState::Ready(items) if items.is_empty() => html! {
            <Text tone={TextTone::Muted}>
                { "No photos selected for print yet. Browse the feed and tap “Select for print” on any image." }
            </Text>
        },
        PrintsState::Ready(items) => html! {
            <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                { for items.iter().map(|item| {
                    let jwt = jwt.clone();
                    let prints = prints.clone();
                    let id = item.id.clone();
                    let remove = Callback::from(move |_| {
                        let jwt = jwt.clone();
                        let id = id.clone();
                        let prints = prints.clone();
                        spawn_local(async move {
                            let url = format!("/api/account/prints/{}", id);
                            let auth_val = auth_header(&jwt);
                            if let Ok(resp) = Request::delete(&url)
                                .header("Authorization", auth_val.as_str())
                                .send()
                                .await
                            {
                                if resp.ok() {
                                    if let PrintsState::Ready(current) = &*prints {
                                        let next: Vec<PrintSelection> = current
                                            .iter()
                                            .filter(|p| p.id != id)
                                            .cloned()
                                            .collect();
                                        prints.set(PrintsState::Ready(next));
                                    }
                                }
                            }
                        });
                    });
                    html! {
                        <div key={item.id.clone()} class="flex gap-3 rounded-xl border border-border p-3">
                            <LazyImage src={item.thumbnail_url.clone()} alt={item.title.clone().unwrap_or_default()} />
                            <div class="flex-1 space-y-1">
                                <p class="text-sm font-medium">{ item.title.clone().unwrap_or_else(|| "Photo".into()) }</p>
                                <p class="text-xs text-muted">{ item.source.clone() }</p>
                                <button
                                    type="button"
                                    class="text-xs text-accent hover:underline"
                                    onclick={remove}
                                >
                                    { "Remove" }
                                </button>
                            </div>
                        </div>
                    }
                }) }
            </div>
        },
    };

    html! {
        <div class="space-y-10">
            <div class="rounded-2xl border border-border bg-surface-elevated p-6">
                <Heading level={HeadingLevel::H1} subtitle="Password, two-factor auth, email, and print selections.">
                    { html! { { format!("Account — {}", display_user.username) } } }
                </Heading>
                <div class="mt-4 space-y-1 text-sm text-muted">
                    <p>{ format!("Email: {}", display_user.email) }</p>
                    <p>{ format!("Role: {}", display_user.role) }</p>
                    <p>{ format!("2FA: {}", if totp_enabled { "enabled" } else { "not enabled" }) }</p>
                </div>
            </div>

            if let Some(msg) = &*message {
                <p class="rounded-lg border border-accent/30 bg-accent/10 px-4 py-3 text-sm text-foreground">{ msg }</p>
            }

            <section class="space-y-4 rounded-2xl border border-border bg-surface-elevated p-6">
                <Heading level={HeadingLevel::H2}>{ "Change password" }</Heading>
                <input
                    type="password"
                    placeholder="Current password"
                    class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
                    oninput={{
                        let current_password = current_password.clone();
                        Callback::from(move |e: InputEvent| {
                            current_password.set(e.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
                />
                <input
                    type="password"
                    placeholder="New password (min 8 characters)"
                    class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
                    oninput={{
                        let new_password = new_password.clone();
                        Callback::from(move |e: InputEvent| {
                            new_password.set(e.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
                />
                <button
                    type="button"
                    class="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-zinc-950"
                    onclick={on_change_password}
                >
                    { "Update password" }
                </button>
            </section>

            <section class="space-y-4 rounded-2xl border border-border bg-surface-elevated p-6">
                <Heading level={HeadingLevel::H2}>{ "Change email" }</Heading>
                <Text tone={TextTone::Muted}>
                    { "Request a verification link for your new email address." }
                </Text>
                <input
                    type="email"
                    placeholder="New email"
                    class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
                    oninput={{
                        let new_email = new_email.clone();
                        Callback::from(move |e: InputEvent| {
                            new_email.set(e.target_unchecked_into::<HtmlInputElement>().value());
                        })
                    }}
                />
                <button
                    type="button"
                    class="rounded-lg border border-border px-4 py-2 text-sm font-medium transition hover:border-accent hover:text-accent"
                    onclick={on_request_email}
                >
                    { "Request email change" }
                </button>
            </section>

            <section class="space-y-4 rounded-2xl border border-border bg-surface-elevated p-6">
                <Heading level={HeadingLevel::H2}>{ "Two-factor authentication" }</Heading>
                if totp_enabled {
                    <Text tone={TextTone::Muted}>{ "Enter a code from your authenticator to disable 2FA." }</Text>
                    <input
                        type="text"
                        placeholder="6-digit code"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
                        oninput={{
                            let totp_code = totp_code.clone();
                            Callback::from(move |e: InputEvent| {
                                totp_code.set(e.target_unchecked_into::<HtmlInputElement>().value());
                            })
                        }}
                    />
                    <button
                        type="button"
                        class="rounded-lg border border-border px-4 py-2 text-sm font-medium transition hover:border-accent hover:text-accent"
                        onclick={on_totp_disable}
                    >
                        { "Disable 2FA" }
                    </button>
                } else {
                    <button
                        type="button"
                        class="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-zinc-950"
                        onclick={on_totp_enroll}
                    >
                        { "Enable 2FA" }
                    </button>
                    if let Some(setup) = &*totp_setup {
                        <div class="space-y-2 rounded-lg border border-border bg-surface p-4 text-sm">
                            <p class="text-muted">{ "Secret (manual entry):" }</p>
                            <code class="block text-xs">{ setup.secret.clone() }</code>
                            <a
                                href={setup.otpauth_url.clone()}
                                class="text-accent hover:underline"
                            >
                                { "Open in authenticator app" }
                            </a>
                        </div>
                        <input
                            type="text"
                            placeholder="6-digit code to confirm"
                            class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
                            oninput={{
                                let totp_code = totp_code.clone();
                                Callback::from(move |e: InputEvent| {
                                    totp_code.set(e.target_unchecked_into::<HtmlInputElement>().value());
                                })
                            }}
                        />
                        <button
                            type="button"
                            class="rounded-lg border border-border px-4 py-2 text-sm font-medium transition hover:border-accent hover:text-accent"
                            onclick={on_totp_confirm}
                        >
                            { "Confirm 2FA" }
                        </button>
                    }
                }
            </section>

            <section class="space-y-4 rounded-2xl border border-border bg-surface-elevated p-6">
                <Heading level={HeadingLevel::H2} subtitle="Photos you've marked for print purchase.">
                    { "Print selections" }
                </Heading>
                { prints_section }
            </section>

            <ContentAdmin />

            <button
                type="button"
                class="rounded-lg border border-border px-4 py-2 text-sm text-foreground transition hover:border-accent hover:text-accent"
                onclick={{
                    let auth_ctx = auth_ctx.clone();
                    Callback::from(move |_| auth_ctx.logout.emit(()))
                }}
            >
                { "Log out" }
            </button>
        </div>
    }
}
