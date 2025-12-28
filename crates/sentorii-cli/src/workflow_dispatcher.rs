use tokio::sync::mpsc::Sender;
use sentorii_contracts::workflow_request::WorkflowRequest;
use crate::cli::{Cli, Commands, FeatureCommands};
use anyhow::Result;

pub fn dispatch(cli: Cli, request_tx: &Sender<WorkflowRequest>) -> Result<()> {
    let request = match cli.command {
        Commands::Feature(feature) => match feature.command {
            FeatureCommands::Start => WorkflowRequest::StartFeature { branch_name: None },
        },
    };

    let _ = request_tx.send(request);

    Ok(())
}