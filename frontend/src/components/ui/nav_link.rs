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
            classes={classes!(
                "rounded-md",
                "px-3",
                "py-2",
                "text-sm",
                "font-medium",
                "text-muted",
                "transition",
                "hover:bg-surface-elevated",
                "hover:text-foreground",
                props.class.clone(),
            )}
        >
            { props.label.clone() }
        </Link<AppRoutes>>
    }
}
