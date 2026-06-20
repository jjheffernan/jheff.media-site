use yew::prelude::*;

/// Placeholder gallery index for automotive photo collections.
pub struct Galleries {}

impl Component for Galleries {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="page galleries-page">
                <h1>{"Galleries"}</h1>
                <p class="page-lead">
                    {"Organize finished sets by vehicle, event, or client delivery."}
                </p>
                <ul class="placeholder-list">
                    <li class="placeholder-card">
                        <span class="placeholder-title">{"Track Day — Laguna Seca"}</span>
                        <span class="placeholder-meta">{"48 selects · draft"}</span>
                    </li>
                    <li class="placeholder-card">
                        <span class="placeholder-title">{"Dealer Inventory — Q2"}</span>
                        <span class="placeholder-meta">{"120 selects · published"}</span>
                    </li>
                </ul>
            </div>
        }
    }
}
