use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub route: AppRoutes,
    pub label: String,
    #[prop_or_default]
    pub class: String,
}

#[function_component(NavLink)]
pub fn nav_link(props: &Props) -> Html {
    html! {
        <Link<AppRoutes>
            to={props.route.clone()}
            classes={classes!("nav-link", props.class.clone())}
        >
            { props.label.clone() }
        </Link<AppRoutes>>
    }
}
