// Inner module
mod emitter;

use rdkafka::ClientConfig;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::internals::Emitter;

pub use emitter::{ConsumerGroups, ConsumerGroupsEmitter};

pub fn init(
    admin_client_config: ClientConfig,
    shutdown_token: CancellationToken,
) -> (Receiver<ConsumerGroups>, JoinHandle<()>) {
    let consumer_groups_emitter = ConsumerGroupsEmitter::new(admin_client_config);
    let (cg_rx, cg_join) = consumer_groups_emitter.spawn(shutdown_token);

    debug!("Initialized");
    (cg_rx, cg_join)
}
