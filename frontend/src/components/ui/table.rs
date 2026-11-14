use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
}

#[function_component(Table)]
pub fn table(props: &Props) -> Html {
    html! {
        <div class={format!("data-table-wrap {}", props.class)}>
            <table class="data-table">
                { for props.children.iter() }
            </table>
        </div>
    }
}
