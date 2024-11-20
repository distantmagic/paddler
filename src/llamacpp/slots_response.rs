use crate::llamacpp::slot::Slot;

pub struct SlotsResponse {
    pub is_authorized: bool,
    pub slots: Vec<Slot>,
}
