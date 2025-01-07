use std::collections::VecDeque;

use crate::errors::{app_error::AppError, result::Result};

#[derive(Clone, Debug)]
pub struct Config(pub VecDeque<String>);

impl Config {
    pub fn to_llamacpp_arg(&mut self) -> Result<Vec<String>> {
        if let Some(index) = self.0.iter().position(|x| x == "binary") {
            self.0.push_front(self.0[index + 1].clone());
            self.0.remove(index + 1);
            self.0.remove(index + 1);
            self.0.retain(|x| x != "");
            self.0.push_front("--args".to_string());

            return Ok(self.0.clone().into());
        }
        Err(AppError::UnexpectedError(
            "No binary found in JSON struct".to_string(),
        ))
    }
}
