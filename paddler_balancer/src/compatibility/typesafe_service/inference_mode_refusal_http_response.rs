use actix_web::HttpResponse;

use crate::cluster_does_not_serve_inference_capability::ClusterDoesNotServeInferenceCapability;
use crate::compatibility::typesafe_service::typesafe_error_body::TypeSafeErrorBody;
use crate::compatibility::upstream_failure::UpstreamFailure;

#[must_use]
pub fn inference_mode_refusal_http_response(
    cluster_does_not_serve_inference_capability: ClusterDoesNotServeInferenceCapability,
) -> HttpResponse {
    TypeSafeErrorBody {
        detail: cluster_does_not_serve_inference_capability.to_string(),
    }
    .into_http_response(UpstreamFailure::Unavailable.status_code())
}
