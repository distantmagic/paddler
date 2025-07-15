use actix_web::get;
use actix_web::web;
use actix_web::Responder;
use askama::Template;
use esbuild_metafile::filters;
use esbuild_metafile::HttpPreloader;

use crate::balancer::response::view;
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    preloads: HttpPreloader,
    web_dashboard_service_configuration: web::Data<WebDashboardServiceConfiguration>,
}

#[get("/{_:.*}")]
async fn respond(
    preloads: HttpPreloader,
    web_dashboard_service_configuration: web::Data<WebDashboardServiceConfiguration>,
) -> impl Responder {
    view(DashboardTemplate {
        preloads,
        web_dashboard_service_configuration,
    })
}
