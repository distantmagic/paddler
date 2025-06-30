mod desired_state_params;

pub use self::desired_state_params::DesiredStateParams;

pub trait RequestParams {
    fn request_id(&self) -> String;
}
