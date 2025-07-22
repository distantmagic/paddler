use actix_web::web::Data;

use crate::balancer::buffered_request_manager::BufferedRequestManager;

pub struct InferenceSocketControllerContext {
    pub buffered_request_manager: Data<BufferedRequestManager>,
}
