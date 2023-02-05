#[macro_use]
extern crate log;

use std::error::Error;

use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::cli::Cli;
use crate::cluster_meta_emitter::ClusterMetaEmitter;

mod cli;
mod cluster_meta_emitter;
mod kafka_types;
mod logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = parse_cli_and_init_logging();

    let shutdown_rx = build_shutdown_channel();

    let cluster_meta_emitter = ClusterMetaEmitter::new(cli.build_client_config());

    let (mut cluster_meta_rx, cluster_meta_emitter_handle) = cluster_meta_emitter.spawn(shutdown_rx);
    let receiver_handle = tokio::spawn(async move {
        while let Some(cluster_meta) = cluster_meta_rx.recv().await {
            println!("{:#?}", cluster_meta);
        }
    });

    cluster_meta_emitter_handle.await?;
    receiver_handle.await?;

    Ok(())
}

fn parse_cli_and_init_logging() -> Cli {
    // Parse command line input and initialize logging
    let cli = Cli::parse_and_validate();
    logging::init(cli.verbosity_level());

    trace!("Created:\n{:#?}", cli);

    cli
}

fn build_shutdown_channel() -> broadcast::Receiver<()> {
    let (sender, receiver) = broadcast::channel(1);

    // Setup shutdown signal handler:
    // when it's time to shutdown, broadcast to all receiver a unit.
    //
    // NOTE: This handler will be listening on its own dedicated thread.
    if let Err(e) = ctrlc::set_handler(move || {
        info!("Shutting down...");
        sender.send(()).unwrap();
    }) {
        error!("Failed to register signal handler: {e}");
    }

    // Return a receiver to we can notify other parts of the system.
    receiver
}