use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: String,
    #[prop_or_default]
    pub hint: Option<String>,
    #[prop_or_default]
    pub error: Option<String>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(false)]
    pub compact: bool,
}

/// Label + control + optional hint/error. Children should be the input/select/textarea.
#[function_component(Field)]
pub fn field(props: &Props) -> Html {
    let label_class = if props.compact { "label-xs" } else { "label" };
    html! {
        <label class={format!("form-field {}", props.class)}>
            <span class={label_class}>{ &props.label }</span>
            { for props.children.iter() }
            if let Some(hint) = &props.hint {
                <span class="form-hint">{ hint }</span>
            }
            if let Some(error) = &props.error {
                <span class="form-error">{ error }</span>
            }
        </label>
    }
}
