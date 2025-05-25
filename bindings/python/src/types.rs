use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use std::collections::HashMap;
use ::terminator_core::{
    ScreenshotResult as CoreScreenshotResult,
    ClickResult as CoreClickResult,
    CommandOutput as CoreCommandOutput,
};

/// Result of a screenshot operation.
#[gen_stub_pyclass]
#[pyclass(name = "ScreenshotResult")]
pub struct ScreenshotResult {
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    #[pyo3(get)]
    pub image_data: Vec<u8>,
}

/// Result of a click operation.
#[gen_stub_pyclass]
#[pyclass(name = "ClickResult")]
pub struct ClickResult {
    #[pyo3(get)]
    pub method: String,
    #[pyo3(get)]
    pub coordinates: Option<Coordinates>,
    #[pyo3(get)]
    pub details: String,
}

/// Result of a command execution.
#[gen_stub_pyclass]
#[pyclass(name = "CommandOutput")]
pub struct CommandOutput {
    #[pyo3(get)]
    pub exit_status: Option<i32>,
    #[pyo3(get)]
    pub stdout: String,
    #[pyo3(get)]
    pub stderr: String,
}

/// UI Element attributes
#[gen_stub_pyclass]
#[pyclass(name = "UIElementAttributes")]
pub struct UIElementAttributes {
    #[pyo3(get)]
    pub role: String,
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub label: Option<String>,
    #[pyo3(get)]
    pub value: Option<String>,
    #[pyo3(get)]
    pub description: Option<String>,
    #[pyo3(get)]
    pub properties: HashMap<String, Option<String>>,
    #[pyo3(get)]
    pub is_keyboard_focusable: Option<bool>,
}

/// Coordinates for mouse operations
#[gen_stub_pyclass]
#[pyclass(name = "Coordinates")]
#[derive(Clone)]
pub struct Coordinates {
    #[pyo3(get)]
    pub x: f64,
    #[pyo3(get)]
    pub y: f64,
}

/// Bounds for element coordinates
#[gen_stub_pyclass]
#[pyclass(name = "Bounds")]
pub struct Bounds {
    #[pyo3(get)]
    pub x: f64,
    #[pyo3(get)]
    pub y: f64,
    #[pyo3(get)]
    pub width: f64,
    #[pyo3(get)]
    pub height: f64,
}

/// Run command options
#[gen_stub_pyclass]
#[pyclass(name = "RunCommandOptions")]
pub struct RunCommandOptions {
    #[pyo3(get)]
    pub windows_command: Option<String>,
    #[pyo3(get)]
    pub unix_command: Option<String>,
}

impl From<CoreScreenshotResult> for ScreenshotResult {
    fn from(r: CoreScreenshotResult) -> Self {
        ScreenshotResult {
            width: r.width,
            height: r.height,
            image_data: r.image_data,
        }
    }
}

impl From<CoreClickResult> for ClickResult {
    fn from(r: CoreClickResult) -> Self {
        ClickResult {
            method: r.method,
            coordinates: r.coordinates.map(|(x, y)| Coordinates { x, y }),
            details: r.details,
        }
    }
}

impl From<CoreCommandOutput> for CommandOutput {
    fn from(r: CoreCommandOutput) -> Self {
        CommandOutput {
            exit_status: r.exit_status,
            stdout: r.stdout,
            stderr: r.stderr,
        }
    }
} 