use std::net::SocketAddr;

use actix_web::get;
use actix_web::web;
use actix_web::Responder;
use askama::Template;
use esbuild_metafile::filters;
use esbuild_metafile::HttpPreloader;

use crate::balancer::response::view;
use crate::balancer::web_admin_panel_service::template_data::TemplateData;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Template)]
#[template(path = "web_admin_panel.html")]
struct WebAdminPanelTemplate {
    buffered_request_timeout_millis: u128,
    inference_addr: SocketAddr,
    management_addr: SocketAddr,
    max_buffered_requests: i32,
    preloads: HttpPreloader,
    statsd_addr: String,
    statsd_prefix: String,
    statsd_reporting_interval_millis: u128,
}

#[get("/{_:.*}")]
async fn respond(
    preloads: HttpPreloader,
    template_data: web::Data<TemplateData>,
) -> impl Responder {
    view(WebAdminPanelTemplate {
        buffered_request_timeout_millis: template_data.buffered_request_timeout.as_millis(),
        inference_addr: template_data.inference_addr,
        management_addr: template_data.management_addr,
        max_buffered_requests: template_data.max_buffered_requests,
        preloads,
        statsd_addr: match template_data.statsd_addr {
            Some(addr) => addr.to_string(),
            None => String::new(),
        },
        statsd_prefix: template_data.statsd_prefix.clone(),
        statsd_reporting_interval_millis: template_data.statsd_reporting_interval.as_millis(),
    })
}
