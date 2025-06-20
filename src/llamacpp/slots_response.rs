use crate::llamacpp::slot::Slot;

pub struct SlotsResponse {
    pub is_authorized: Option<bool>,
    pub error: Option<String>,
    pub is_slot_endpoint_enabled: Option<bool>,
    pub is_llamacpp_reachable: Option<bool>,
    pub is_llamacpp_response_decodeable: Option<bool>,
    pub is_llamacpp_request_error: Option<bool>,
    pub slots: Vec<Slot>,
}
