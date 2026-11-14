use yew::html::create_portal;
use yew::prelude::*;

fn document_body() -> Option<web_sys::Element> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
        .map(|b| b.into())
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub open: bool,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub title: Option<String>,
    #[prop_or_default]
    pub description: Option<String>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub footer: Option<Html>,
    #[prop_or(false)]
    pub sheet: bool,
    #[prop_or(false)]
    pub wide: bool,
    #[prop_or_default]
    pub class: String,
}

/// Modal dialog (or bottom sheet). Renders into `document.body` when open.
#[function_component(Dialog)]
pub fn dialog(props: &Props) -> Html {
    if !props.open {
        return html! {};
    }

    let stop = Callback::from(|e: MouseEvent| e.stop_propagation());
    let panel_class = format!(
        "dialog-panel {} {} {}",
        if props.sheet { "is-sheet" } else { "" },
        if props.wide { "is-wide" } else { "" },
        props.class
    );

    let overlay = html! {
        <div class="dialog-backdrop" onclick={props.on_close.reform(|_| ())}>
            <div
                class={panel_class}
                role="dialog"
                aria-modal="true"
                aria-label={props.title.clone().unwrap_or_else(|| "Dialog".into())}
                onclick={stop}
            >
                <div class="dialog-header">
                    <div>
                        if let Some(title) = &props.title {
                            <h2 class="dialog-title">{ title }</h2>
                        }
                        if let Some(description) = &props.description {
                            <p class="dialog-desc">{ description }</p>
                        }
                    </div>
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm"
                        onclick={props.on_close.reform(|_| ())}
                        aria-label="Close"
                    >
                        { "✕" }
                    </button>
                </div>
                { for props.children.iter() }
                if let Some(footer) = &props.footer {
                    <div class="dialog-actions">{ footer.clone() }</div>
                }
            </div>
        </div>
    };

    if let Some(body) = document_body() {
        create_portal(overlay, body)
    } else {
        overlay
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct ConfirmProps {
    pub open: bool,
    pub title: String,
    pub message: String,
    pub on_confirm: Callback<()>,
    pub on_cancel: Callback<()>,
    #[prop_or("Confirm".into())]
    pub confirm_label: String,
    #[prop_or(false)]
    pub destructive: bool,
}

#[function_component(ConfirmDialog)]
pub fn confirm_dialog(props: &ConfirmProps) -> Html {
    let confirm_class = if props.destructive {
        "btn btn-destructive"
    } else {
        "btn btn-primary"
    };
    let footer = html! {
        <>
            <button type="button" class="btn btn-outline" onclick={props.on_cancel.reform(|_| ())}>
                { "Cancel" }
            </button>
            <button type="button" class={confirm_class} onclick={props.on_confirm.reform(|_| ())}>
                { &props.confirm_label }
            </button>
        </>
    };
    html! {
        <Dialog
            open={props.open}
            on_close={props.on_cancel.clone()}
            title={Some(props.title.clone())}
            description={Some(props.message.clone())}
            footer={Some(footer)}
        />
    }
}
