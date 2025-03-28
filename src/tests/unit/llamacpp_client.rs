use cucumber::{given, then, when, World};
use httpmock::{Method::GET, MockServer};
use serde_json::json;

use crate::{
    errors::{app_error::AppError, result::Result},
    llamacpp::{llamacpp_client::LlamacppClient, slots_response::SlotsResponse},
};

#[derive(Default)]
struct Mock(Option<MockServer>);

impl std::fmt::Debug for Mock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockServer")
    }
}

#[derive(Debug, Default, cucumber::World)]
struct LlamacppClientWorld {
    pub mock: Mock,
    pub response: SlotsResponse,
    pub error: Option<AppError>,
    pub client: LlamacppClient,
}

#[given(regex = r"llamacpp server is running")]
async fn setup_llamacpp_server(world: &mut LlamacppClientWorld) -> Result<()> {
    let mock_server = MockServer::start();
    world.client = LlamacppClient::new(*mock_server.address(), None)?;

    world.mock.0 = Some(mock_server);

    Ok(())
}

#[when("I request available slots with a authorized response")]
async fn request_slots_success(world: &mut LlamacppClientWorld) -> Result<()> {
    let mock_server = world.mock.0.as_ref().unwrap();
    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([
                {
                    "id": 0,
                    "is_processing": false,
                    "prompt": "",
                }
            ]));
    });

    world.response = world.client.get_available_slots().await?;
    world.error = None;

    Ok(())
}

#[when("I request available slots with an unauthorized response")]
async fn request_slots_failure(world: &mut LlamacppClientWorld) -> Result<()> {
    let mock_server = world.mock.0.as_ref().unwrap();
    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(401)
            .header("content-type", "application/json")
            .json_body(json!([
                {
                    "id": 0,
                    "is_processing": false,
                    "prompt": "",
                }
            ]));
    });

    world.response = world.client.get_available_slots().await?;
    world.error = None;

    Ok(())
}

#[when("I request available slots with a not implemented response")]
async fn request_slots_not_implemented(world: &mut LlamacppClientWorld) -> Result<()> {
    let mock_server = world.mock.0.as_ref().unwrap();
    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(501)
            .header("content-type", "application/json")
            .json_body(json!([
                {
                    "id": 0,
                    "is_processing": false,
                    "prompt": "",
                }
            ]));
    });

    world.response = world.client.get_available_slots().await?;
    world.error = None;

    Ok(())
}

#[when("I request available slots with an error response")]
async fn request_slots_error(world: &mut LlamacppClientWorld) -> Result<()> {
    let mock_server = world.mock.0.as_ref().unwrap();

    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(99);
    });

    world.error = world.client.get_available_slots().await.err();

    Ok(())
}

#[then("I must receive a successful response with slots")]
async fn verify_successful_response(world: &mut LlamacppClientWorld) {
    let response = &world.response;
    assert!(response.is_authorized.unwrap());
    assert!(response.is_slot_endpoint_enabled.unwrap());
    assert!(!response.slots.is_empty());
}

#[then("I must receive an unauthorized response")]
async fn verify_unauthorized_response(world: &mut LlamacppClientWorld) {
    let response = &world.response;

    assert!(!response.is_authorized.unwrap());
    assert!(response.is_slot_endpoint_enabled.is_none());
    assert!(response.slots.is_empty());
}

#[then("I must receive a not implemented response")]
async fn verify_not_implemented_response(world: &mut LlamacppClientWorld) {
    let response = &world.response;
    assert!(response.is_authorized.is_none());
    assert!(!response.is_slot_endpoint_enabled.unwrap());
    assert!(response.slots.is_empty());
}

#[then("I must receive an error")]
async fn verify_error_response(world: &mut LlamacppClientWorld) {
    assert!(world.error.is_some());
}

#[cfg(not(miri))]
#[tokio::test]
async fn run_cucumber_tests() {
    LlamacppClientWorld::run("src/tests/features/llamacpp_client.feature").await;
}
