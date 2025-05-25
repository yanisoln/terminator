#![allow(non_local_definitions)]
#![allow(unsafe_op_in_unsafe_fn)]

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3_asyncio_0_21::tokio as pyo3_tokio;
use pyo3_stub_gen::{create_exception, define_stub_info_gatherer, derive::*};
use ::terminator_core::{
    Desktop as TerminatorDesktop,
    element::UIElement as TerminatorUIElement,
    ScreenshotResult as CoreScreenshotResult,
    ClickResult as CoreClickResult,
    CommandOutput as CoreCommandOutput,
    locator::Locator as TerminatorLocator,
    errors::AutomationError
};
use std::sync::Once;
use pyo3::IntoPy;
use pyo3::types::PyAny;
use std::collections::HashMap;

/// Main entry point for desktop automation.
#[gen_stub_pyclass]
#[pyclass(name = "Desktop")]
pub struct Desktop {
    inner: TerminatorDesktop,
}

/// Represents a UI element in the desktop UI tree.
#[gen_stub_pyclass]
#[pyclass(name = "UIElement")]
#[derive(Clone)]
pub struct UIElement {
    inner: TerminatorUIElement,
}

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

/// Locator for finding UI elements by selector.
#[gen_stub_pyclass]
#[pyclass(name = "Locator")]
pub struct Locator {
    inner: TerminatorLocator,
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

// Custom Python exceptions for advanced error mapping
create_exception!(terminator, ElementNotFoundError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, TimeoutError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, PermissionDeniedError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, PlatformError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, UnsupportedOperationError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, UnsupportedPlatformError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, InvalidArgumentError, pyo3::exceptions::PyRuntimeError);
create_exception!(terminator, InternalError, pyo3::exceptions::PyRuntimeError);

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

// Advanced error mapping
fn automation_error_to_pyerr(e: AutomationError) -> pyo3::PyErr {
    let msg = format!("{e}");
    match e {
        AutomationError::ElementNotFound(_) => ElementNotFoundError::new_err(msg),
        AutomationError::Timeout(_) => TimeoutError::new_err(msg),
        AutomationError::PermissionDenied(_) => PermissionDeniedError::new_err(msg),
        AutomationError::PlatformError(_) => PlatformError::new_err(msg),
        AutomationError::UnsupportedOperation(_) => UnsupportedOperationError::new_err(msg),
        AutomationError::UnsupportedPlatform(_) => UnsupportedPlatformError::new_err(msg),
        AutomationError::InvalidArgument(_) => InvalidArgumentError::new_err(msg),
        AutomationError::Internal(_) => InternalError::new_err(msg),
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Desktop {
    #[new]
    #[pyo3(text_signature = "()")]
    /// Create a new Desktop automation instance with default settings.
    /// 
    /// Returns:
    ///     Desktop: A new Desktop automation instance.
    pub fn new() -> PyResult<Self> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init();
        });
        let desktop = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            .block_on(TerminatorDesktop::new(false, false))
            .map_err(|e| automation_error_to_pyerr(e))?;
        Ok(Desktop { inner: desktop })
    }

