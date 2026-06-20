mod footer;
mod header;
mod content_nav;
mod profile_menu;
mod theme_toggle;

use footer::Footer;
use header::Header;
use yew::prelude::*;

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
                <main class="mx-auto w-full max-w-6xl flex-1 px-4 py-8 sm:px-6 lg:px-8">
                    { for self.props.children.iter() }
                </main>
                <Footer />
            </div>
        }
    }
}
