use crate::components::SocialHub;
use yew::prelude::*;

pub struct Social {}

impl Component for Social {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! { <SocialHub /> }
    }
}
