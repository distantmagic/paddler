use serde_json::from_value;
use serde_json::json;

use paddler_messaging::request_params_validation_error::RequestParamsValidationError;
use paddler_typesafe_translation::system_one_request::SystemOneRequest;
use paddler_typesafe_translation::typesafe_translation_error::TypeSafeTranslationError;

#[test]
fn a_system_one_request_without_questions_is_refused() {
    let request: SystemOneRequest =
        from_value(json!({"state": "state", "questions": {}})).expect("the request must parse");

    assert_eq!(
        request.translate().err().map(|error| error.to_string()),
        Some(
            TypeSafeTranslationError::DecisionInvalid {
                source: RequestParamsValidationError::DecisionWithoutQuestions,
            }
            .to_string()
        )
    );
}
