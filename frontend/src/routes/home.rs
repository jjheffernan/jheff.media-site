use yew::prelude::*;

/// Dashboard for automotive photography operations.
pub struct Home {}

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="page home-page">
                <h1>{"Automotive Photography"}</h1>
                <p class="page-lead">
                    {"Manage shoots, galleries, and client deliveries for jheff.media."}
                </p>
                <section class="dashboard-grid">
                    <article class="dashboard-card">
                        <h2>{"Shoots"}</h2>
                        <p>{"Schedule track days, dealer shoots, and private collections."}</p>
                    </article>
                    <article class="dashboard-card">
                        <h2>{"Galleries"}</h2>
                        <p>{"Curate selects, mark favorites, and publish client galleries."}</p>
                    </article>
                    <article class="dashboard-card">
                        <h2>{"Deliverables"}</h2>
                        <p>{"Export web, print, and social crops from a single master set."}</p>
                    </article>
                </section>
            </div>
        }
    }
}
