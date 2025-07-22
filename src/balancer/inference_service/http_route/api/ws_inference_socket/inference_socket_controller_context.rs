use actix_web::web::Data;

use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;

pub struct InferenceSocketControllerContext {
    pub buffered_request_manager: Data<BufferedRequestManager>,
    pub inference_service_configuration: Data<InferenceServiceConfiguration>,
}
