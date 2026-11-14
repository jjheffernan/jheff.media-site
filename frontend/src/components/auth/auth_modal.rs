use super::{Login, Signup};
use crate::components::ui::Dialog;
use crate::model::Auth;
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

#[function_component(AuthModal)]
pub fn auth_modal(props: &Props) -> Html {
    let kind = use_state(|| props.kind);

    {
        let kind = kind.clone();
        let prop_kind = props.kind;
        use_effect_with(prop_kind, move |prop_kind| {
            kind.set(*prop_kind);
            || ()
        });
    }

    let title = match *kind {
        AuthModalKind::SignIn => "Sign in",
        AuthModalKind::SignUp => "Sign up",
    };

    let form = match *kind {
        AuthModalKind::SignIn => html! {
            <Login on_login={props.on_login.clone()} embedded=true />
        },
        AuthModalKind::SignUp => html! {
            <Signup embedded=true />
        },
    };

    let switch = match *kind {
        AuthModalKind::SignIn => {
            let kind = kind.clone();
            html! {
                <p class="mt-4 text-center text-sm text-muted">
                    { "Need an account? " }
                    <button
                        type="button"
                        class="font-medium text-accent underline-offset-2 hover:underline"
                        onclick={Callback::from(move |_| kind.set(AuthModalKind::SignUp))}
                    >
                        { "Sign up" }
                    </button>
                </p>
            }
        }
        AuthModalKind::SignUp => {
            let kind = kind.clone();
            html! {
                <p class="mt-4 text-center text-sm text-muted">
                    { "Already have an account? " }
                    <button
                        type="button"
                        class="font-medium text-accent underline-offset-2 hover:underline"
                        onclick={Callback::from(move |_| kind.set(AuthModalKind::SignIn))}
                    >
                        { "Sign in" }
                    </button>
                </p>
            }
        }
    };

    html! {
        <Dialog
            open=true
            on_close={props.on_close.clone()}
            title={title}
        >
            <div class="max-h-[min(70vh,560px)] overflow-y-auto">
                { form }
                { switch }
            </div>
        </Dialog>
    }
}
