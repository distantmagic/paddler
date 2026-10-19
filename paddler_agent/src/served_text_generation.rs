use std::sync::Arc;

use paddler_agent_runtime::scheduler_handle::SchedulerHandle;
use paddler_agent_text_generation::generation_request_preparer::GenerationRequestPreparer;

pub type ServedTextGeneration = Arc<SchedulerHandle<GenerationRequestPreparer>>;
