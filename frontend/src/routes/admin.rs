use crate::components::{
    admin::{AnalyticsAdmin, ContactAdmin, ContentAdmin, LinksAdmin, SetupAdmin},
    auth::{AuthModal, AuthModalKind},
};
use crate::context::AuthContext;
use crate::routes::AppRoutes;
use crate::services::auth_header;
use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Setup,
    Analytics,
    Links,
    Inbox,
    Content,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetupProbe {
    needs_setup: bool,
}

#[function_component(Admin)]
pub fn admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let navigator = use_navigator().expect("Router required");
    let panel = use_state(|| Panel::Analytics);
    let setup_hint = use_state(|| false);

    {
        let panel = panel.clone();
        let setup_hint = setup_hint.clone();
        let jwt = auth_ctx
            .auth
            .as_ref()
            .filter(|auth| auth.user.role == "admin")
            .map(|auth| auth.jwt.clone());
        use_effect_with(jwt, move |jwt| {
            if let Some(jwt) = jwt.clone() {
                let panel = panel.clone();
                let setup_hint = setup_hint.clone();
                spawn_local(async move {
                    let auth_val = auth_header(&jwt);
                    if let Ok(resp) = Request::get("/api/admin/setup")
                        .header("Authorization", auth_val.as_str())
                        .send()
                        .await
                    {
                        if resp.ok() {
                            if let Ok(body) = resp.json::<SetupProbe>().await {
                                if body.needs_setup {
                                    setup_hint.set(true);
                                    panel.set(Panel::Setup);
                                }
                            }
                        }
                    }
                });
            }
            || ()
        });
    }

    match &auth_ctx.auth {
        None => {
            let on_login = auth_ctx.login.clone();
            let on_close = {
                let navigator = navigator.clone();
                Callback::from(move |_| {
                    navigator.replace(&AppRoutes::Home);
                })
            };
            html! {
                <AuthModal
                    kind={AuthModalKind::SignIn}
                    {on_close}
                    {on_login}
                />
            }
        }
        Some(auth) if auth.user.role != "admin" => html! {
            <Redirect<AppRoutes> to={AppRoutes::NotFound} />
        },
        Some(auth) => {
            let current = *panel;
            let who = auth.user.username.clone();
            let pick = |next: Panel| {
                let panel = panel.clone();
                Callback::from(move |_| panel.set(next))
            };
            html! {
                <div class="admin-shell">
                    <aside class="admin-rail">
                        <p class="kicker">{ "jheff.media" }</p>
                        <nav aria-label="Admin">
                            <button type="button" class="admin-link" aria-current={(current == Panel::Setup).then_some("page")} onclick={pick(Panel::Setup)}>{ "Setup" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Analytics).then_some("page")} onclick={pick(Panel::Analytics)}>{ "Analytics" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Links).then_some("page")} onclick={pick(Panel::Links)}>{ "Links" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Inbox).then_some("page")} onclick={pick(Panel::Inbox)}>{ "Inbox" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Content).then_some("page")} onclick={pick(Panel::Content)}>{ "Content" }</button>
                        </nav>
                        <p class="muted admin-who">{ who }</p>
                    </aside>
                    <div class="min-w-0 space-y-3">
                        if *setup_hint && current != Panel::Setup {
                            <p class="admin-warn" role="status">
                                { "Site setup has not been saved yet. Open Setup to choose a feed provider." }
                            </p>
                        }
                        { match current {
                            Panel::Setup => html! { <SetupAdmin /> },
                            Panel::Analytics => html! { <AnalyticsAdmin /> },
                            Panel::Links => html! { <LinksAdmin /> },
                            Panel::Inbox => html! { <ContactAdmin /> },
                            Panel::Content => html! { <ContentAdmin /> },
                        } }
                    </div>
                </div>
            }
        }
    }
}
