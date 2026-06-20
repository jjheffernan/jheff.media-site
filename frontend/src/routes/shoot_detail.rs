use crate::components::content::ContentDetail;
use crate::routes::AppRoutes;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(ShootDetail)]
pub fn shoot_detail(props: &Props) -> Html {
    html! {
        <ContentDetail
            id={props.id.clone()}
            api_path="/api/shoots"
            back_route={AppRoutes::Shoots}
            back_label="Shoots"
        />
    }
}
