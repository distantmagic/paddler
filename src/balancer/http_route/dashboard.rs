use actix_web::get;
use actix_web::web;
use actix_web::Responder;
use askama_actix::Template;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {}

#[get("/dashboard")]
async fn respond() -> impl Responder {
    DashboardTemplate {}
}
