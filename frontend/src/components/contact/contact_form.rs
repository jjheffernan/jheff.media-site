use crate::model::{ContactSubmission, ServerResponse};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or(false)]
    pub booking: bool,
}

#[derive(Clone, PartialEq)]
enum SubmitState {
    Idle,
    Sending,
    Success(String),
    Error(String),
}

#[function_component(ContactForm)]
pub fn contact_form(props: &Props) -> Html {
    let name = use_state(String::new);
    let email = use_state(String::new);
    let subject = use_state(String::new);
    let message = use_state(String::new);
    let website = use_state(String::new);
    let submit_state = use_state(|| SubmitState::Idle);
    let booking = props.booking;

    let on_name = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            name.set(input.value());
        })
    };
    let on_email = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            email.set(input.value());
        })
    };
    let on_subject = {
        let subject = subject.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            subject.set(input.value());
        })
    };
    let on_message = {
        let message = message.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>();
            message.set(input.value());
        })
    };

    let on_website = {
        let website = website.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            website.set(input.value());
        })
    };

    let on_submit = {
        let name = name.clone();
        let email = email.clone();
        let subject = subject.clone();
        let message = message.clone();
        let website = website.clone();
        let submit_state = submit_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if (*name).trim().is_empty()
                || (*email).trim().is_empty()
                || (*message).trim().is_empty()
            {
                submit_state.set(SubmitState::Error(
                    "Name, email, and message are required.".into(),
                ));
                return;
            }
            submit_state.set(SubmitState::Sending);
            let payload = ContactSubmission {
                name: (*name).clone(),
                email: (*email).clone(),
                subject: (*subject).clone(),
                message: (*message).clone(),
                website: (*website).clone(),
                kind: if booking {
                    "booking".into()
                } else {
                    "contact".into()
                },
            };
            let submit_state = submit_state.clone();
            spawn_local(async move {
                let body = serde_json::to_string(&payload).unwrap_or_default();
                let result = match Request::post("/api/contact")
                    .header("Content-Type", "application/json")
                    .body(body)
                {
                    Ok(req) => req.send().await,
                    Err(err) => Err(err),
                };
                match result {
                    Ok(resp) if resp.ok() => {
                        if booking {
                            submit_state.set(SubmitState::Success(
                                "Request received. We'll follow up.".into(),
                            ));
                            return;
                        }
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            submit_state.set(SubmitState::Success(body.data));
                        } else {
                            submit_state.set(SubmitState::Success(
                                "Message sent. We'll get back to you soon.".into(),
                            ));
                        }
                    }
                    Ok(resp) => {
                        if let Ok(body) = resp.json::<ServerResponse<String>>().await {
                            let text = if body.data.is_empty() {
                                body.message
                            } else {
                                body.data
                            };
                            submit_state.set(SubmitState::Error(text));
                        } else {
                            submit_state.set(SubmitState::Error("Could not send message.".into()));
                        }
                    }
                    Err(err) => submit_state.set(SubmitState::Error(err.to_string())),
                }
            });
        })
    };

    let disabled = *submit_state == SubmitState::Sending;

    let feedback = match &*submit_state {
        SubmitState::Success(msg) => html! {
            <p class="muted">{ msg.clone() }</p>
        },
        SubmitState::Error(msg) => html! {
            <p class="muted">{ msg.clone() }</p>
        },
        _ => html! {},
    };

    let subject_label = if booking { "What do you want" } else { "Subject" };

    html! {
        <form class="relative space-y-4" onsubmit={Callback::from(|e: SubmitEvent| e.prevent_default())}>
            <div class="absolute -left-[9999px] h-px w-px overflow-hidden" aria-hidden="true">
                <label>
                    { "Website" }
                    <input
                        type="text"
                        tabindex="-1"
                        autocomplete="off"
                        value={(*website).clone()}
                        oninput={on_website}
                    />
                </label>
            </div>
            <div class="grid gap-4 sm:grid-cols-2">
                <label class="label">
                    { "Name" }
                    <input
                        type="text"
                        class="field"
                        value={(*name).clone()}
                        oninput={on_name}
                        disabled={disabled}
                    />
                </label>
                <label class="label">
                    { "Email" }
                    <input
                        type="email"
                        class="field"
                        value={(*email).clone()}
                        oninput={on_email}
                        disabled={disabled}
                    />
                </label>
            </div>
            if !booking {
                <label class="label">
                    { subject_label }
                    <input
                        type="text"
                        class="field"
                        value={(*subject).clone()}
                        oninput={on_subject.clone()}
                        disabled={disabled}
                        placeholder="General question"
                    />
                </label>
            }
            <label class="label">
                { "Message" }
                <textarea
                    class="field field-area"
                    value={(*message).clone()}
                    oninput={on_message}
                    disabled={disabled}
                />
            </label>
            if booking {
                <label class="label">
                    { subject_label }
                    <input
                        type="text"
                        class="field"
                        value={(*subject).clone()}
                        oninput={on_subject}
                        disabled={disabled}
                    />
                </label>
            }
            { feedback }
            <button type="button" class="btn btn-primary" disabled={disabled} onclick={on_submit}>
                if disabled {
                    { "Sending…" }
                } else if booking {
                    { "Send request" }
                } else {
                    { "Send message" }
                }
            </button>
        </form>
    }
}
