use crate::llamacpp::slot::Slot;

pub struct SlotsResponse {
    pub is_authorized: Option<bool>,
    pub is_slot_endpoint_enabled: Option<bool>,
    pub slots: Vec<Slot>,
}
