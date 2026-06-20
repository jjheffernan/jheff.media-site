use crate::components::content::ContentDetail;
use crate::routes::AppRoutes;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(GalleryDetail)]
pub fn gallery_detail(props: &Props) -> Html {
    html! {
        <ContentDetail
            id={props.id.clone()}
            api_path="/api/galleries"
            back_route={AppRoutes::Galleries}
            back_label="Galleries"
        />
    }
}
