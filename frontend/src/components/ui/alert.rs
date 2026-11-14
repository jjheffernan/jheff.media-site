use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum AlertVariant {
    Default,
    Info,
    Success,
    Warning,
    Destructive,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or(AlertVariant::Default)]
    pub variant: AlertVariant,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(Alert)]
pub fn alert(props: &Props) -> Html {
    let variant = match props.variant {
        AlertVariant::Default => "alert alert-default",
        AlertVariant::Info => "alert alert-info",
        AlertVariant::Success => "alert alert-success",
        AlertVariant::Warning => "alert alert-warning",
        AlertVariant::Destructive => "alert alert-destructive",
    };
    html! {
        <div class={classes!(variant, props.class.clone())} role="status">
            if let Some(title) = &props.title {
                <p class="alert-title">{ title }</p>
            }
            <div class="alert-body">{ props.children.clone() }</div>
        </div>
    }
}
