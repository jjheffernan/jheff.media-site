use yew::prelude::*;

/// Footer e.g. for displaying copyright notice and version info.
pub struct Footer {
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
}

pub enum Msg {}

impl Component for Footer {
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
            <div class={classes!("site-footer", self.props.class.clone())}>
                <p>{"jheff.media — automotive photography"}</p>
            </div>
        }
    }
}
