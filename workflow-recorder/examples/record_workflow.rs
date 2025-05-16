use workflow_recorder::{WorkflowRecorder, WorkflowRecorderConfig};
use std::path::PathBuf;
use tokio::signal::ctrl_c;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Workflow Recorder Example");
    info!("Press Ctrl+C to stop recording");
    
    // Create a default configuration
    let config = WorkflowRecorderConfig::default();
    
    // Create a recorder
    let mut recorder = WorkflowRecorder::new("Example Workflow".to_string(), config);
    
    // Start recording
    recorder.start().await?;
    
    // Wait for Ctrl+C
    info!("Recording started. Interact with your desktop...");
    ctrl_c().await?;
    
    // Stop recording
    info!("Stopping recording...");
    recorder.stop().await?;
    
    // Save the recording
    let output_path = PathBuf::from("workflow_recording.json");
    recorder.save(&output_path)?;
    info!("Recording saved to {:?}", output_path);
    
    Ok(())
} 