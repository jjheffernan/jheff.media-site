use crate::{
    components::auth::{AuthModal, AuthModalKind},
    context::AuthContext,
    routes::AppRoutes,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum OpenPanel {
    None,
    Menu,
    SignIn,
    SignUp,
}

#[function_component(ProfileMenu)]
pub fn profile_menu() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let panel = use_state(|| OpenPanel::None);

    let toggle_menu = {
        let panel = panel.clone();
        Callback::from(move |_| {
            panel.set(match *panel {
                OpenPanel::Menu => OpenPanel::None,
                _ => OpenPanel::Menu,
            });
        })
    };

    let open_sign_in = {
        let panel = panel.clone();
        Callback::from(move |_| panel.set(OpenPanel::SignIn))
    };

    let open_sign_up = {
        let panel = panel.clone();
        Callback::from(move |_| panel.set(OpenPanel::SignUp))
    };

    let close_all = {
        let panel = panel.clone();
        Callback::from(move |_| panel.set(OpenPanel::None))
    };

    let on_login = {
        let auth_ctx = auth_ctx.clone();
        let panel = panel.clone();
        Callback::from(move |auth| {
            auth_ctx.login.emit(auth);
            panel.set(OpenPanel::None);
        })
    };

    let on_logout = {
        let auth_ctx = auth_ctx.clone();
        let panel = panel.clone();
        Callback::from(move |_| {
            if let Some(auth) = &auth_ctx.auth {
                let jwt = auth.jwt.clone();
                spawn_local(async move {
                    let header = format!("bearer {}", jwt);
                    let _ = gloo_net::http::Request::post("/api/auth/logout")
                        .header("Authorization", header.as_str())
                        .send()
                        .await;
                });
            }
            auth_ctx.logout.emit(());
            panel.set(OpenPanel::None);
        })
    };

    let navigator = use_navigator().unwrap();

    let go_account = {
        let navigator = navigator.clone();
        let close_all = close_all.clone();
        Callback::from(move |_| {
            navigator.push(&AppRoutes::Account);
            close_all.emit(());
        })
    };

    let menu_dropdown = if *panel == OpenPanel::Menu {
        Some(match &auth_ctx.auth {
            Some(auth) => html! {
                <div class="absolute right-0 top-full z-50 mt-2 min-w-[11rem] rounded-xl border border-border bg-surface-elevated py-1 shadow-lg">
                    <p class="px-3 py-2 text-xs text-muted truncate">
                        { format!("Signed in as {}", auth.user.username) }
                    </p>
                    <button
                        type="button"
                        class="block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
                        onclick={go_account}
                    >
                        { "Account" }
                    </button>
                    <button
                        type="button"
                        class="block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
                        onclick={{
                            let navigator = navigator.clone();
                            let close_all = close_all.clone();
                            Callback::from(move |_| {
                                navigator.push(&AppRoutes::Profile);
                                close_all.emit(());
                            })
                        }}
                    >
                        { "Profile" }
                    </button>
                    <button
                        type="button"
                        class="block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
                        onclick={on_logout}
                    >
                        { "Log out" }
                    </button>
                </div>
            },
            None => html! {
                <div class="absolute right-0 top-full z-50 mt-2 min-w-[11rem] rounded-xl border border-border bg-surface-elevated py-1 shadow-lg">
                    <button
                        type="button"
                        class="block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
                        onclick={open_sign_in}
                    >
                        { "Sign in" }
                    </button>
                    <button
                        type="button"
                        class="block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
                        onclick={open_sign_up}
                    >
                        { "Sign up" }
                    </button>
                </div>
            },
        })
    } else {
        None
    };

    html! {
        <div class="relative">
            <button
                type="button"
                class="flex h-10 w-10 items-center justify-center rounded-full border border-border bg-surface-elevated text-foreground transition hover:border-accent hover:text-accent"
                aria-label="Account menu"
                aria-expanded={(*panel == OpenPanel::Menu).to_string()}
                onclick={toggle_menu}
            >
                <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                    <circle cx="12" cy="8" r="4" />
                    <path stroke-linecap="round" d="M5 20c0-4 3.5-6 7-6s7 2 7 6" />
                </svg>
            </button>

            { menu_dropdown }

            if *panel == OpenPanel::SignIn {
                <AuthModal
                    kind={AuthModalKind::SignIn}
                    on_close={close_all.clone()}
                    on_login={on_login}
                />
            }
            if *panel == OpenPanel::SignUp {
                <AuthModal
                    kind={AuthModalKind::SignUp}
                    on_close={close_all.clone()}
                    on_login={Callback::noop()}
                />
            }
        </div>
    }
}
