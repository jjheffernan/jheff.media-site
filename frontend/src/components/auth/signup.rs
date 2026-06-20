use crate::model::Signup as Model;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDTO {
    pub email: String,
    pub username: String,
    pub password: String,
    pub login_session: String,
}

pub struct Signup {
    id: String,
    form_data: Model,
    error: Option<String>,
    success: Option<String>,
    submitting: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub embedded: bool,
    #[prop_or_default]
    pub on_success: Option<Callback<()>>,
}

pub enum Msg {
    UpdateEmail(String),
    UpdateUsername(String),
    UpdatePassword(String),
    UpdatePasswordRepeat(String),
    Request,
    Response(Result<String, String>),
}

impl Component for Signup {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            form_data: Model::default(),
            error: None,
            success: None,
            submitting: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateEmail(email) => {
                self.form_data.email = email;
                self.error = None;
            }
            Msg::UpdateUsername(username) => {
                self.form_data.username = username;
                self.error = None;
            }
            Msg::UpdatePassword(password) => {
                self.form_data.password = password;
                self.error = None;
            }
            Msg::UpdatePasswordRepeat(password) => {
                self.form_data.password_repeat = password;
                self.error = None;
            }
            Msg::Request => {
                if self.form_data.email.is_empty()
                    || self.form_data.username.is_empty()
                    || self.form_data.password.is_empty()
                {
                    self.error = Some("All fields are required.".into());
                    return true;
                }
                if self.form_data.password != self.form_data.password_repeat {
                    self.error = Some("Passwords do not match.".into());
                    return true;
                }
                self.submitting = true;
                self.error = None;
                self.success = None;
                let request_data = UserDTO::from(self.form_data.clone());
                let body = serde_json::to_string(&request_data).unwrap_or_default();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match gloo_net::http::Request::post("/api/auth/signup")
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        Ok(req) => match req.send().await {
                            Ok(resp) if resp.ok() => {
                                Ok("Account created. You can sign in now.".to_string())
                            }
                            Ok(resp) => {
                                let msg = resp
                                    .text()
                                    .await
                                    .unwrap_or_else(|_| "Registration failed.".into());
                                Err(msg)
                            }
                            Err(_) => Err("Network error. Try again.".into()),
                        },
                        Err(_) => Err("Could not send request.".into()),
                    };
                    link.send_message(Msg::Response(result));
                });
            }
            Msg::Response(res) => {
                self.submitting = false;
                match res {
                    Ok(message) => {
                        self.success = Some(message);
                        self.error = None;
                        if let Some(on_success) = &ctx.props().on_success {
                            on_success.emit(());
                        }
                    }
                    Err(message) => {
                        self.error = Some(message);
                        self.success = None;
                    }
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.link().callback(|e: SubmitEvent| {
            e.prevent_default();
            Msg::Request
        });
        let oninput_username = ctx.link().callback(|e: InputEvent| {
            Msg::UpdateUsername(e.target_unchecked_into::<HtmlInputElement>().value())
        });
        let oninput_email = ctx.link().callback(|e: InputEvent| {
            Msg::UpdateEmail(e.target_unchecked_into::<HtmlInputElement>().value())
        });
        let oninput_password = ctx.link().callback(|e: InputEvent| {
            Msg::UpdatePassword(e.target_unchecked_into::<HtmlInputElement>().value())
        });
        let oninput_password_repeat = ctx.link().callback(|e: InputEvent| {
            Msg::UpdatePasswordRepeat(e.target_unchecked_into::<HtmlInputElement>().value())
        });

        let wrapper_class = if ctx.props().embedded {
            format!("{}", ctx.props().class)
        } else {
            format!(
                "rounded-xl border border-border bg-surface-elevated p-4 {}",
                ctx.props().class
            )
        };

        html! {
            <div class={wrapper_class}>
                <p class="mb-3 text-xs font-semibold uppercase tracking-wide text-muted">
                    { "Sign up" }
                </p>
                if let Some(err) = &self.error {
                    <p class="mb-3 rounded-lg border border-red-500/40 bg-red-500/10 px-3 py-2 text-sm text-red-600 dark:text-red-300">
                        { err.clone() }
                    </p>
                }
                if let Some(msg) = &self.success {
                    <p class="mb-3 rounded-lg border border-accent/40 bg-accent/10 px-3 py-2 text-sm text-accent">
                        { msg.clone() }
                    </p>
                }
                <form id={self.id.clone()} class="space-y-3" onsubmit={onsubmit}>
                    <label class="block text-xs font-medium text-muted" for={format!("{}-email", self.id)}>{ "Email" }</label>
                    <input
                        id={format!("{}-email", self.id)}
                        type="email"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground placeholder:text-muted focus:border-accent focus:outline-none focus:ring-2 focus:ring-accent/30"
                        value={self.form_data.email.clone()}
                        oninput={oninput_email}
                        placeholder="Email"
                        spellcheck="false" />
                    <label class="block text-xs font-medium text-muted" for={format!("{}-username", self.id)}>{ "Username" }</label>
                    <input
                        id={format!("{}-username", self.id)}
                        type="text"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground placeholder:text-muted focus:border-accent focus:outline-none focus:ring-2 focus:ring-accent/30"
                        value={self.form_data.username.clone()}
                        oninput={oninput_username}
                        placeholder="Username"
                        spellcheck="false" />
                    <label class="block text-xs font-medium text-muted" for={format!("{}-password", self.id)}>{ "Password" }</label>
                    <input
                        id={format!("{}-password", self.id)}
                        type="password"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground placeholder:text-muted focus:border-accent focus:outline-none focus:ring-2 focus:ring-accent/30"
                        value={self.form_data.password.clone()}
                        oninput={oninput_password}
                        placeholder="Password"
                        spellcheck="false" />
                    <label class="block text-xs font-medium text-muted" for={format!("{}-password-repeat", self.id)}>
                        { "Repeat Password" }
                    </label>
                    <input
                        id={format!("{}-password-repeat", self.id)}
                        type="password"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground placeholder:text-muted focus:border-accent focus:outline-none focus:ring-2 focus:ring-accent/30"
                        value={self.form_data.password_repeat.clone()}
                        oninput={oninput_password_repeat}
                        placeholder="Repeat Password"
                        spellcheck="false" />
                    <button
                        type="submit"
                        disabled={self.submitting}
                        class="w-full rounded-lg bg-accent px-4 py-2 text-sm font-semibold text-zinc-950 transition hover:bg-accent-hover disabled:opacity-50"
                    >
                        { if self.submitting { "Creating account…" } else { "Sign up" } }
                    </button>
                </form>
            </div>
        }
    }
}

impl From<Model> for UserDTO {
    fn from(model: Model) -> Self {
        Self {
            email: model.email,
            username: model.username,
            password: model.password,
            login_session: String::new(),
        }
    }
}
