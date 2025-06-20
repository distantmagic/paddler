use actix_web::get;
use actix_web::web;
use actix_web::Responder;
use askama::Template;
use esbuild_metafile::filters;
use esbuild_metafile::HttpPreloader;

use crate::balancer::response::view;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    preloads: HttpPreloader,
}

#[get("/dashboard")]
async fn respond(preloads: HttpPreloader) -> impl Responder {
    view(DashboardTemplate {
        preloads,
    })
}
