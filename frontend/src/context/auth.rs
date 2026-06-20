use crate::model::Auth;
use gloo_storage::{Storage, SessionStorage};
use yew::prelude::*;

const AUTH_KEY: &str = "Auth";

#[derive(Clone, PartialEq)]
pub struct AuthContext {
    pub auth: Option<Auth>,
    pub login: Callback<Auth>,
    pub logout: Callback<()>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub children: Children,
}

pub enum Msg {
    Login(Auth),
    Logout,
}

pub struct AuthProvider {
    auth: Option<Auth>,
}

impl Component for AuthProvider {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            auth: restore_auth(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Login(auth) => {
                store_auth(&auth);
                self.auth = Some(auth);
            }
            Msg::Logout => {
                SessionStorage::delete(AUTH_KEY);
                self.auth = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let login = ctx.link().callback(Msg::Login);
        let logout = ctx.link().callback(|_| Msg::Logout);
        let auth_ctx = AuthContext {
            auth: self.auth.clone(),
            login,
            logout,
        };
        html! {
            <ContextProvider<AuthContext> context={auth_ctx}>
                { for ctx.props().children.iter() }
            </ContextProvider<AuthContext>>
        }
    }
}

fn restore_auth() -> Option<Auth> {
    SessionStorage::get(AUTH_KEY).ok()
}

fn store_auth(auth: &Auth) {
    if let Ok(json) = serde_json::to_string(auth) {
        SessionStorage::set(AUTH_KEY, json).ok();
    }
}
