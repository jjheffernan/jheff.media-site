use super::{Login, Signup};
use crate::model::Auth;
use yew::html::create_portal;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AuthModalKind {
    SignIn,
    SignUp,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub kind: AuthModalKind,
    pub on_close: Callback<()>,
    pub on_login: Callback<Auth>,
}

fn document_body() -> Option<web_sys::Element> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
        .map(|b| b.into())
}

#[function_component(AuthModal)]
pub fn auth_modal(props: &Props) -> Html {
    let stop_propagation = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

    let title = match props.kind {
        AuthModalKind::SignIn => "Sign in",
        AuthModalKind::SignUp => "Sign up",
    };

    let form = match props.kind {
        AuthModalKind::SignIn => html! {
            <Login on_login={props.on_login.clone()} embedded=true />
        },
        AuthModalKind::SignUp => html! {
            <Signup embedded=true />
        },
    };

    let overlay = html! {
        <div
            class="fixed inset-0 z-[100] flex items-center justify-center overflow-y-auto bg-black/60 p-4 backdrop-blur-sm"
            onclick={props.on_close.reform(|_| ())}
        >
            <div
                class="relative my-auto w-full max-w-md rounded-2xl border border-border bg-surface-elevated shadow-2xl"
                role="dialog"
                aria-modal="true"
                aria-label={title}
                onclick={stop_propagation}
            >
                <button
                    type="button"
                    class="absolute right-3 top-3 z-10 rounded-md p-1.5 text-muted transition hover:bg-surface hover:text-foreground"
                    onclick={props.on_close.reform(|_| ())}
                    aria-label="Close"
                >
                    <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" d="M6 6l12 12M18 6L6 18" />
                    </svg>
                </button>
                <div class="max-h-[min(85vh,640px)] overflow-y-auto px-4 pb-4 pt-10">
                    { form }
                </div>
            </div>
        </div>
    };

    if let Some(body) = document_body() {
        create_portal(overlay, body)
    } else {
        overlay
    }
}
