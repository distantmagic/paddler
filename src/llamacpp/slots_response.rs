use crate::llamacpp::slot::Slot;

pub struct SlotsResponse {
    pub error: Option<String>,
    pub is_authorized: Option<bool>,
    pub is_connect_error: Option<bool>,
    pub is_decode_error: Option<bool>,
    pub is_deserialize_error: Option<bool>,
    pub is_request_error: Option<bool>,
    pub is_unexpected_reponse_status: Option<bool>,
    pub is_slot_endpoint_enabled: Option<bool>,
    pub slots: Vec<Slot>,
}
