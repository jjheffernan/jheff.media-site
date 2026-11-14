use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub items: Vec<TabItem>,
    pub active: String,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
}

#[function_component(Tabs)]
pub fn tabs(props: &Props) -> Html {
    html! {
        <div class={format!("tabs {}", props.class)}>
            <div class="tabs-list" role="tablist">
                { for props.items.iter().map(|item| {
                    let id = item.id.clone();
                    let selected = props.active == item.id;
                    let on_change = props.on_change.clone();
                    html! {
                        <button
                            type="button"
                            role="tab"
                            class="tabs-trigger"
                            aria-selected={selected.to_string()}
                            onclick={Callback::from(move |_| on_change.emit(id.clone()))}
                        >
                            { &item.label }
                        </button>
                    }
                }) }
            </div>
            <div class="tabs-panel" role="tabpanel">
                { for props.children.iter() }
            </div>
        </div>
    }
}
