use crate::{
    components::ui::{Button, ButtonVariant, Text, TextTone},
    model::{ContactSubmission, ServerResponse},
};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum SubmitState {
    Idle,
    Sending,
    Success(String),
    Error(String),
}

#[function_component(ContactForm)]
pub fn contact_form() -> Html {
    let name = use_state(String::new);
    let email = use_state(String::new);
    let subject = use_state(String::new);
    let message = use_state(String::new);
    let submit_state = use_state(|| SubmitState::Idle);

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

    let on_submit = {
        let name = name.clone();
        let email = email.clone();
        let subject = subject.clone();
        let message = message.clone();
        let submit_state = submit_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if (*name).trim().is_empty() || (*email).trim().is_empty() || (*message).trim().is_empty()
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
                            submit_state.set(SubmitState::Error(body.data));
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
            <p class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-700 dark:text-emerald-300">
                { msg.clone() }
            </p>
        },
        SubmitState::Error(msg) => html! {
            <p class="rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-700 dark:text-red-300">
                { msg.clone() }
            </p>
        },
        _ => html! {},
    };

    html! {
        <form class="space-y-4" onsubmit={Callback::from(|e: SubmitEvent| e.prevent_default())}>
            <div class="grid gap-4 sm:grid-cols-2">
                <label class="block space-y-1">
                    <span class="text-sm font-medium text-foreground">{ "Name" }</span>
                    <input
                        type="text"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground outline-none focus:border-accent"
                        value={(*name).clone()}
                        oninput={on_name}
                        disabled={disabled}
                    />
                </label>
                <label class="block space-y-1">
                    <span class="text-sm font-medium text-foreground">{ "Email" }</span>
                    <input
                        type="email"
                        class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground outline-none focus:border-accent"
                        value={(*email).clone()}
                        oninput={on_email}
                        disabled={disabled}
                    />
                </label>
            </div>
            <label class="block space-y-1">
                <span class="text-sm font-medium text-foreground">{ "Subject" }</span>
                <input
                    type="text"
                    class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground outline-none focus:border-accent"
                    value={(*subject).clone()}
                    oninput={on_subject}
                    disabled={disabled}
                    placeholder="Track instruction, shoot booking, data engineering…"
                />
            </label>
            <label class="block space-y-1">
                <span class="text-sm font-medium text-foreground">{ "Message" }</span>
                <textarea
                    class="min-h-[8rem] w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm text-foreground outline-none focus:border-accent"
                    value={(*message).clone()}
                    oninput={on_message}
                    disabled={disabled}
                />
            </label>
            { feedback }
            <Button variant={ButtonVariant::Primary} disabled={disabled} onclick={on_submit}>
                if disabled {
                    { "Sending…" }
                } else {
                    { "Send message" }
                }
            </Button>
            <Text tone={TextTone::Muted} class="text-xs">
                { "For booking track days, media shoots, or race engineering inquiries — tell me what you're working on and how I can help." }
            </Text>
        </form>
    }
}
