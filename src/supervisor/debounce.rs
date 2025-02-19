use std::{
    collections::VecDeque,
    sync::{atomic::Ordering, Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use actix_web::web::Data;
use debounce::EventDebouncer;
use log::error;
use mavec::core::to_vec;
use serde_json::{Map, Value};

use crate::errors::{app_error::AppError, result::Result};

use super::management_service::State;

pub fn handle_throttle(state: Data<State>) -> Result<()> {
    if state.is_throttle_running.swap(true, Ordering::Relaxed) {
        return Ok(());
    }

    let state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;

            if let Some(last_request) = state.last_request.lock().ok().and_then(|lr| *lr) {
                if last_request.elapsed() >= Duration::from_millis(1000) {
                    let mut args = match state.args.lock() {
                        Ok(guard) => guard,
                        Err(e) => {
                            error!("Failed to acquire lock on args: {}", e);
                            continue;
                        }
                    };

                    if !args.is_empty() {
                        let llama_args = debounce_args(args.to_vec())
                            .and_then(|rebounced_args| Ok(to_vec(Value::Object(rebounced_args))?))
                            .and_then(|vec| to_llamacpp_arg(vec.into()))
                            .and_then(|llama_args| Ok(state.update_llamacpp.send(llama_args)));

                        match llama_args {
                            Ok(_) => args.clear(),
                            Err(e) => error!("Error processing args: {}", e),
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

pub fn to_llamacpp_arg(mut vec: VecDeque<String>) -> Result<Vec<String>> {
    if let Some(index) = vec.iter().position(|x| x == "binary") {
        vec.push_front(vec[index + 1].clone());
        vec.remove(index + 1);
        vec.remove(index + 1);
        vec.retain(|x| x != "");
        vec.push_front("--args".to_string());

        return Ok(vec.clone().into());
    }
    Err(AppError::UnexpectedError(
        "No binary found in JSON struct".to_string(),
    ))
}

pub fn debounce_args(maps: Vec<Map<String, Value>>) -> Result<Map<String, Value>> {
    let delay = Duration::from_millis(10);
    let debounced_btreemap = Arc::new(Mutex::new(Map::new()));
    let debounced_btreemap_clone = Arc::clone(&debounced_btreemap);

    let debouncer = EventDebouncer::new(delay, move |data: (String, Value)| {
        let mut map = match debounced_btreemap_clone.lock() {
            Ok(map) => map,
            Err(err) => {
                error!("Failed to acquire lock: {}", err);
                return;
            }
        };
        map.insert(data.0, data.1);
    });

    for peer in &maps {
        for (key, value) in peer {
            debouncer.put((key.clone(), value.clone()));
        }
    }

    sleep(Duration::from_millis(100));

    let map = debounced_btreemap.lock()?.clone();
    Ok(map)
}
