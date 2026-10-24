use serde_json::Value;
use serde_json::json;

#[must_use]
pub fn weather_tool_parameters_with_an_integer_location() -> Value {
    json!({
        "type": "object",
        "properties": {
            "location": {"type": "integer", "description": "The city name"}
        },
        "required": ["location"],
        "additionalProperties": false
    })
}
