use crate::model::{Auth as AuthModel, Login as Model, ServerResponse, User as UserModel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct Login {
    id: String,
    form_data: Model,
    username_error: Option<String>,
    password_error: Option<String>,
    submitting: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
    pub on_login: Callback<AuthModel>,
}

pub enum LoginFailure {
    BadUsername,
    BadPassword,
    Other,
}

pub enum Msg {
    UpdateEmailOrUsername(String),
    UpdatePassword(String),
    Request,
    Response(Result<String, LoginFailure>),
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            form_data: Model::default(),
            username_error: None,
            password_error: None,
            submitting: false,
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateEmailOrUsername(email_or_username) => {
                self.username_error = if email_or_username.is_empty() {
                    Some("CANNOT_BE_EMPTY".to_string())
                } else {
                    None
                };
                self.form_data.email_or_username = email_or_username;
            }
            Msg::UpdatePassword(password) => {
                self.password_error = if password.is_empty() {
                    Some("CANNOT_BE_EMPTY".to_string())
                } else {
                    None
                };
                self.form_data.password = password;
            }
            Msg::Request => {
                self.submitting = true;
                let body = serde_json::to_string(&self.form_data).unwrap_or_default();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match gloo_net::http::Request::post("/api/auth/login")
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        Ok(req) => match req.send().await {
                            Ok(resp) if resp.ok() => {
                                Ok(resp.text().await.unwrap_or_default())
                            }
                            Ok(resp) if resp.status() == 404 => Err(LoginFailure::BadUsername),
                            Ok(resp) if resp.status() == 400 => Err(LoginFailure::BadPassword),
                            Ok(_) | Err(_) => Err(LoginFailure::Other),
                        },
                        Err(_) => Err(LoginFailure::Other),
                    };
                    link.send_message(Msg::Response(result));
                });
            }
            Msg::Response(res) => {
                self.submitting = false;
                match res {
                    Ok(data) => {
                        let login_info: ServerResponse<LoginSuccess> =
                            serde_json::from_str(data.as_str()).unwrap();
                        ctx.props().on_login.emit(AuthModel {
                            jwt: login_info.data.token,
                            user: login_info.data.user,
                        });
                    }
                    Err(fail) => match fail {
                        LoginFailure::BadUsername => {
                            self.username_error = Some("BAD_USERNAME".to_string());
                        }
                        LoginFailure::BadPassword => {
                            self.password_error = Some("BAD_PASSWORD".to_string());
                        }
                        LoginFailure::Other => {}
                    },
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
        let oninput_email_or_username = ctx.link().callback(|e: InputEvent| {
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value();
            Msg::UpdateEmailOrUsername(value)
        });
        let oninput_password = ctx.link().callback(|e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            Msg::UpdatePassword(value)
        });
        let username_class = if self.username_error.is_some() {
            "invalid"
        } else {
            ""
        };
        let password_class = if self.password_error.is_some() {
            "invalid"
        } else {
            ""
        };
        html! {
            <div class={classes!("login", ctx.props().class.clone())}>
                <form id={self.id.clone()} class="login" onsubmit={onsubmit}>
                    <label for={format!("{}-email-or-userame", self.id)}>
                        {"Email or Username"}
                    </label>
                    <input
                        id={format!("{}-email-or-userame", self.id)}
                        type="text"
                        class={username_class}
                        value={self.form_data.email_or_username.clone()}
                        oninput={oninput_email_or_username}
                        placeholder="Email or Username"
                        spellcheck="false" />
                    <label for={format!("{}-password", self.id)}>
                        {"Password"}
                    </label>
                    <input
                        id={format!("{}-password", self.id)}
                        type="password"
                        class={password_class}
                        value={self.form_data.password.clone()}
                        oninput={oninput_password}
                        placeholder="Password"
                        spellcheck="false" />
                    <button type="submit" disabled={self.submitting}>
                        { "Log in" }
                    </button>
                </form>
            </div>
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LoginSuccess {
    token: String,
    token_type: String,
    user: UserModel,
}
