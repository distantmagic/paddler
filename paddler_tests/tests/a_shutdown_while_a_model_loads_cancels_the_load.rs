#![cfg(feature = "tests_that_use_llms")]

use std::mem::discriminant;
use std::pin::pin;
use std::sync::mpsc::channel;

use futures_util::FutureExt as _;
use tokio::runtime::Builder;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::llama_model_settings::LlamaModelSettings;
use paddler_agent_runtime::loaded_llama_model::LoadedLlamaModel;
use paddler_agent_runtime::startup_outcome::StartupOutcome;
use paddler_inference_parameters::all_gpu_layers::ALL_GPU_LAYERS;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;

const ONLY_BLOCKING_THREAD: usize = 1;

#[test]
fn a_shutdown_while_a_model_loads_cancels_the_load() {
    Builder::new_current_thread()
        .max_blocking_threads(ONLY_BLOCKING_THREAD)
        .enable_all()
        .build()
        .expect("the test runtime must build")
        .block_on(async {
            let agent_shutdown = CancellationToken::new();
            let (release_blocking_thread_tx, release_blocking_thread_rx) = channel::<()>();
            let blocking_thread_occupant =
                spawn_blocking(move || release_blocking_thread_rx.recv());
            let mut model_load = pin!(LoadedLlamaModel::load_unless_shutting_down(
                &agent_shutdown,
                agent_inference_runtime_context(1),
                LlamaModelSettings {
                    model_path: kev_0_8b_target_path("model.gguf"),
                    n_gpu_layers: ALL_GPU_LAYERS,
                },
            ));

            assert!(model_load.as_mut().now_or_never().is_none());

            agent_shutdown.cancel();
            release_blocking_thread_tx
                .send(())
                .expect("the blocking thread occupant must be waiting");
            blocking_thread_occupant
                .await
                .expect("the blocking thread occupant must not panic")
                .expect("the blocking thread occupant must be released");

            assert_eq!(
                discriminant(
                    &model_load
                        .await
                        .expect("the model must load before the shutdown is noticed")
                ),
                discriminant(&StartupOutcome::Cancelled)
            );
        });
}
