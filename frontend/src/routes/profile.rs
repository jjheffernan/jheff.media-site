use yew::prelude::*;

/// Profile page lining to login.
pub struct Profile {}

impl Component for Profile {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="profile-page">
                {"Profile Page"}
            </div>
        }
    }
}
