use crate::errors::app_error::AppError;

pub type Result<T> = std::result::Result<T, AppError>;
