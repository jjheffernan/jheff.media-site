mod account;
mod booking;
mod contact;
mod content;
mod galleries;
mod gallery_detail;
mod home;
mod legal;
mod other_sites;
mod profile;
mod schedule;
mod shoot_detail;
mod shoots;
mod social;

use yew::prelude::*;
use yew_router::prelude::*;

pub use account::Account;
pub use booking::Booking;
pub use contact::Contact;
pub use content::Content;
pub use galleries::Galleries;
pub use gallery_detail::GalleryDetail;
pub use home::Home;
pub use legal::{Privacy, Terms};
pub use other_sites::OtherSites;
pub use profile::Profile;
pub use schedule::Schedule;
pub use shoot_detail::ShootDetail;
pub use shoots::Shoots;
pub use social::Social;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Routable)]
pub enum AppRoutes {
    #[at("/")]
    Home,
    #[at("/content")]
    Content,
    #[at("/schedule")]
    Schedule,
    #[at("/social")]
    Social,
    #[at("/shoots")]
    Shoots,
    #[at("/shoots/:id")]
    ShootDetail { id: String },
    #[at("/galleries")]
    Galleries,
    #[at("/galleries/:id")]
    GalleryDetail { id: String },
    #[at("/booking")]
    Booking,
    #[at("/contact")]
    Contact,
    #[at("/legal/terms")]
    Terms,
    #[at("/legal/privacy")]
    Privacy,
    #[at("/other-sites")]
    OtherSites,
    #[at("/account")]
    Account,
    #[at("/profile")]
    Profile,
    #[at("/404")]
    #[not_found]
    NotFound,
}

pub fn switch(routes: AppRoutes) -> Html {
    match routes {
        AppRoutes::Home => html! { <Home /> },
        AppRoutes::Content => html! { <Content /> },
        AppRoutes::Schedule => html! { <Schedule /> },
        AppRoutes::Social => html! { <Social /> },
        AppRoutes::Shoots => html! { <Shoots /> },
        AppRoutes::ShootDetail { id } => html! { <ShootDetail id={id} /> },
        AppRoutes::Galleries => html! { <Galleries /> },
        AppRoutes::GalleryDetail { id } => html! { <GalleryDetail id={id} /> },
        AppRoutes::Booking => html! { <Booking /> },
        AppRoutes::Contact => html! { <Contact /> },
        AppRoutes::Terms => html! { <Terms /> },
        AppRoutes::Privacy => html! { <Privacy /> },
        AppRoutes::OtherSites => html! { <OtherSites /> },
        AppRoutes::Account => html! { <Account /> },
        AppRoutes::Profile => html! { <Profile /> },
        AppRoutes::NotFound => html! {
            <p class="text-sm text-muted">{ "Page not found" }</p>
        },
    }
}
