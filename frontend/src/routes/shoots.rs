use yew::prelude::*;

/// Placeholder shoot schedule for on-location automotive work.
pub struct Shoots {}

impl Component for Shoots {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="page shoots-page">
                <h1>{"Shoots"}</h1>
                <p class="page-lead">
                    {"Plan sessions, locations, and vehicles before you roll cameras."}
                </p>
                <ul class="placeholder-list">
                    <li class="placeholder-card">
                        <span class="placeholder-title">{"Private collection — 911 GT3"}</span>
                        <span class="placeholder-meta">{"Sat 9:00 · studio"}</span>
                    </li>
                    <li class="placeholder-card">
                        <span class="placeholder-title">{"Club meet — canyon run"}</span>
                        <span class="placeholder-meta">{"Sun 6:30 · Malibu"}</span>
                    </li>
                </ul>
            </div>
        }
    }
}
