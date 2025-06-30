mod bad_request_params;
mod too_many_requests_params;
mod version_params;

pub use self::bad_request_params::BadRequestParams;
pub use self::too_many_requests_params::TooManyRequestsParams;
pub use self::version_params::VersionParams;

pub trait NotificationParams {}
