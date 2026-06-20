use crate::{
    components::auth::{AuthControls, Login, Signup},
    context::AuthContext,
    routes::AppRoutes,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
}

#[function_component(Header)]
pub fn header(props: &Props) -> Html {
    let auth_ctx = use_context::<AuthContext>().expect("AuthProvider required");
    html! {
        <div class={classes!("site-header", props.class.clone())}>
            <div class="menu">
                <Link<AppRoutes> to={AppRoutes::Home} classes="nav-link">
                    { "Dashboard" }
                </Link<AppRoutes>>
                <Link<AppRoutes> to={AppRoutes::Shoots} classes="nav-link">
                    { "Shoots" }
                </Link<AppRoutes>>
                <Link<AppRoutes> to={AppRoutes::Galleries} classes="nav-link">
                    { "Galleries" }
                </Link<AppRoutes>>
                <Link<AppRoutes> to={AppRoutes::Profile} classes="nav-link">
                    { "Profile" }
                </Link<AppRoutes>>
            </div>
            <div class="divider" />
            <div class="auth-info">
                { render_auth(&auth_ctx) }
            </div>
        </div>
    }
}

fn render_auth(auth_ctx: &AuthContext) -> Html {
    match &auth_ctx.auth {
        Some(auth) => html! {
            <AuthControls auth={auth.clone()} on_logout={auth_ctx.logout.clone()} />
        },
        None => html! {
            <>
                <Login class="login" on_login={auth_ctx.login.clone()} />
                <Signup class="signup" />
            </>
        },
    }
}
