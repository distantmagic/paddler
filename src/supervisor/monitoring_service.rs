// use actix_web::web::Bytes;
// use async_trait::async_trait;
// use log::{debug, error};
// use pingora::{server::ShutdownWatch, services::Service};
// use tokio::{
//     sync::broadcast::Sender,
//     time::{interval, Duration, MissedTickBehavior},
// };

// #[cfg(unix)]
// use pingora::server::ListenFds;

// use crate::{
//     balancer::status_update::StatusUpdate, errors::result::Result,
//     llamacpp::llamacpp_client::LlamacppClient,
// };

// pub struct SupervisingService {
//     monitoring_interval: Duration,
//     llamacpp_client: LlamacppClient,
//     status_update_tx: Sender<Bytes>,
// }

// impl SupervisingService {
//     pub fn new(
//         monitoring_interval: Duration,
//         llamacpp_client: LlamacppClient,
//         status_update_tx: Sender<Bytes>,
//     ) -> Result<Self> {
//         Ok(SupervisingService {
//             monitoring_interval,
//             llamacpp_client,
//             status_update_tx,
//         })
//     }

//     async fn fetch_status(&self) -> Result<StatusUpdate> {
//         match self.llamacpp_client.get_available_slots().await {
//             Ok(slots_response) => Ok(StatusUpdate::new(
//                 self.name.to_owned(),
//                 None,
//                 self.local_llamacpp_addr.to_owned(),
//                 slots_response.is_authorized,
//                 slots_response.is_slot_endpoint_enabled,
//                 slots_response.slots,
//             )),
//             Err(err) => Ok(StatusUpdate::new(
//                 self.name.to_owned(),
//                 Some(err.to_string()),
//                 self.local_llamacpp_addr.to_owned(),
//                 None,
//                 None,
//                 vec![],
//             )),
//         }
//     }

//     async fn report_status(&self, status: StatusUpdate) -> Result<usize> {
//         let status = Bytes::from(serde_json::to_vec(&status)?);

//         Ok(self.status_update_tx.send(status)?)
//     }
// }

// #[async_trait]
// impl Service for SupervisingService {
//     async fn start_service(
//         &mut self,
//         #[cfg(unix)] _fds: Option<ListenFds>,
//         mut shutdown: ShutdownWatch,
//     ) {
//         let mut ticker = interval(self.monitoring_interval);

//         ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

//         loop {
//             tokio::select! {
//                 _ = shutdown.changed() => {
//                     debug!("Shutting down monitoring service");
//                     return;
//                 },
//                 _ = ticker.tick() => {
//                     match self.fetch_status().await {
//                         Err(err) => {
//                             error!("Failed to fetch status: {}", err);
//                         },
//                         _ => ()
//                     }
//                 }
//             }
//         }
//     }

//     fn name(&self) -> &str {
//         "monitoring"
//     }

//     fn threads(&self) -> Option<usize> {
//         Some(1)
//     }
// }
