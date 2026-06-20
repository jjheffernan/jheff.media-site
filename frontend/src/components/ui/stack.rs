use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum StackGap {
    Sm,
    Md,
    Lg,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(StackGap::Md)]
    pub gap: StackGap,
}

#[function_component(Stack)]
pub fn stack(props: &Props) -> Html {
    let gap = match props.gap {
        StackGap::Sm => "space-y-3",
        StackGap::Md => "space-y-6",
        StackGap::Lg => "space-y-8",
    };

    html! {
        <div class={format!("{} {}", gap, props.class)}>
            { for props.children.iter() }
        </div>
    }
}
