use serde_json::Error as JsonError;
use serde_json::Value;
use serde_json::from_value;
use serde_json::json;

use paddler_typesafe_translation::system_one_request::SystemOneRequest;

pub fn system_one_request_with_question(question: Value) -> Result<SystemOneRequest, JsonError> {
    from_value(json!({"state": "state", "questions": {"question": question}}))
}
