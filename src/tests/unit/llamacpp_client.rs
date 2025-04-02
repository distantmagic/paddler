// use httpmock::{Method::GET, MockServer};
// use serde_json::json;

// use crate::{errors::result::Result, llamacpp::llamacpp_client::LlamacppClient};

// #[tokio::test]
// async fn slots_are_authorized() -> Result<()> {
//     let mock_server = MockServer::start();
//     let client = LlamacppClient::new(*mock_server.address(), None)?;

//     let _mock = mock_server.mock(|when, then| {
//         when.method(GET).path("/slots");
//         then.status(200)
//             .header("content-type", "application/json")
//             .json_body(json!([
//                 {
//                     "id": 0,
//                     "is_processing": false,
//                     "prompt": "",
//                 }
//             ]));
//     });

//     let response = client.get_available_slots().await?;

//     assert!(response.is_authorized.unwrap());
//     assert!(response.is_slot_endpoint_enabled.unwrap());
//     assert!(!response.slots.is_empty());

//     Ok(())
// }

// #[tokio::test]
// async fn slots_are_unauthorized() -> Result<()> {
//     let mock_server = MockServer::start();
//     let client = LlamacppClient::new(*mock_server.address(), None)?;

//     let _mock = mock_server.mock(|when, then| {
//         when.method(GET).path("/slots");
//         then.status(401)
//             .header("content-type", "application/json")
//             .json_body(json!([
//                 {
//                     "id": 0,
//                     "is_processing": false,
//                     "prompt": "",
//                 }
//             ]));
//     });

//     let response = client.get_available_slots().await?;

//     assert!(!response.is_authorized.unwrap());
//     assert!(response.is_slot_endpoint_enabled.is_none());
//     assert!(response.slots.is_empty());

//     Ok(())
// }

// #[tokio::test]
// async fn response_is_unimplemented() -> Result<()> {
//     let mock_server = MockServer::start();
//     let client = LlamacppClient::new(*mock_server.address(), None)?;

//     let _mock = mock_server.mock(|when, then| {
//         when.method(GET).path("/slots");
//         then.status(501)
//             .header("content-type", "application/json")
//             .json_body(json!([
//                 {
//                     "id": 0,
//                     "is_processing": false,
//                     "prompt": "",
//                 }
//             ]));
//     });

//     let response = client.get_available_slots().await?;

//     assert!(response.is_authorized.is_none());
//     assert!(!response.is_slot_endpoint_enabled.unwrap());
//     assert!(response.slots.is_empty());

//     Ok(())
// }

// #[tokio::test]
// async fn response_is_error() -> Result<()> {
//     let mock_server = MockServer::start();
//     let client = LlamacppClient::new(*mock_server.address(), None)?;

//     let _mock = mock_server.mock(|when, then| {
//         when.method(GET).path("/slots");
//         then.status(99);
//     });

//     let error = client.get_available_slots().await.err();

//     assert!(error.is_some());

//     Ok(())
// }
