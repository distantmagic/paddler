use std::{net::SocketAddr, path::Path};

use serde::{Deserialize, Serialize};

use crate::{errors::result::Result, errors::app_error::AppError};

#[derive(Clone, Deserialize, Serialize)]
pub struct LlamacppConfiguration {
    address: SocketAddr,
    llama_path: String,
    model_path: String,
    threads_number: i8
}

impl LlamacppConfiguration {
    pub fn new(
        address: SocketAddr,
        llama_path: String,
        model_path: String,
        threads_number: i8
    ) -> Self {
        LlamacppConfiguration {
            address,
            llama_path,
            model_path,
            threads_number
        }
    }

    pub fn get_port(self) -> String {
        unsafe {
            self.address.to_string().split(':')
                .nth(1)
                .unwrap_unchecked()
                .parse::<String>()
                .unwrap_unchecked()
        }
    }
    
    pub fn get_host(self) -> String {
        unsafe {
            self.address.to_string().split(':')
                .nth(0)
                .unwrap_unchecked()
                .parse::<String>()
                .unwrap_unchecked()
        }
    }

    pub fn get_model_path(self) -> String {
        self.model_path
    }

    pub fn get_binary_path(self) -> String {
        self.llama_path
    }

    pub fn get_threads_number(self) -> i8 {
        self.threads_number
    }

    pub fn set_configuration(&mut self, addr: String) {
        // self.address = addr;
    }    
    
    pub fn is_a_gguf_file(self) -> Result<()> {
        let file = Path::new(&self.model_path);
    
        if file.exists() {
            if let Some(ext) = file.extension() {
                if ext.to_str() == Some("gguf") {
                    return Ok(());
                }
                return Err(AppError::InvalidFileError(
                    "File must be gguf.".to_owned(),
                ))
            }
            return Err(AppError::InvalidFileError(
                "Insert a file with a valid name.".to_owned(),
            ))
        }
        return Err(AppError::InvalidFileError(
            "File doesnt exist.".to_owned(),
        ))
    }
}