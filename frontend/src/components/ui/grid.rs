use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
    #[prop_or(1)]
    pub cols_sm: u8,
    #[prop_or(2)]
    pub cols_lg: u8,
}

#[function_component(Grid)]
pub fn grid(props: &Props) -> Html {
    let cols_sm = props.cols_sm.clamp(1, 4);
    let cols_lg = props.cols_lg.clamp(1, 6);
    let grid_class = match (cols_sm, cols_lg) {
        (1, 3) => "grid gap-4 sm:grid-cols-1 lg:grid-cols-3",
        (2, 3) => "grid gap-4 sm:grid-cols-2 lg:grid-cols-3",
        (2, 4) => "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
        (1, 2) => "grid gap-4 sm:grid-cols-1 lg:grid-cols-2",
        _ => "grid gap-4 sm:grid-cols-2 lg:grid-cols-3",
    };

    html! {
        <div class={format!("{} {}", grid_class, props.class)}>
            { for props.children.iter() }
        </div>
    }
}
