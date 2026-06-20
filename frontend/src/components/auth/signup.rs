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
    response_data: Option<Result<String, String>>,
    submitting: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
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
            response_data: None,
            submitting: false,
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateEmail(email) => self.form_data.email = email,
            Msg::UpdateUsername(username) => self.form_data.username = username,
            Msg::UpdatePassword(password) => self.form_data.password = password,
            Msg::UpdatePasswordRepeat(password) => self.form_data.password_repeat = password,
            Msg::Request => {
                self.submitting = true;
                let request_data = UserDTO::from(self.form_data.clone());
                let body = serde_json::to_string(&request_data).unwrap_or_default();
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match gloo_net::http::Request::post("/api/auth/signup")
                        .header("Content-Type", "application/json")
                        .body(body)
                    {
                        Ok(req) => match req.send().await {
                            Ok(resp) if resp.ok() => Ok("User has been registered".to_string()),
                            Ok(_) => Err("User could not be registered".to_string()),
                            Err(_) => Err("User could not be registered".to_string()),
                        },
                        Err(_) => Err("User could not be registered".to_string()),
                    };
                    link.send_message(Msg::Response(result));
                });
            }
            Msg::Response(res) => {
                self.response_data = Some(res);
                self.submitting = false;
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
        html! {
            <div class={classes!("signup", ctx.props().class.clone())}>
                <form id={self.id.clone()} class="signup" onsubmit={onsubmit}>
                    <label for={format!("{}-email", self.id)}>{"Email"}</label>
                    <input
                        id={format!("{}-email", self.id)}
                        type="email"
                        value={self.form_data.email.clone()}
                        oninput={oninput_email}
                        placeholder="Email"
                        spellcheck="false" />
                    <label for={format!("{}-username", self.id)}>{"Username"}</label>
                    <input
                        id={format!("{}-username", self.id)}
                        type="text"
                        value={self.form_data.username.clone()}
                        oninput={oninput_username}
                        placeholder="Username"
                        spellcheck="false" />
                    <label for={format!("{}-password", self.id)}>{"Password"}</label>
                    <input
                        id={format!("{}-password", self.id)}
                        type="password"
                        value={self.form_data.password.clone()}
                        oninput={oninput_password}
                        placeholder="Password"
                        spellcheck="false" />
                    <label for={format!("{}-password-repeat", self.id)}>
                        {"Repeat Password"}
                    </label>
                    <input
                        id={format!("{}-password-repeat", self.id)}
                        type="password"
                        value={self.form_data.password_repeat.clone()}
                        oninput={oninput_password_repeat}
                        placeholder="Repeat Password"
                        spellcheck="false" />
                    <button type="submit" disabled={self.submitting}>
                        { "Sign up" }
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
