mod footer;
mod header;

use footer::Footer;
use header::Header;
use yew::prelude::*;

/// Site layout.
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
            <div class={classes!("site-layout", self.props.class.clone())}>
                <Header />
                <div class="content">
                    { for self.props.children.iter() }
                </div>
                <Footer />
            </div>
        }
    }
}
