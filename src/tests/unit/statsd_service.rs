// use std::{net::UdpSocket, sync::Arc, time::Duration};

// use cadence::{BufferedUdpMetricSink, StatsdClient};
// use httpmock::{Method::GET, MockServer};

// use crate::{
//     balancer::{statsd_service::StatsdService, upstream_peer_pool::UpstreamPeerPool},
//     errors::result::Result,
// };

// #[tokio::test]
// async fn metrics_are_reported() -> Result<()> {
//     let mock_server = MockServer::start();

//     let mock = mock_server.mock(|when, then| {
//         when.method(GET).path("/");
//         then.status(200)
//             .header("content-type", "application/json")
//             .body("OK");
//     });

//     let upstream_peer_pool = UpstreamPeerPool::new();

//     let statsd_service = StatsdService::new(
//         *mock.server_address(),
//         "paddler".to_string(),
//         Duration::from_secs(10),
//         Arc::new(upstream_peer_pool),
//     )?;

//     let statsd_sink_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
//     let statsd_sink = BufferedUdpMetricSink::from(*mock.server_address(), statsd_sink_socket)
//         .expect("Failed to create statsd sink");

//     let client = StatsdClient::builder("paddler", statsd_sink).build();

//     let response = statsd_service.report_metrics(&client).await;

//     assert!(response.is_ok());

//     Ok(())
// }