    #[staticmethod]
    #[pyo3(text_signature = "()")]
    /// Create a new Desktop automation instance with background apps enabled.
    /// 
    /// Returns:
    ///     Desktop: A new Desktop automation instance with background apps enabled.
    pub fn with_background_apps() -> PyResult<Self> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init();
        });
        let desktop = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            .block_on(TerminatorDesktop::new(true, false))
            .map_err(|e| automation_error_to_pyerr(e))?;
        Ok(Desktop { inner: desktop })
    }

    #[staticmethod]
    #[pyo3(text_signature = "()")]
    /// Create a new Desktop automation instance with app activation enabled.
    /// 
    /// Returns:
    ///     Desktop: A new Desktop automation instance with app activation enabled.
    pub fn with_app_activation() -> PyResult<Self> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init();
        });
        let desktop = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            .block_on(TerminatorDesktop::new(false, true))
            .map_err(|e| automation_error_to_pyerr(e))?;
        Ok(Desktop { inner: desktop })
    }

    #[staticmethod]
    #[pyo3(text_signature = "()")]
    /// Create a new Desktop automation instance with both background apps and app activation enabled.
    /// 
    /// Returns:
    ///     Desktop: A new Desktop automation instance with all features enabled.
    pub fn with_all_features() -> PyResult<Self> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init();
        });
        let desktop = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            .block_on(TerminatorDesktop::new(true, true))
            .map_err(|e| automation_error_to_pyerr(e))?;
        Ok(Desktop { inner: desktop })
    }

    #[pyo3(text_signature = "($self)")]
    /// Get the root UI element of the desktop.
    /// 
    /// Returns:
    ///     UIElement: The root UI element.
    pub fn root(&self) -> PyResult<UIElement> {
        let root = self.inner.root();
        Ok(UIElement { inner: root })
    }

    #[pyo3(text_signature = "($self)")]
    /// Get a list of all running applications.
    /// 
    /// Returns:
    ///     List[UIElement]: List of application UI elements.
    pub fn applications(&self) -> PyResult<Vec<UIElement>> {
        self.inner.applications()
            .map(|apps| apps.into_iter().map(|e| UIElement { inner: e }).collect())
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(text_signature = "($self, name)")]
    /// Get a running application by name.
    /// 
    /// Args:
    ///     name (str): The name of the application to find.
    /// 
    /// Returns:
    ///     UIElement: The application UI element.
    pub fn application(&self, name: &str) -> PyResult<UIElement> {
        self.inner.application(name)
            .map(|e| UIElement { inner: e })
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "open_application", text_signature = "($self, name)")]
    /// Open an application by name.
    /// 
    /// Args:
    ///     name (str): The name of the application to open.
    pub fn open_application(&self, name: &str) -> PyResult<()> {
        self.inner.open_application(name)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "activate_application", text_signature = "($self, name)")]
    /// Activate an application by name.
    /// 
    /// Args:
    ///     name (str): The name of the application to activate.
    pub fn activate_application(&self, name: &str) -> PyResult<()> {
        self.inner.activate_application(name)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "locator", text_signature = "($self, selector)")]
    /// Create a locator for finding UI elements.
    /// 
    /// Args:
    ///     selector (str): The selector string to find elements.
    /// 
    /// Returns:
    ///     Locator: A locator for finding elements.
    pub fn locator(&self, selector: &str) -> PyResult<Locator> {
        let locator = self.inner.locator(selector);
        Ok(Locator { inner: locator })
    }

    #[pyo3(name = "capture_screen", text_signature = "($self)")]
    /// Capture a screenshot of the primary monitor.
    /// 
    /// Returns:
    ///     ScreenshotResult: The screenshot data.
    pub fn capture_screen<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.capture_screen().await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = ScreenshotResult::from(result);
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "run_command", text_signature = "($self, windows_command, unix_command)")]
    /// Run a shell command.
    /// 
    /// Args:
    ///     windows_command (Optional[str]): Command to run on Windows.
    ///     unix_command (Optional[str]): Command to run on Unix.
    /// 
    /// Returns:
    ///     CommandOutput: The command output.
    pub fn run_command<'py>(&self, py: Python<'py>, windows_command: Option<String>, unix_command: Option<String>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.run_command(windows_command.as_deref(), unix_command.as_deref()).await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = CommandOutput::from(result);
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "capture_monitor_by_name", text_signature = "($self, name)")]
    /// Capture a screenshot of a specific monitor.
    /// 
    /// Args:
    ///     name (str): The name of the monitor to capture.
    /// 
    /// Returns:
    ///     ScreenshotResult: The screenshot data.
    pub fn capture_monitor_by_name<'py>(&self, py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let name = name.to_string();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.capture_monitor_by_name(&name).await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = ScreenshotResult::from(result);
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "ocr_image_path", text_signature = "($self, image_path)")]
    /// Perform OCR on an image file.
    /// 
    /// Args:
    ///     image_path (str): Path to the image file.
    /// 
    /// Returns:
    ///     str: The extracted text.
    pub fn ocr_image_path<'py>(&self, py: Python<'py>, image_path: &str) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let image_path = image_path.to_string();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.ocr_image_path(&image_path).await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                Ok(result.into_py(py))
            })
        })
    }

    #[pyo3(name = "ocr_screenshot", text_signature = "($self, screenshot)")]
    /// Perform OCR on a screenshot.
    /// 
    /// Args:
    ///     screenshot (ScreenshotResult): The screenshot to process.
    /// 
    /// Returns:
    ///     str: The extracted text.
    pub fn ocr_screenshot<'py>(&self, py: Python<'py>, screenshot: &ScreenshotResult) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let core_screenshot = CoreScreenshotResult {
            image_data: screenshot.image_data.clone(),
            width: screenshot.width,
            height: screenshot.height,
        };
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.ocr_screenshot(&core_screenshot).await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                Ok(result.into_py(py))
            })
        })
    }

    #[pyo3(name = "find_window_by_criteria", text_signature = "($self, title_contains, timeout_ms)")]
    /// Find a window by criteria.
    /// 
    /// Args:
    ///     title_contains (Optional[str]): Text that should be in the window title.
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     UIElement: The found window element.
    pub fn find_window_by_criteria<'py>(&self, py: Python<'py>, title_contains: Option<&str>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        let title_contains = title_contains.map(|s| s.to_string());
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.find_window_by_criteria(title_contains.as_deref(), timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = UIElement { inner: result };
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "get_current_browser_window", text_signature = "($self)")]
    /// Get the currently focused browser window.
    /// 
    /// Returns:
    ///     UIElement: The current browser window element.
    pub fn get_current_browser_window<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.get_current_browser_window().await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = UIElement { inner: result };
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "get_current_window", text_signature = "($self)")]
    /// Get the currently focused window.
    /// 
    /// Returns:
    ///     UIElement: The current window element.
    pub fn get_current_window<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.get_current_window().await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = UIElement { inner: result };
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "get_current_application", text_signature = "($self)")]
    /// Get the currently focused application.
    /// 
    /// Returns:
    ///     UIElement: The current application element.
    pub fn get_current_application<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let desktop = self.inner.clone();
        pyo3_tokio::future_into_py(py, async move {
            let result = desktop.get_current_application().await.map_err(|e| automation_error_to_pyerr(e))?;
            Python::with_gil(|py| {
                let py_result = UIElement { inner: result };
                Ok(py_result.into_py(py))
            })
        })
    }

    #[pyo3(name = "open_url", text_signature = "($self, url, browser)")]
    /// Open a URL in a browser.
    /// 
    /// Args:
    ///     url (str): The URL to open.
    ///     browser (Optional[str]): The browser to use.
    pub fn open_url(&self, url: &str, browser: Option<&str>) -> PyResult<()> {
        self.inner.open_url(url, browser)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "open_file", text_signature = "($self, file_path)")]
    /// Open a file with its default application.
    /// 
    /// Args:
    ///     file_path (str): Path to the file to open.
    pub fn open_file(&self, file_path: &str) -> PyResult<()> {
        self.inner.open_file(file_path)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "activate_browser_window_by_title", text_signature = "($self, title)")]
    /// Activate a browser window by title.
    /// 
    /// Args:
    ///     title (str): The window title to match.
    pub fn activate_browser_window_by_title(&self, title: &str) -> PyResult<()> {
        self.inner.activate_browser_window_by_title(title)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "focused_element", text_signature = "($self)")]
    /// Get the currently focused element.
    /// 
    /// Returns:
    ///     UIElement: The focused element.
    pub fn focused_element(&self) -> PyResult<UIElement> {
        self.inner.focused_element()
            .map(|e| UIElement { inner: e })
            .map_err(|e| automation_error_to_pyerr(e))
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UIElement {
    /// Get the element's role (e.g., "button", "textfield").
    /// 
    /// Returns:
    ///     str: The element's role.
    pub fn role(&self) -> String {
        self.inner.role()
    }

    /// Get the element's name.
    /// 
    /// Returns:
    ///     Optional[str]: The element's name, if available.
    pub fn name(&self) -> Option<String> {
        self.inner.name()
    }

    /// Get the element's ID.
    /// 
    /// Returns:
    ///     Optional[str]: The element's ID, if available.
    pub fn id(&self) -> Option<String> {
        self.inner.id()
    }

    /// Get all attributes of the element.
    /// 
    /// Returns:
    ///     UIElementAttributes: The element's attributes.
    pub fn attributes(&self) -> PyResult<UIElementAttributes> {
        let attrs = self.inner.attributes();
        Ok(UIElementAttributes {
            role: attrs.role,
            name: attrs.name,
            label: attrs.label,
            value: attrs.value,
            description: attrs.description,
            properties: attrs.properties.into_iter()
                .map(|(k, v)| (k, v.map(|v| v.to_string())))
                .collect(),
            is_keyboard_focusable: attrs.is_keyboard_focusable,
        })
    }

    /// Get child elements.
    /// 
    /// Returns:
    ///     List[UIElement]: List of child elements.
    pub fn children(&self) -> PyResult<Vec<UIElement>> {
        self.inner.children()
            .map(|kids| kids.into_iter().map(|e| UIElement { inner: e }).collect())
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Get parent element.
    /// 
    /// Returns:
    ///     Optional[UIElement]: The parent element, if available.
    pub fn parent(&self) -> PyResult<Option<UIElement>> {
        self.inner.parent()
            .map(|opt| opt.map(|e| UIElement { inner: e }))
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Get element bounds (x, y, width, height).
    /// 
    /// Returns:
    ///     Bounds: The element's bounds.
    pub fn bounds(&self) -> PyResult<Bounds> {
        let (x, y, width, height) = self.inner.bounds().map_err(|e| automation_error_to_pyerr(e))?;
        Ok(Bounds { x, y, width, height })
    }

    /// Click on this element.
    /// 
    /// Returns:
    ///     ClickResult: Result of the click operation.
    pub fn click(&self) -> PyResult<ClickResult> {
        self.inner.click()
            .map(ClickResult::from)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Double click on this element.
    /// 
    /// Returns:
    ///     ClickResult: Result of the click operation.
    pub fn double_click(&self) -> PyResult<ClickResult> {
        self.inner.double_click()
            .map(ClickResult::from)
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Right click on this element.
    /// 
    /// Returns:
    ///     None
    pub fn right_click(&self) -> PyResult<()> {
        self.inner.right_click()
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Hover over this element.
    /// 
    /// Returns:
    ///     None
    pub fn hover(&self) -> PyResult<()> {
        self.inner.hover()
            .map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "is_visible", text_signature = "($self)")]
    /// Check if element is visible.
    /// 
    /// Returns:
    ///     bool: True if the element is visible.
    pub fn is_visible(&self) -> PyResult<bool> {
        self.inner.is_visible().map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "is_enabled", text_signature = "($self)")]
    /// Check if element is enabled.
    /// 
    /// Returns:
    ///     bool: True if the element is enabled.
    pub fn is_enabled(&self) -> PyResult<bool> {
        self.inner.is_enabled().map_err(|e| automation_error_to_pyerr(e))
    }

    /// Focus this element.
    /// 
    /// Returns:
    ///     None
    pub fn focus(&self) -> PyResult<()> {
        self.inner.focus().map_err(|e| automation_error_to_pyerr(e))
    }

    /// Get text content of this element.
    /// 
    /// Args:
    ///     max_depth (Optional[int]): Maximum depth to search for text.
    /// 
    /// Returns:
    ///     str: The element's text content.
    pub fn text(&self, max_depth: Option<usize>) -> PyResult<String> {
        self.inner.text(max_depth.unwrap_or(1)).map_err(|e| automation_error_to_pyerr(e))
    }

    /// Type text into this element.
    /// 
    /// Args:
    ///     text (str): The text to type.
    ///     use_clipboard (Optional[bool]): Whether to use clipboard for pasting.
    /// 
    /// Returns:
    ///     None
    pub fn type_text(&self, text: &str, use_clipboard: Option<bool>) -> PyResult<()> {
        self.inner.type_text(text, use_clipboard.unwrap_or(false)).map_err(|e| automation_error_to_pyerr(e))
    }

    /// Press a key while this element is focused.
    /// 
    /// Args:
    ///     key (str): The key to press.
    /// 
    /// Returns:
    ///     None
    pub fn press_key(&self, key: &str) -> PyResult<()> {
        self.inner.press_key(key).map_err(|e| automation_error_to_pyerr(e))
    }

    /// Set value of this element.
    /// 
    /// Args:
    ///     value (str): The value to set.
    /// 
    /// Returns:
    ///     None
    pub fn set_value(&self, value: &str) -> PyResult<()> {
        self.inner.set_value(value).map_err(|e| automation_error_to_pyerr(e))
    }

    /// Perform a named action on this element.
    /// 
    /// Args:
    ///     action (str): The action to perform.
    /// 
    /// Returns:
    ///     None
    pub fn perform_action(&self, action: &str) -> PyResult<()> {
        self.inner.perform_action(action).map_err(|e| automation_error_to_pyerr(e))
    }

    /// Scroll the element in a given direction.
    /// 
    /// Args:
    ///     direction (str): The direction to scroll.
    ///     amount (float): The amount to scroll.
    /// 
    /// Returns:
    ///     None
    pub fn scroll(&self, direction: &str, amount: f64) -> PyResult<()> {
        self.inner.scroll(direction, amount).map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "activate_window", text_signature = "($self)")]
    /// Activate the window containing this element.
    /// 
    /// Returns:
    ///     None
    pub fn activate_window(&self) -> PyResult<()> {
        self.inner.activate_window().map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "is_focused", text_signature = "($self)")]
    /// Check if element is focused.
    /// 
    /// Returns:
    ///     bool: True if the element is focused.
    pub fn is_focused(&self) -> PyResult<bool> {
        self.inner.is_focused().map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "is_keyboard_focusable", text_signature = "($self)")]
    /// Check if element is keyboard focusable.
    /// 
    /// Returns:
    ///     bool: True if the element can receive keyboard focus.
    pub fn is_keyboard_focusable(&self) -> PyResult<bool> {
        self.inner.is_keyboard_focusable().map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "mouse_drag", text_signature = "($self, start_x, start_y, end_x, end_y)")]
    /// Drag mouse from start to end coordinates.
    /// 
    /// Args:
    ///     start_x (float): Starting X coordinate.
    ///     start_y (float): Starting Y coordinate.
    ///     end_x (float): Ending X coordinate.
    ///     end_y (float): Ending Y coordinate.
    /// 
    /// Returns:
    ///     None
    pub fn mouse_drag(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> PyResult<()> {
        self.inner.mouse_drag(start_x, start_y, end_x, end_y).map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "mouse_click_and_hold", text_signature = "($self, x, y)")]
    /// Press and hold mouse at coordinates.
    /// 
    /// Args:
    ///     x (float): X coordinate.
    ///     y (float): Y coordinate.
    /// 
    /// Returns:
    ///     None
    pub fn mouse_click_and_hold(&self, x: f64, y: f64) -> PyResult<()> {
        self.inner.mouse_click_and_hold(x, y).map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "mouse_move", text_signature = "($self, x, y)")]
    /// Move mouse to coordinates.
    /// 
    /// Args:
    ///     x (float): X coordinate.
    ///     y (float): Y coordinate.
    /// 
    /// Returns:
    ///     None
    pub fn mouse_move(&self, x: f64, y: f64) -> PyResult<()> {
        self.inner.mouse_move(x, y).map_err(|e| automation_error_to_pyerr(e))
    }

    #[pyo3(name = "mouse_release", text_signature = "($self)")]
    /// Release mouse button.
    /// 
    /// Returns:
    ///     None
    pub fn mouse_release(&self) -> PyResult<()> {
        self.inner.mouse_release().map_err(|e| automation_error_to_pyerr(e))
    }

    /// Get the containing application element.
    /// 
    /// Returns:
    ///     Optional[UIElement]: The containing application element, if available.
    pub fn application(&self) -> PyResult<Option<UIElement>> {
        self.inner.application()
            .map(|opt| opt.map(|e| UIElement { inner: e }))
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Get the containing window element.
    /// 
    /// Returns:
    ///     Optional[UIElement]: The containing window element, if available.
    pub fn window(&self) -> PyResult<Option<UIElement>> {
        self.inner.window()
            .map(|opt| opt.map(|e| UIElement { inner: e }))
            .map_err(|e| automation_error_to_pyerr(e))
    }

    /// Create a locator from this element.
    /// 
    /// Args:
    ///     selector (str): The selector string.
    /// 
    /// Returns:
    ///     Locator: A new locator for finding elements.
    pub fn locator(&self, selector: &str) -> PyResult<Locator> {
        let locator = self.inner.locator(selector).map_err(|e| automation_error_to_pyerr(e))?;
        Ok(Locator { inner: locator })
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Locator {
    #[pyo3(name = "first", text_signature = "($self)")]
    /// Get the first matching element.
    /// 
    /// Returns:
    ///     UIElement: The first matching element.
    pub fn first<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, UIElement>(py, async move {
            let element = locator.first(None).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(UIElement { inner: element })
        })
    }

    #[pyo3(name = "all", text_signature = "($self, timeout_ms, depth)")]
    /// Get all matching elements.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    ///     depth (Optional[int]): Maximum depth to search.
    /// 
    /// Returns:
    ///     List[UIElement]: List of matching elements.
    pub fn all<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>, depth: Option<usize>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, Vec<UIElement>>(py, async move {
            let elements = locator.all(timeout_ms.map(std::time::Duration::from_millis), depth).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(elements.into_iter().map(|e| UIElement { inner: e }).collect())
        })
    }

    #[pyo3(name = "wait", text_signature = "($self, timeout_ms)")]
    /// Wait for the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     UIElement: The first matching element.
    pub fn wait<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, UIElement>(py, async move {
            let element = locator.wait(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(UIElement { inner: element })
        })
    }

    #[pyo3(name = "timeout", text_signature = "($self, timeout_ms)")]
    /// Set a default timeout for this locator.
    /// 
    /// Args:
    ///     timeout_ms (int): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     Locator: A new locator with the specified timeout.
    pub fn timeout(&self, timeout_ms: u64) -> Locator {
        Locator { inner: self.inner.clone().set_default_timeout(std::time::Duration::from_millis(timeout_ms)) }
    }

    #[pyo3(name = "locator", text_signature = "($self, selector)")]
    /// Chain another selector.
    /// 
    /// Args:
    ///     selector (str): The selector string.
    /// 
    /// Returns:
    ///     Locator: A new locator with the chained selector.
    pub fn locator(&self, selector: &str) -> PyResult<Locator> {
        let locator = self.inner.locator(selector);
        Ok(Locator { inner: locator })
    }

    #[pyo3(name = "click", text_signature = "($self, timeout_ms)")]
    /// Click on the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     ClickResult: Result of the click operation.
    pub fn click<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, ClickResult>(py, async move {
            let result = locator.click(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(ClickResult::from(result))
        })
    }

    #[pyo3(name = "type_text", text_signature = "($self, text, use_clipboard, timeout_ms)")]
    /// Type text into the first matching element.
    /// 
    /// Args:
    ///     text (str): The text to type.
    ///     use_clipboard (Optional[bool]): Whether to use clipboard for pasting.
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     None
    pub fn type_text<'py>(&self, py: Python<'py>, text: &str, use_clipboard: Option<bool>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        let text = text.to_string();
        pyo3_tokio::future_into_py::<_, ()>(py, async move {
            locator.type_text(&text, use_clipboard.unwrap_or(false), timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(())
        })
    }

    #[pyo3(name = "press_key", text_signature = "($self, key, timeout_ms)")]
    /// Press a key on the first matching element.
    /// 
    /// Args:
    ///     key (str): The key to press.
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     None
    pub fn press_key<'py>(&self, py: Python<'py>, key: &str, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        let key = key.to_string();
        pyo3_tokio::future_into_py::<_, ()>(py, async move {
            locator.press_key(&key, timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(())
        })
    }

    #[pyo3(name = "text", text_signature = "($self, max_depth, timeout_ms)")]
    /// Get text from the first matching element.
    /// 
    /// Args:
    ///     max_depth (Optional[int]): Maximum depth to search for text.
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     str: The element's text content.
    pub fn text<'py>(&self, py: Python<'py>, max_depth: Option<usize>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, String>(py, async move {
            let text = locator.text(max_depth.unwrap_or(1), timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(text)
        })
    }

    #[pyo3(name = "attributes", text_signature = "($self, timeout_ms)")]
    /// Get attributes from the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     UIElementAttributes: The element's attributes.
    pub fn attributes<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, UIElementAttributes>(py, async move {
            let attrs = locator.attributes(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(UIElementAttributes {
                role: attrs.role,
                name: attrs.name,
                label: attrs.label,
                value: attrs.value,
                description: attrs.description,
                properties: attrs.properties.into_iter()
                    .map(|(k, v)| (k, v.map(|v| v.to_string())))
                    .collect(),
                is_keyboard_focusable: attrs.is_keyboard_focusable,
            })
        })
    }

    #[pyo3(name = "bounds", text_signature = "($self, timeout_ms)")]
    /// Get bounds from the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     Bounds: The element's bounds.
    pub fn bounds<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, Bounds>(py, async move {
            let (x, y, width, height) = locator.bounds(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(Bounds { x, y, width, height })
        })
    }

    #[pyo3(name = "is_visible", text_signature = "($self, timeout_ms)")]
    /// Check if the element is visible.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     bool: True if the element is visible.
    pub fn is_visible<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, bool>(py, async move {
            let visible = locator.is_visible(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(visible)
        })
    }

    #[pyo3(name = "expect_enabled", text_signature = "($self, timeout_ms)")]
    /// Wait for the element to be enabled.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     UIElement: The enabled element.
    pub fn expect_enabled<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, UIElement>(py, async move {
            let element = locator.expect_enabled(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(UIElement { inner: element })
        })
    }

    #[pyo3(name = "expect_visible", text_signature = "($self, timeout_ms)")]
    /// Wait for the element to be visible.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     UIElement: The visible element.
    pub fn expect_visible<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, UIElement>(py, async move {
            let element = locator.expect_visible(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(UIElement { inner: element })
        })
    }

    #[pyo3(name = "expect_text_equals", text_signature = "($self, expected_text, max_depth, timeout_ms)")]
    /// Wait for the element's text to equal the expected text.
    /// 
    /// Args:
    ///     expected_text (str): The expected text.
    ///     max_depth (Optional[int]): Maximum depth to search for text.
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     UIElement: The element with matching text.
    pub fn expect_text_equals<'py>(&self, py: Python<'py>, expected_text: &str, max_depth: Option<usize>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        let expected_text = expected_text.to_string();
        pyo3_tokio::future_into_py::<_, UIElement>(py, async move {
            let element = locator.expect_text_equals(&expected_text, max_depth.unwrap_or(1), timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(UIElement { inner: element })
        })
    }

    #[pyo3(name = "within", text_signature = "($self, element)")]
    /// Set the root element for this locator.
    /// 
    /// Args:
    ///     element (UIElement): The root element.
    /// 
    /// Returns:
    ///     Locator: A new locator with the specified root element.
    pub fn within(&self, element: &UIElement) -> Locator {
        Locator { inner: self.inner.clone().within(element.inner.clone()) }
    }

    #[pyo3(name = "double_click", text_signature = "($self, timeout_ms)")]
    /// Double click on the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     ClickResult: Result of the click operation.
    pub fn double_click<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, ClickResult>(py, async move {
            let result = locator.double_click(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(ClickResult::from(result))
        })
    }

    #[pyo3(name = "right_click", text_signature = "($self, timeout_ms)")]
    /// Right click on the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     None
    pub fn right_click<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, ()>(py, async move {
            locator.right_click(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(())
        })
    }

    #[pyo3(name = "hover", text_signature = "($self, timeout_ms)")]
    /// Hover over the first matching element.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     None
    pub fn hover<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, ()>(py, async move {
            locator.hover(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(())
        })
    }
}

#[pymodule]
fn terminator(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Desktop>()?;
    m.add_class::<UIElement>()?;
    m.add_class::<Locator>()?;
    m.add_class::<ScreenshotResult>()?;
    m.add_class::<ClickResult>()?;
    m.add_class::<CommandOutput>()?;
    m.add_class::<UIElementAttributes>()?;
    m.add_class::<Coordinates>()?;
    m.add_class::<Bounds>()?;
    m.add_class::<RunCommandOptions>()?;

    m.add("ElementNotFoundError", _py.get_type_bound::<ElementNotFoundError>())?;
    m.add("TimeoutError", _py.get_type_bound::<TimeoutError>())?;
    m.add("PermissionDeniedError", _py.get_type_bound::<PermissionDeniedError>())?;
    m.add("PlatformError", _py.get_type_bound::<PlatformError>())?;
    m.add("UnsupportedOperationError", _py.get_type_bound::<UnsupportedOperationError>())?;
    m.add("UnsupportedPlatformError", _py.get_type_bound::<UnsupportedPlatformError>())?;
    m.add("InvalidArgumentError", _py.get_type_bound::<InvalidArgumentError>())?;
    m.add("InternalError", _py.get_type_bound::<InternalError>())?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
