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
            classes={classes!(
                "block",
                "rounded-2xl",
                "border",
                "border-border",
                "bg-surface-elevated",
                "p-6",
                "shadow-sm",
                "shadow-black/5",
                "transition",
                "hover:border-accent/50",
                "hover:shadow-md",
                "dark:shadow-black/20",
                props.class.clone(),
            )}
        >
            <h2 class="text-lg font-semibold text-foreground">{ props.title.clone() }</h2>
            <div class="mt-2 text-sm text-muted">
                { for props.children.iter() }
            </div>
        </Link<AppRoutes>>
    }
}
