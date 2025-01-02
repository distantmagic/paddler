use std::{net::SocketAddr, path::Path};

use serde::{Deserialize, Serialize};

use crate::{errors::app_error::AppError, errors::result::Result};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct LlamacppConfiguration {
    address: SocketAddr,
    pub binary: String,
    pub model: String,
    pub threads: i8,
    pub no_webui: bool,
    pub slots: bool,
    pub predict: i8,
    pub ctx_size: i8,
    pub temperature: i8,
    pub props: bool,
}

impl LlamacppConfiguration {
    pub fn new(
        address: SocketAddr,
        binary: String,
        model: String,
        threads: i8,
        no_webui: bool,
        slots: bool,
        predict: i8,
        ctx_size: i8,
        temperature: i8,
        props: bool,
    ) -> Self {
        LlamacppConfiguration {
            address,
            binary,
            model,
            threads,
            no_webui,
            slots,
            predict,
            ctx_size,
            temperature,
            props,
        }
    }

    pub fn get_port(self) -> String {
        unsafe {
            self.address
                .to_string()
                .split(':')
                .nth(1)
                .unwrap_unchecked()
                .parse::<String>()
                .unwrap_unchecked()
        }
    }

    pub fn get_host(self) -> String {
        unsafe {
            self.address
                .to_string()
                .split(':')
                .nth(0)
                .unwrap_unchecked()
                .parse::<String>()
                .unwrap_unchecked()
        }
    }

    pub fn is_a_gguf_file(model: String) -> Result<()> {
        let file = Path::new(&model);

        if file.exists() {
            if let Some(ext) = file.extension() {
                if ext.to_str() == Some("gguf") {
                    return Ok(());
                }
                return Err(AppError::InvalidFileError("File must be gguf.".to_owned()));
            }
            return Err(AppError::InvalidFileError(
                "Insert a file with a valid name.".to_owned(),
            ));
        }
        return Err(AppError::InvalidFileError("File doesnt exist.".to_owned()));
    }
}
