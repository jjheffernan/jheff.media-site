use crate::routes::AppRoutes;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub route: AppRoutes,
    pub title: String,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: String,
}

#[function_component(LinkCard)]
pub fn link_card(props: &Props) -> Html {
    html! {
        <Link<AppRoutes>
            to={props.route.clone()}
            classes={classes!("card-link", props.class.clone())}
        >
            <h2 class="title-sm">{ props.title.clone() }</h2>
            <div class="mt-2 muted">
                { for props.children.iter() }
            </div>
        </Link<AppRoutes>>
    }
}
