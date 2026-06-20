use crate::api::{
    admin_controller, account_controller, contact_controller, content_controller,
    feed_controller, media_controller, print_controller, schedule_controller, social_controller,
};
#[cfg(feature = "forward-frontend")]
use crate::services::forward_frontend::forward;
#[cfg(not(feature = "forward-frontend"))]
use crate::services::serve_frontend::serve_static;
use actix_web::web;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    info!("Configurating routes...");
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/feed")
                    .route("", web::get().to(feed_controller::list))
                    .route("/thumbnail/{asset_id}", web::get().to(feed_controller::thumbnail)),
            )
            .service(
                web::scope("/schedule")
                    .route("", web::get().to(schedule_controller::config)),
            )
            .service(
                web::scope("/social")
                    .route("/hub", web::get().to(social_controller::hub)),
            )
            .service(
                web::scope("/galleries")
                    .route("", web::get().to(content_controller::list_galleries))
                    .route("/{id}", web::get().to(content_controller::get_gallery)),
            )
            .service(
                web::scope("/shoots")
                    .route("", web::get().to(content_controller::list_shoots))
                    .route("/{id}", web::get().to(content_controller::get_shoot)),
            )
            .service(
                web::scope("/booking")
                    .route("", web::get().to(content_controller::booking)),
            )
            .service(
                web::scope("/sites")
                    .route("/other", web::get().to(content_controller::other_sites)),
            )
            .service(
                web::scope("/contact")
                    .route("", web::post().to(contact_controller::submit)),
            )
            .service(
                web::scope("/media")
                    .route("/feed", web::get().to(media_controller::feed))
                    .route(
                        "/instagram/featured",
                        web::get().to(media_controller::instagram_featured),
                    ),
            )
            .service(
                web::scope("/account")
                    .route("/prints", web::get().to(print_controller::list))
                    .route("/prints", web::post().to(print_controller::add))
                    .route("/prints/{id}", web::delete().to(print_controller::remove)),
            )
            .service(
                web::scope("/admin")
                    .route("/galleries", web::post().to(admin_controller::create_gallery))
                    .route("/shoots", web::post().to(admin_controller::create_shoot)),
            )
            .service(
                web::scope("/auth")
                    .service(web::resource("/signup").route(web::post().to(account_controller::signup)))
                    .service(web::resource("/login").route(web::post().to(account_controller::login)))
                    .service(
                        web::resource("/logout").route(web::post().to(account_controller::logout)),
                    )
                    .route("/me", web::get().to(account_controller::me))
                    .route("/password", web::post().to(account_controller::change_password))
                    .route(
                        "/email-change",
                        web::post().to(account_controller::request_email_change),
                    )
                    .route("/2fa/enroll", web::post().to(account_controller::totp_enroll))
                    .route("/2fa/confirm", web::post().to(account_controller::totp_confirm))
                    .route("/2fa/disable", web::post().to(account_controller::totp_disable)),
            ),
    );

    #[cfg(feature = "forward-frontend")]
    {
        info!("Configurating frontend reverse proxy...");
        cfg.default_service(web::route().to(forward));
    }

    #[cfg(not(feature = "forward-frontend"))]
    {
        info!("Configurating static frontend...");
        cfg.default_service(web::route().to(serve_static));
    }
}
