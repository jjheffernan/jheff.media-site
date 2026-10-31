use crate::components::{
    admin::{AnalyticsAdmin, ContactAdmin, ContentAdmin, LinksAdmin},
    auth::{AuthModal, AuthModalKind},
};
use crate::context::AuthContext;
use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Analytics,
    Links,
    Inbox,
    Content,
}

#[function_component(Admin)]
pub fn admin() -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    let navigator = use_navigator().expect("Router required");
    let panel = use_state(|| Panel::Analytics);

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
                            <button type="button" class="admin-link" aria-current={(current == Panel::Analytics).then_some("page")} onclick={pick(Panel::Analytics)}>{ "Analytics" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Links).then_some("page")} onclick={pick(Panel::Links)}>{ "Links" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Inbox).then_some("page")} onclick={pick(Panel::Inbox)}>{ "Inbox" }</button>
                            <button type="button" class="admin-link" aria-current={(current == Panel::Content).then_some("page")} onclick={pick(Panel::Content)}>{ "Content" }</button>
                        </nav>
                        <p class="muted admin-who">{ who }</p>
                    </aside>
                    <div class="min-w-0">
                        { match current {
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
