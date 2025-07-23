use actix_web::get;
use actix_web::web;
use actix_web::Responder;
use askama::Template;
use esbuild_metafile::filters;
use esbuild_metafile::HttpPreloader;

use crate::balancer::response::view;
use crate::balancer::web_admin_panel_service::configuration::Configuration as WebAdminPanelServiceConfiguration;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Template)]
#[template(path = "web_admin_panel.html")]
struct WebAdminPanelTemplate {
    preloads: HttpPreloader,
    web_admin_panel_service_configuration: web::Data<WebAdminPanelServiceConfiguration>,
}

#[get("/{_:.*}")]
async fn respond(
    preloads: HttpPreloader,
    web_admin_panel_service_configuration: web::Data<WebAdminPanelServiceConfiguration>,
) -> impl Responder {
    view(WebAdminPanelTemplate {
        preloads,
        web_admin_panel_service_configuration,
    })
}
