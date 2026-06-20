use crate::components::Layout;
use crate::context::{AuthProvider, ThemeProvider};
use crate::router::Router;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct App {}

pub enum Msg {}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <ThemeProvider>
                    <AuthProvider>
                        <Layout>
                            <Router />
                        </Layout>
                    </AuthProvider>
                </ThemeProvider>
            </BrowserRouter>
        }
    }
}
