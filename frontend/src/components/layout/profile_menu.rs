use super::theme_toggle::ThemeToggle;
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

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or(false)]
    pub inline: bool,
}

#[function_component(ProfileMenu)]
pub fn profile_menu(props: &Props) -> Html {
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

    let close_menu = {
        let panel = panel.clone();
        Callback::from(move |_| {
            if *panel == OpenPanel::Menu {
                panel.set(OpenPanel::None);
            }
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

    let item_class = if props.inline {
        "menu-action"
    } else {
        "block w-full px-3 py-2 text-left text-sm text-foreground transition hover:bg-surface"
    };
    let menu_open = *panel == OpenPanel::Menu;
    let show_actions = menu_open;
    let actions = if show_actions {
        Some(match &auth_ctx.auth {
            Some(auth) => html! {
                <>
                    <p class="menu-user">
                        { format!("Signed in as {}", auth.user.username) }
                    </p>
                    <button type="button" class={item_class} onclick={go_account}>
                        { "Account" }
                    </button>
                    if auth.user.role == "admin" {
                        <button
                            type="button"
                            class={item_class}
                            onclick={{
                                let navigator = navigator.clone();
                                let close_all = close_all.clone();
                                Callback::from(move |_| {
                                    navigator.push(&AppRoutes::Admin);
                                    close_all.emit(());
                                })
                            }}
                        >
                            { "Admin" }
                        </button>
                    }
                    <button
                        type="button"
                        class={item_class}
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
                    <button type="button" class={item_class} onclick={on_logout}>
                        { "Log out" }
                    </button>
                </>
            },
            None => html! {
                <>
                    <button type="button" class={item_class} onclick={open_sign_in}>
                        { "Sign in" }
                    </button>
                    <button type="button" class={item_class} onclick={open_sign_up}>
                        { "Sign up" }
                    </button>
                </>
            },
        })
    } else {
        None
    };

    let modals = html! {
        <>
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
        </>
    };

    if props.inline {
        return html! {
            <div class="account-nav">
                <button
                    type="button"
                    class="account-nav-trigger"
                    aria-label="Account"
                    aria-expanded={menu_open.to_string()}
                    aria-haspopup="true"
                    aria-controls="account-nav-menu"
                    onclick={toggle_menu}
                >
                    { "Account" }
                    <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.06 1.06l-4.24 4.25a.75.75 0 01-1.06 0L5.21 8.27a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
                    </svg>
                </button>
                if menu_open {
                    <div id="account-nav-menu" class="account-nav-menu">
                        <ThemeToggle inline={true} />
                        { actions }
                    </div>
                }
                { modals }
            </div>
        };
    }

    html! {
        <div
            class="relative"
            onmouseleave={close_menu}
        >
            <button
                type="button"
                class="icon-btn"
                aria-label="Account menu"
                aria-expanded={(*panel == OpenPanel::Menu).to_string()}
                onclick={toggle_menu}
            >
                <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                    <circle cx="12" cy="8" r="4" />
                    <path stroke-linecap="round" d="M5 20c0-4 3.5-6 7-6s7 2 7 6" />
                </svg>
            </button>

            if *panel == OpenPanel::Menu {
                <div class="menu">
                    { actions }
                </div>
            }
            { modals }
        </div>
    }
}
