#![cfg(feature = "tests_that_use_llms")]

use paddler_tests::typesafe_tone_and_payment_decision::typesafe_tone_and_payment_decision;

#[tokio::test(flavor = "multi_thread")]
async fn typesafe_system_one_echoes_the_model_it_was_asked_for() {
    let response = typesafe_tone_and_payment_decision()
        .await
        .expect("the System One request must be answered");

    assert_eq!(response.body["model"], "kev-latest");
}
