use crate::llamacpp::slot::Slot;

pub struct SlotsResponse {
    pub error: Option<String>,
    pub is_authorized: Option<bool>,
    pub is_reachable: Option<bool>,
    pub is_response_decodeable: Option<bool>,
    pub is_request_error: Option<bool>,
    pub is_slot_endpoint_enabled: Option<bool>,
    pub slots: Vec<Slot>,
}
