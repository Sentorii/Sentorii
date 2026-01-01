use crate::cli::{Cli, Commands, FeatureCommands};
use anyhow::Result;
use sentorii_contracts::workflow_request::WorkflowRequest;
use tokio::sync::mpsc::Sender;

pub fn dispatch(cli: Cli, request_tx: &Sender<WorkflowRequest>) -> Result<()> {
    let request = match cli.command {
        Commands::Feature(feature) => match feature.command {
            FeatureCommands::Start => WorkflowRequest::StartFeature { branch_name: None },
        },
    };

    let sent = request_tx.send(request);
    drop(sent);

    Ok(())
}
