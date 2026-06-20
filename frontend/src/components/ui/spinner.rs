use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub label: Option<String>,
}

#[function_component(Spinner)]
pub fn spinner(props: &Props) -> Html {
    html! {
        <div class={format!("flex items-center gap-3 text-sm text-zinc-400 {}", props.class)} role="status">
            <span class="inline-block h-5 w-5 animate-spin rounded-full border-2 border-zinc-600 border-t-cyan-400" />
            if let Some(label) = &props.label {
                <span>{ label }</span>
            }
        </div>
    }
}
