//! Desktop UI automation through accessibility APIs
//!
//! This module provides a cross-platform API for automating desktop applications
//! through accessibility APIs, inspired by Playwright's web automation model.

use std::sync::Arc;

mod element;
mod errors;
mod locator;
pub mod platforms;
mod selector;
#[cfg(test)]
mod tests;

pub use element::{UIElement, UIElementAttributes};
pub use errors::AutomationError;
pub use locator::Locator;
pub use selector::Selector;

// Define a new struct to hold click result information - move to module level
pub struct ClickResult {
    pub method: String,
    pub coordinates: Option<(f64, f64)>,
    pub details: String,
}

/// Holds the output of a terminal command execution
pub struct CommandOutput {
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Holds the screenshot data
#[derive(Debug, Clone)]
pub struct ScreenshotResult {
    /// Raw image data (e.g., RGBA)
    pub image_data: Vec<u8>,
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
}

/// The main entry point for UI automation
pub struct Desktop {
    engine: Arc<dyn platforms::AccessibilityEngine>,
}

impl Desktop {
    /// Create a new instance with the default platform-specific implementation
    pub fn new(use_background_apps: bool, activate_app: bool) -> Result<Self, AutomationError> {
        let boxed_engine = platforms::create_engine(use_background_apps, activate_app)?;
        // Move the boxed engine into an Arc
        let engine = Arc::from(boxed_engine);
        Ok(Self { engine })
    }

    /// Get the root UI element representing the entire desktop
    pub fn root(&self) -> UIElement {
        self.engine.get_root_element()
    }

    /// Create a locator to find elements matching the given selector
    pub fn locator(&self, selector: impl Into<Selector>) -> Locator {
        Locator::new(Arc::clone(&self.engine), selector.into())
    }

    /// Get the currently focused element
    pub fn focused_element(&self) -> Result<UIElement, AutomationError> {
        self.engine.get_focused_element()
    }

    /// List all running applications
    pub fn applications(&self) -> Result<Vec<UIElement>, AutomationError> {
        self.engine.get_applications()
    }

    /// Find an application by name
    pub fn application(&self, name: &str) -> Result<UIElement, AutomationError> {
        self.engine.get_application_by_name(name)
    }

    /// Open an application by name
    pub fn open_application(&self, app_name: &str) -> Result<UIElement, AutomationError> {
        self.engine.open_application(app_name)
    }

    /// Open an application by name
    pub fn activate_application(&self, app_name: &str) -> Result<(), AutomationError> {
        self.engine.get_application_by_name(app_name)?.activate_window()
    }


    /// Open a URL in a specified browser (or default browser if None)
    pub fn open_url(&self, url: &str, browser: Option<&str>) -> Result<UIElement, AutomationError> {
        self.engine.open_url(url, browser)
    }

    /// Open a file with its default application
    pub fn open_file(&self, file_path: &str) -> Result<(), AutomationError> {
        self.engine.open_file(file_path)
    }

    /// Execute a terminal command
    ///
    /// Provide the appropriate command string for the target OS family.
    /// At least one command must be provided.
    pub fn run_command(
        &self,
        windows_command: Option<&str>,
        unix_command: Option<&str>,
    ) -> Result<CommandOutput, AutomationError> {
        self.engine.run_command(windows_command, unix_command)
    }

    /// Capture a screenshot of the primary monitor
    pub fn capture_screen(&self) -> Result<ScreenshotResult, AutomationError> {
        self.engine.capture_screen()
    }

    /// Capture a screenshot of a specific monitor by name
    pub fn capture_monitor_by_name(&self, name: &str) -> Result<ScreenshotResult, AutomationError> {
        self.engine.capture_monitor_by_name(name)
    }

    /// Perform OCR on the specified image file
    ///
    /// This function requires an active Tokio runtime.
    pub async fn ocr_image_path(&self, image_path: &str) -> Result<String, AutomationError> {
        self.engine.ocr_image_path(image_path).await
    }

    /// Perform OCR on the provided screenshot data
    ///
    /// This function requires an active Tokio runtime.
    pub async fn ocr_screenshot(&self, screenshot: &ScreenshotResult) -> Result<String, AutomationError> {
        self.engine.ocr_screenshot(screenshot).await
    }

    /// Activate a browser window containing a specific title.
    /// Brings the browser window to the foreground if found.
    pub fn activate_browser_window_by_title(&self, title: &str) -> Result<(), AutomationError> {
        self.engine.activate_browser_window_by_title(title)
    }
}
