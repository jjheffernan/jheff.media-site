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
    totp_error: Option<String>,
    form_error: Option<String>,
    show_totp: bool,
    submitting: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub embedded: bool,
    pub on_login: Callback<AuthModel>,
}

pub enum LoginFailure {
    BadUsername,
    BadPassword,
    TotpRequired,
    TotpInvalid,
    RateLimited,
    Other,
}

pub enum Msg {
    UpdateEmailOrUsername(String),
    UpdatePassword(String),
    UpdateTotp(String),
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
            totp_error: None,
            form_error: None,
            show_totp: false,
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
            Msg::UpdateTotp(code) => {
                self.totp_error = None;
                self.form_data.totp_code = Some(code);
            }
            Msg::Request => {
                self.submitting = true;
                self.form_error = None;
                if !self.show_totp {
                    self.form_data.totp_code = None;
                } else if self.form_data.totp_code.is_none() {
                    self.form_data.totp_code = Some(String::new());
                }
                let body = serde_json::to_string(&self.form_data).unwrap_or_default();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match gloo_net::http::Request::post("/api/auth/login")
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        Ok(req) => match req.send().await {
                            Ok(resp) => {
                                let status = resp.status();
                                let text = resp.text().await.unwrap_or_default();
                                if resp.ok() {
                                    Ok(text)
                                } else if status == 401 && text.contains("TOTP_REQUIRED") {
                                    Err(LoginFailure::TotpRequired)
                                } else if status == 400 && text.contains("TOTP_INVALID") {
                                    Err(LoginFailure::TotpInvalid)
                                } else if status == 429 {
                                    Err(LoginFailure::RateLimited)
                                } else if status == 404 {
                                    Err(LoginFailure::BadUsername)
                                } else if status == 400 {
                                    Err(LoginFailure::BadPassword)
                                } else {
                                    Err(LoginFailure::Other)
                                }
                            }
                            Err(_) => Err(LoginFailure::Other),
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
                        LoginFailure::TotpRequired => {
                            self.show_totp = true;
                            self.totp_error = None;
                            if self.form_data.totp_code.is_none() {
                                self.form_data.totp_code = Some(String::new());
                            }
                        }
                        LoginFailure::TotpInvalid => {
                            self.show_totp = true;
                            self.totp_error = Some("TOTP_INVALID".to_string());
                        }
                        LoginFailure::RateLimited => {
                            self.form_error =
                                Some("Too many attempts. Try again later.".to_string());
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
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            Msg::UpdateEmailOrUsername(value)
        });
        let oninput_password = ctx.link().callback(|e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            Msg::UpdatePassword(value)
        });
        let oninput_totp = ctx.link().callback(|e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            Msg::UpdateTotp(value)
        });
        let username_class = if self.username_error.is_some() {
            "field field-invalid"
        } else {
            "field"
        };
        let password_class = if self.password_error.is_some() {
            "field field-invalid"
        } else {
            "field"
        };
        let totp_class = if self.totp_error.is_some() {
            "field field-invalid"
        } else {
            "field"
        };
        let wrapper_class = if ctx.props().embedded {
            format!("p-0 {}", ctx.props().class)
        } else {
            format!("card {}", ctx.props().class)
        };
        html! {
            <div class={wrapper_class}>
                <p class="mb-3 text-xs font-semibold uppercase tracking-wide text-muted">
                    { "Log in" }
                </p>
                <form id={self.id.clone()} class="space-y-3" onsubmit={onsubmit}>
                    <label class="label-xs" for={format!("{}-email-or-userame", self.id)}>
                        { "Email or Username" }
                    </label>
                    <input
                        id={format!("{}-email-or-userame", self.id)}
                        type="text"
                        class={username_class}
                        value={self.form_data.email_or_username.clone()}
                        oninput={oninput_email_or_username}
                        placeholder="Email or Username"
                        spellcheck="false" />
                    <label class="label-xs" for={format!("{}-password", self.id)}>
                        { "Password" }
                    </label>
                    <input
                        id={format!("{}-password", self.id)}
                        type="password"
                        class={password_class}
                        value={self.form_data.password.clone()}
                        oninput={oninput_password}
                        placeholder="Password"
                        spellcheck="false" />
                    if self.show_totp {
                        <p class="text-xs text-muted">
                            { "Enter the code from your authenticator app." }
                        </p>
                        <input
                            id={format!("{}-totp", self.id)}
                            type="text"
                            inputmode="numeric"
                            autocomplete="one-time-code"
                            maxlength="6"
                            class={totp_class}
                            value={self.form_data.totp_code.clone().unwrap_or_default()}
                            oninput={oninput_totp}
                            spellcheck="false" />
                        if let Some(err) = &self.totp_error {
                            <p class="text-xs text-red-600 dark:text-red-300">{ err.clone() }</p>
                        }
                    }
                    if let Some(err) = &self.form_error {
                        <p class="text-xs text-red-600 dark:text-red-300">{ err.clone() }</p>
                    }
                    <button
                        type="submit"
                        disabled={self.submitting}
                        class="w-full btn btn-primary w-full"
                    >
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
