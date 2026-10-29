use yew::prelude::*;

#[function_component(Hero)]
pub fn hero() -> Html {
    html! {
        <div>
            <p class="kicker">
                { "Automotive media & pursuit" }
            </p>
            <p class="muted">
                { "Track instruction, race data engineering, social content, action-camera runs, and restoration projects." }
            </p>
        </div>
    }
}
