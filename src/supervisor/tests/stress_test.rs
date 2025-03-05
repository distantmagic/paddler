#[cfg(test)]
mod tests {
    use std::{
        sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
        time::{Duration, Instant},
    };

    use actix_web::web::Data;
    use lazy_static::lazy_static;
    use serde_json::{Map, Value};
    use throttle::Throttle;
    use tokio::{sync::broadcast::channel, time::sleep};

    use crate::{
        errors::result::Result,
        supervisor::{debounce::handle_throttle, management_service::State},
    };

    lazy_static! {
        static ref ARG1: Map<String, Value> = {
            let mut arg1: Map<String, Value> = Map::new();
            arg1.insert(
                "binary".to_string(),
                Value::String("llama-server".to_string()),
            );
            arg1.insert("-m".to_string(), Value::String("foo".to_string()));
            arg1.insert("--port".to_string(), Value::String("8088".to_string()));
            arg1
        };
        static ref ARG2: Map<String, Value> = {
            let mut arg2: Map<String, Value> = Map::new();

            arg2.insert("-m".to_string(), Value::String("bar".to_string()));
            arg2.insert("-cb".to_string(), Value::String("".to_string()));
            arg2
        };
        static ref ARG3: Map<String, Value> = {
            let mut arg3: Map<String, Value> = Map::new();

            arg3.insert("--ctx-size".to_string(), Value::String("5000".to_string()));
            arg3
        };
        static ref DEBOUNCED_ARG: Vec<String> = {
            let debounced_arg = vec![
                "--args".to_string(),
                "llama-server".to_string(),
                "-m".to_string(),
                "bar".to_string(),
                "--port".to_string(),
                "8088".to_string(),
                "-cb".to_string(),
                "--ctx-size".to_string(),
                "5000".to_string(),
            ];

            debounced_arg
        };
    }

    #[tokio::test]
    async fn arguments_are_debounced() -> Result<()> {
        let (update_llamacpp, mut update_llamacpp_rx) = channel::<Vec<String>>(1);

        let state = Data::new(State {
            update_llamacpp,
            throttle: Mutex::new(Throttle::new(Duration::from_millis(600), 20)),
            args: Mutex::new(Vec::new()),
            last_request: Arc::new(Mutex::new(None)),
            is_throttle_running: AtomicBool::new(false),
        });

        let _ = handle_throttle(state.clone());

        // 700 miliseconds span request
        make_request(state.clone(), ARG1.clone())?;
        sleep(Duration::from_millis(700)).await;

        // 800 miliseconds span request
        make_request(state.clone(), ARG2.clone())?;
        sleep(Duration::from_millis(800)).await;

        // 900 miliseconds span request
        make_request(state.clone(), ARG3.clone())?;
        sleep(Duration::from_millis(900)).await;

        state.is_throttle_running.store(false, Ordering::Relaxed);

        // Wait 1 second (enough time to batch the 3 requests)
        sleep(Duration::from_millis(1000)).await;

        let actual_arg = update_llamacpp_rx.recv().await.unwrap();

        assert_eq!(actual_arg, *DEBOUNCED_ARG);

        Ok(())
    }

    #[tokio::test]
    async fn arguments_are_not_debounced() -> Result<()> {
        let (update_llamacpp_tx, mut update_llamacpp_rx) = channel::<Vec<String>>(10);

        let state = Data::new(State {
            update_llamacpp: update_llamacpp_tx,
            throttle: Mutex::new(Throttle::new(Duration::from_millis(600), 20)),
            args: Mutex::new(Vec::new()),
            last_request: Arc::new(Mutex::new(None)),
            is_throttle_running: AtomicBool::new(false),
        });

        handle_throttle(state.clone())?;

        // Two seconds span request
        make_request(state.clone(), ARG1.clone())?;
        sleep(Duration::from_millis(2000)).await;

        // Two seconds span request
        make_request(state.clone(), ARG2.clone())?;      
        sleep(Duration::from_millis(2000)).await;

        make_request(state.clone(), ARG3.clone())?;

        let actual_arg = update_llamacpp_rx.recv().await.unwrap();

        assert_ne!(actual_arg, *DEBOUNCED_ARG);

        Ok(())
    }

    fn make_request(state: Data<State>, arg: Map<String, Value>) -> Result<()> {
        let now = Instant::now();
    
        let mut last_request_time = state.last_request.lock()?;
        let mut args_vec = state.args.lock()?;
        let mut throttle = state.throttle.lock()?;
    
        args_vec.push(arg);
        let _ = throttle.accept().ok();
        *last_request_time = Some(now);

        // As handle_request is a loop, we need to manualy stop it after a request
        let is_thtorrle_running = &state.is_throttle_running;
        is_thtorrle_running.store(false, Ordering::Relaxed);

        Ok(())
    }
}
