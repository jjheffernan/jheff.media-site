use crate::model::Auth;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

pub struct Controls {
    submitting: bool,
}

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct Props {
    pub auth: Auth,
    pub on_logout: Callback<()>,
}

pub enum Msg {
    Logout,
    Response,
}

impl Component for Controls {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { submitting: false }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Logout => {
                self.submitting = true;
                let jwt = ctx.props().auth.jwt.clone();
                let link = ctx.link().clone();
                let auth_header = format!("bearer {}", jwt);
                spawn_local(async move {
                    let _ = gloo_net::http::Request::post("/api/auth/logout")
                        .header("Authorization", auth_header.as_str())
                        .send()
                        .await;
                    link.send_message(Msg::Response);
                });
            }
            Msg::Response => {
                self.submitting = false;
                ctx.props().on_logout.emit(());
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick_logout = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Logout
        });
        html! {
            <div class="rounded-xl border border-zinc-800 bg-zinc-900/80 px-4 py-3 text-sm">
                <span class="font-medium text-zinc-200">
                    { format!("Hi, {}!", ctx.props().auth.user.username) }
                </span>
                <button
                    type="button"
                    onclick={onclick_logout}
                    disabled={self.submitting}
                    class="mt-2 text-cyan-400 transition hover:text-cyan-300 disabled:opacity-50"
                >
                    { "Logout" }
                </button>
            </div>
        }
    }
}
