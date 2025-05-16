# Workflow Recorder

A Rust crate for recording user interactions with the desktop UI, designed for capturing and replaying workflows.

## Features

- Records mouse events (clicks, movements)
- Records keyboard events
- Captures UI element information (using Windows UI Automation)
- Exports recordings to JSON format
- Cross-platform design (currently Windows-only implementation)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
workflow-recorder = { path = "path/to/workflow-recorder" }
```

### Example

```rust
use workflow_recorder::{WorkflowRecorder, WorkflowRecorderConfig};
use std::path::PathBuf;
use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a default configuration
    let config = WorkflowRecorderConfig::default();
    
    // Create a recorder
    let mut recorder = WorkflowRecorder::new("Example Workflow".to_string(), config);
    
    // Start recording
    recorder.start().await?;
    
    // Wait for Ctrl+C
    println!("Recording started. Press Ctrl+C to stop...");
    ctrl_c().await?;
    
    // Stop recording
    recorder.stop().await?;
    
    // Save the recording
    let output_path = PathBuf::from("workflow_recording.json");
    recorder.save(&output_path)?;
    println!("Recording saved to {:?}", output_path);
    
    Ok(())
}
```

## Running the Example

```bash
cargo run --example record_workflow
```

Press `Ctrl+C` to stop recording. The workflow will be saved to `workflow_recording.json`.

## Output Format

The recorded workflow is saved as a JSON file with the following structure:

```json
{
  "name": "Example Workflow",
  "start_time": 1621234567890,
  "end_time": 1621234598765,
  "events": [
    {
      "timestamp": 1621234568000,
      "event": {
        "Mouse": {
          "event_type": "Click",
          "button": "Left",
          "position": {
            "x": 100,
            "y": 200
          },
          "ui_element": {
            "name": "Button",
            "automation_id": "button1",
            "class_name": "Button",
            "control_type": "Button",
            "process_id": 1234,
            "application_name": "example.exe",
            "window_title": "Example Window"
          }
        }
      }
    },
    {
      "timestamp": 1621234569000,
      "event": {
        "Keyboard": {
          "key_code": 65,
          "is_key_down": true,
          "ctrl_pressed": false,
          "alt_pressed": false,
          "shift_pressed": false,
          "win_pressed": false
        }
      }
    }
  ]
}
```

## Platform Support

Currently, this crate only supports Windows. The architecture is designed to be cross-platform, but implementations for macOS and Linux are not yet available.
