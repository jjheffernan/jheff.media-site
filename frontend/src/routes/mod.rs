mod galleries;
mod home;
mod profile;
mod shoots;

use yew::prelude::*;
use yew_router::prelude::*;

pub use galleries::Galleries;
pub use home::Home;
pub use profile::Profile;
pub use shoots::Shoots;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Routable)]
pub enum AppRoutes {
    #[at("/")]
    Home,
    #[at("/shoots")]
    Shoots,
    #[at("/galleries")]
    Galleries,
    #[at("/profile")]
    Profile,
    #[at("/404")]
    #[not_found]
    NotFound,
}

pub fn switch(routes: AppRoutes) -> Html {
    match routes {
        AppRoutes::Home => html! { <Home /> },
        AppRoutes::Shoots => html! { <Shoots /> },
        AppRoutes::Galleries => html! { <Galleries /> },
        AppRoutes::Profile => html! { <Profile /> },
        AppRoutes::NotFound => html! { <p>{"Page not found"}</p> },
    }
}
