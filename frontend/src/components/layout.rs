mod footer;
mod header;
mod content_nav;
mod profile_menu;
mod theme_toggle;

use footer::Footer;
use gloo_net::http::Request;
use header::Header;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(PageBeacon)]
fn page_beacon() -> Html {
    let location = use_location();
    let path = location
        .as_ref()
        .map(|loc| loc.path().to_string())
        .unwrap_or_default();
    use_effect_with(path, |path| {
        if path.starts_with('/') {
            let body = format!(r#"{{"path":"{}"}}"#, path.replace('\\', "").replace('"', ""));
            spawn_local(async move {
                if let Ok(req) = Request::post("/api/telemetry")
                    .header("Content-Type", "application/json")
                    .body(body)
                {
                    let _ = req.send().await;
                }
            });
        }
        || ()
    });
    html! {}
}

/// Site shell: header, routed content, footer.
pub struct Layout {
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
}

pub enum Msg {}

impl Component for Layout {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            props: ctx.props().clone(),
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, props: &Self::Properties) -> bool {
        if self.props != *props {
            self.props = props.clone();
            true
        } else {
            false
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={format!("flex min-h-screen flex-col bg-background text-foreground {}", self.props.class)}>
                <Header />
                <PageBeacon />
                <main class="page">
                    { for self.props.children.iter() }
                </main>
                <Footer />
            </div>
        }
    }
}
