use actix_web::HttpResponse;
use actix_web::http::StatusCode;

use paddler_messaging::jsonrpc::error::Error as JsonRpcError;
use paddler_typesafe_translation::typesafe_translation_error::TypeSafeTranslationError;

use crate::compatibility::typesafe_service::typesafe_error_body::TypeSafeErrorBody;
use crate::compatibility::upstream_failure::UpstreamFailure;

#[derive(Debug, Eq, PartialEq)]
pub struct TypeSafeDecisionFailure {
    pub detail: String,
    pub status_code: StatusCode,
}

impl TypeSafeDecisionFailure {
    #[must_use]
    pub fn unfinished() -> Self {
        Self {
            detail: "the agent stopped before it finished the decision".to_owned(),
            status_code: UpstreamFailure::RelayFailed.status_code(),
        }
    }

    #[must_use]
    pub fn into_http_response(self) -> HttpResponse {
        TypeSafeErrorBody {
            detail: self.detail,
        }
        .into_http_response(self.status_code)
    }
}

impl From<JsonRpcError> for TypeSafeDecisionFailure {
    fn from(wire_error: JsonRpcError) -> Self {
        Self {
            status_code: UpstreamFailure::from(&wire_error).status_code(),
            detail: wire_error.description,
        }
    }
}

impl From<TypeSafeTranslationError> for TypeSafeDecisionFailure {
    fn from(translation_error: TypeSafeTranslationError) -> Self {
        Self {
            detail: translation_error.to_string(),
            status_code: UpstreamFailure::RelayFailed.status_code(),
        }
    }
}
