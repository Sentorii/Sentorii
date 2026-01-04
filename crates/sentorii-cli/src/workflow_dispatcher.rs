use crate::cli::{Cli, Commands, FeatureCommands};
use anyhow::Result;
use log::info;
use sentorii_contracts::workflow_request::WorkflowRequest;
use tokio::sync::mpsc::Sender;

/// Dispatch workflow request to the engine.
///
/// # Errors
///
/// If the receive half of the channel is closed, either due to close being called or
/// the Receiver handle dropping, the function returns an error.
/// The error includes the value passed to send.
pub async fn dispatch(cli: Cli, request_tx: &Sender<WorkflowRequest>) -> Result<()> {
    let request = match cli.command {
        Commands::Feature(feature) => match feature.command {
            FeatureCommands::Start => WorkflowRequest::StartFeature { branch_name: None },
        },
    };

    info!("Dispatching workflow request");

    request_tx.send(request).await?;

    Ok(())
}
