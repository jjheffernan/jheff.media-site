use crate::model::OtherSitesResponse;
use crate::routes::AppRoutes;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

fn footer_link(route: AppRoutes, label: &str) -> Html {
    html! {
        <Link<AppRoutes>
            to={route}
            classes="text-sm text-muted transition hover:text-accent"
        >
            { label }
        </Link<AppRoutes>>
    }
}

/// Site footer with legal links and business info.
pub struct Footer {
    props: Props,
    has_other_sites: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub class: String,
}

pub enum Msg {
    OtherSitesLoaded(bool),
}

impl Component for Footer {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        spawn_local(async move {
            let has = match Request::get("/api/sites/other").send().await {
                Ok(resp) if resp.ok() => resp
                    .json::<OtherSitesResponse>()
                    .await
                    .map(|data| !data.sites.is_empty())
                    .unwrap_or(false),
                _ => false,
            };
            link.send_message(Msg::OtherSitesLoaded(has));
        });

        Self {
            props: ctx.props().clone(),
            has_other_sites: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OtherSitesLoaded(has) => {
                self.has_other_sites = has;
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, props: &Self::Properties) -> bool {
        if self.props != *props {
            self.props = props.clone();
            true
        } else {
            false
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <footer class={format!(
                "border-t border-border bg-surface px-4 py-10 sm:px-6 {}",
                self.props.class
            )}>
                <div class="mx-auto max-w-6xl">
                    <div class="grid grid-cols-1 gap-10 md:grid-cols-3 md:gap-8">
                        <div class="space-y-3">
                            <p class="text-sm font-semibold text-foreground">{ "jheff.media" }</p>
                            <p class="text-sm leading-relaxed text-muted">
                                { "Media-first hub for @jheffmedia — track instruction, race data, social content, and automotive pursuits." }
                            </p>
                        </div>

                        <div class="space-y-3">
                            <p class="text-xs font-semibold uppercase tracking-wider text-muted">
                                { "Explore" }
                            </p>
                            <nav class="flex flex-col gap-2">
                                { footer_link(AppRoutes::Content, "Content hub") }
                                { footer_link(AppRoutes::Galleries, "Galleries") }
                                { footer_link(AppRoutes::Shoots, "Shoots") }
                                { footer_link(AppRoutes::Social, "Social") }
                                { footer_link(AppRoutes::Schedule, "Schedule") }
                                { footer_link(AppRoutes::Booking, "Booking") }
                                if self.has_other_sites {
                                    { footer_link(AppRoutes::OtherSites, "Other sites") }
                                }
                            </nav>
                        </div>

                        <div class="space-y-3">
                            <p class="text-xs font-semibold uppercase tracking-wider text-muted">
                                { "Legal" }
                            </p>
                            <nav class="flex flex-col gap-2">
                                { footer_link(AppRoutes::Terms, "Terms of use") }
                                { footer_link(AppRoutes::Privacy, "Privacy policy") }
                                { footer_link(AppRoutes::Contact, "Contact") }
                            </nav>
                        </div>
                    </div>

                    <div class="mt-10 border-t border-border pt-6">
                        <p class="text-xs text-muted">
                            { "© 2026 jheff.media — photos and video served from your own media library, not bundled in this site." }
                        </p>
                    </div>
                </div>
            </footer>
        }
    }
}
