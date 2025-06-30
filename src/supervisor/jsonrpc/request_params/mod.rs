mod set_state_params;

pub use self::set_state_params::SetStateParams;

pub trait RequestParams {
    fn request_id(&self) -> String;
}
