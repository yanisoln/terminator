use napi::Status;
use napi_derive::napi;
use std::sync::{Arc, Mutex, Once};
use terminator::{
    Desktop as TerminatorDesktop, 
    element::{UIElement as TerminatorUIElement, UIElementAttributes as TerminatorUIElementAttributes}, 
    locator::Locator as TerminatorLocator, 
    errors::AutomationError
};
use std::collections::HashMap;

/// Main entry point for desktop automation.
#[napi(js_name = "Desktop")]
pub struct Desktop {
    inner: TerminatorDesktop,
}

#[napi]
impl Desktop {
    /// Create a new Desktop automation instance with default settings.
    /// 
    /// @returns {Desktop} A new Desktop automation instance.
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        Self::with_options(false, false)
    }

    /// Create a new Desktop automation instance with background apps enabled.
    /// 
    /// @returns {Desktop} A new Desktop automation instance with background apps enabled.
    #[napi(factory)]
    pub fn with_background_apps() -> napi::Result<Self> {
        Self::with_options(true, false)
    }

    /// Create a new Desktop automation instance with app activation enabled.
    /// 
    /// @returns {Desktop} A new Desktop automation instance with app activation enabled.
    #[napi(factory)]
    pub fn with_app_activation() -> napi::Result<Self> {
        Self::with_options(false, true)
    }

    /// Create a new Desktop automation instance with both background apps and app activation enabled.
    /// 
    /// @returns {Desktop} A new Desktop automation instance with all features enabled.
    #[napi(factory)]
    pub fn with_all_features() -> napi::Result<Self> {
        Self::with_options(true, true)
    }

    /// Internal helper to create a Desktop instance with specific options
    fn with_options(use_background_apps: bool, activate_app: bool) -> napi::Result<Self> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter("info")
                .try_init();
        });
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| napi::Error::from_reason(format!("Tokio runtime error: {e}")))?;
        let desktop = rt.block_on(TerminatorDesktop::new(use_background_apps, activate_app))
            .map_err(|e| napi::Error::from_reason(format!("{e}")))?;
        Ok(Desktop { inner: desktop })
    }

    /// Get the root UI element of the desktop.
    /// 
    /// @returns {Element} The root UI element.
    #[napi]
    pub fn root(&self) -> Element {
        let root = self.inner.root();
        Element::from(root)
    }

    /// Get a list of all running applications.
    /// 
    /// @returns {Array<Element>} List of application UI elements.
    #[napi]
    pub fn applications(&self) -> napi::Result<Vec<Element>> {
        self.inner.applications()
            .map(|apps| apps.into_iter().map(Element::from).collect())
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Get a running application by name.
    /// 
    /// @param {string} name - The name of the application to find.
    /// @returns {Element} The application UI element.
    #[napi]
    pub fn application(&self, name: String) -> napi::Result<Element> {
        self.inner.application(&name)
            .map(Element::from)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Open an application by name.
    /// 
    /// @param {string} name - The name of the application to open.
    #[napi]
    pub fn open_application(&self, name: String) -> napi::Result<()> {
        self.inner.open_application(&name)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Activate an application by name.
    /// 
    /// @param {string} name - The name of the application to activate.
    #[napi]
    pub fn activate_application(&self, name: String) -> napi::Result<()> {
        self.inner.activate_application(&name)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Capture a screenshot of the primary monitor.
    /// 
    /// @returns {Promise<ScreenshotResult>} The screenshot data.
    #[napi]
    pub async fn capture_screen(&self) -> napi::Result<ScreenshotResult> {
        self.inner.capture_screen().await
            .map(|r| ScreenshotResult {
                width: r.width,
                height: r.height,
                image_data: r.image_data,
            })
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Run a shell command.
    /// 
    /// @param {string} [windowsCommand] - Command to run on Windows.
    /// @param {string} [unixCommand] - Command to run on Unix.
    /// @returns {Promise<CommandOutput>} The command output.
    #[napi]
    pub async fn run_command(&self, windows_command: Option<String>, unix_command: Option<String>) -> napi::Result<CommandOutput> {
        self.inner.run_command(windows_command.as_deref(), unix_command.as_deref()).await
            .map(|r| CommandOutput {
                exit_status: r.exit_status,
                stdout: r.stdout,
                stderr: r.stderr,
            })
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Capture a screenshot of a specific monitor.
    /// 
    /// @param {string} name - The name of the monitor to capture.
    /// @returns {Promise<ScreenshotResult>} The screenshot data.
    #[napi]
    pub async fn capture_monitor_by_name(&self, name: String) -> napi::Result<ScreenshotResult> {
        self.inner.capture_monitor_by_name(&name).await
            .map(|r| ScreenshotResult {
                width: r.width,
                height: r.height,
                image_data: r.image_data,
            })
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Perform OCR on an image file.
    /// 
    /// @param {string} imagePath - Path to the image file.
    /// @returns {Promise<string>} The extracted text.
    #[napi]
    pub async fn ocr_image_path(&self, image_path: String) -> napi::Result<String> {
        self.inner.ocr_image_path(&image_path).await
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Perform OCR on a screenshot.
    /// 
    /// @param {ScreenshotResult} screenshot - The screenshot to process.
    /// @returns {Promise<string>} The extracted text.
    #[napi]
    pub async fn ocr_screenshot(&self, screenshot: ScreenshotResult) -> napi::Result<String> {
        let rust_screenshot = terminator::ScreenshotResult {
            image_data: screenshot.image_data,
            width: screenshot.width,
            height: screenshot.height,
        };
        self.inner.ocr_screenshot(&rust_screenshot).await
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Find a window by criteria.
    /// 
    /// @param {string} [titleContains] - Text that should be in the window title.
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The found window element.
    #[napi]
    pub async fn find_window_by_criteria(&self, title_contains: Option<String>, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.find_window_by_criteria(title_contains.as_deref(), timeout).await
            .map(Element::from)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Get the currently focused browser window.
    /// 
    /// @returns {Promise<Element>} The current browser window element.
    #[napi]
    pub async fn get_current_browser_window(&self) -> napi::Result<Element> {
        self.inner.get_current_browser_window().await
            .map(Element::from)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Create a locator for finding UI elements.
    /// 
    /// @param {string} selector - The selector string to find elements.
    /// @returns {Locator} A locator for finding elements.
    #[napi]
    pub fn locator(&self, selector: String) -> napi::Result<Locator> {
        let sel: terminator::selector::Selector = selector.as_str().into();
        let loc = self.inner.locator(sel);
        Ok(Locator::from(loc))
    }

    /// Get the currently focused window.
    /// 
    /// @returns {Promise<Element>} The current window element.
    #[napi]
    pub async fn get_current_window(&self) -> napi::Result<Element> {
        self.inner.get_current_window().await
            .map(Element::from)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Get the currently focused application.
    /// 
    /// @returns {Promise<Element>} The current application element.
    #[napi]
    pub async fn get_current_application(&self) -> napi::Result<Element> {
        self.inner.get_current_application().await
            .map(Element::from)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Get the currently focused element.
    /// 
    /// @returns {Element} The focused element.
    #[napi]
    pub fn focused_element(&self) -> napi::Result<Element> {
        self.inner.focused_element()
            .map(Element::from)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Open a URL in a browser.
    /// 
    /// @param {string} url - The URL to open.
    /// @param {string} [browser] - The browser to use.
    #[napi]
    pub fn open_url(&self, url: String, browser: Option<String>) -> napi::Result<()> {
        self.inner.open_url(&url, browser.as_deref())
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Open a file with its default application.
    /// 
    /// @param {string} filePath - Path to the file to open.
    #[napi]
    pub fn open_file(&self, file_path: String) -> napi::Result<()> {
        self.inner.open_file(&file_path)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }

    /// Activate a browser window by title.
    /// 
    /// @param {string} title - The window title to match.
    #[napi]
    pub fn activate_browser_window_by_title(&self, title: String) -> napi::Result<()> {
        self.inner.activate_browser_window_by_title(&title)
            .map_err(|e| napi::Error::from_reason(format!("{e}")))
    }
}

/// A UI element in the accessibility tree.
#[napi(js_name = "Element")]
pub struct Element {
    inner: Arc<Mutex<TerminatorUIElement>>,
}

impl From<TerminatorUIElement> for Element {
    fn from(e: TerminatorUIElement) -> Self {
        Element {
            inner: Arc::new(Mutex::new(e)),
        }
    }
}

#[napi]
impl Element {
    /// Get the element's ID.
    /// 
    /// @returns {string | null} The element's ID, if available.
    #[napi]
    pub fn id(&self) -> Option<String> {
        self.inner.lock().unwrap().id()
    }

    /// Get the element's role.
    /// 
    /// @returns {string} The element's role (e.g., "button", "textfield").
    #[napi]
    pub fn role(&self) -> napi::Result<String> {
        Ok(self.inner.lock().unwrap().role())
    }

    /// Get all attributes of the element.
    /// 
    /// @returns {UIElementAttributes} The element's attributes.
    #[napi]
    pub fn attributes(&self) -> UIElementAttributes {
        let attrs: TerminatorUIElementAttributes = self.inner.lock().unwrap().attributes();
        UIElementAttributes {
            role: attrs.role,
            name: attrs.name,
            label: attrs.label,
            value: attrs.value,
            description: attrs.description,
            properties: attrs.properties.into_iter()
                .map(|(k, v)| (k, v.map(|v| v.to_string())))
                .collect(),
            is_keyboard_focusable: attrs.is_keyboard_focusable,
        }
    }

    /// Get the element's name.
    /// 
    /// @returns {string | null} The element's name, if available.
    #[napi]
    pub fn name(&self) -> napi::Result<Option<String>> {
        Ok(self.inner.lock().unwrap().name())
    }

    /// Get children of this element.
    /// 
    /// @returns {Array<Element>} List of child elements.
    #[napi]
    pub fn children(&self) -> napi::Result<Vec<Element>> {
        self.inner.lock().unwrap().children()
            .map(|kids| kids.into_iter().map(Element::from).collect())
            .map_err(map_error)
    }

    /// Get the parent element.
    /// 
    /// @returns {Element | null} The parent element, if available.
    #[napi]
    pub fn parent(&self) -> napi::Result<Option<Element>> {
        self.inner.lock().unwrap().parent()
            .map(|opt| opt.map(Element::from))
            .map_err(map_error)
    }

    /// Get element bounds.
    /// 
    /// @returns {Bounds} The element's bounds (x, y, width, height).
    #[napi]
    pub fn bounds(&self) -> napi::Result<Bounds> {
        self.inner.lock().unwrap().bounds()
            .map(Bounds::from)
            .map_err(map_error)
    }

    /// Click on this element.
    /// 
    /// @returns {ClickResult} Result of the click operation.
    #[napi]
    pub fn click(&self) -> napi::Result<ClickResult> {
        self.inner.lock().unwrap().click()
            .map(ClickResult::from)
            .map_err(map_error)
    }

    /// Double click on this element.
    /// 
    /// @returns {ClickResult} Result of the click operation.
    #[napi]
    pub fn double_click(&self) -> napi::Result<ClickResult> {
        self.inner.lock().unwrap().double_click()
            .map(ClickResult::from)
            .map_err(map_error)
    }

    /// Right click on this element.
    #[napi]
    pub fn right_click(&self) -> napi::Result<()> {
        self.inner.lock().unwrap().right_click().map_err(map_error)
    }

    /// Hover over this element.
    #[napi]
    pub fn hover(&self) -> napi::Result<()> {
        self.inner.lock().unwrap().hover().map_err(map_error)
    }

    /// Check if element is visible.
    /// 
    /// @returns {boolean} True if the element is visible.
    #[napi]
    pub fn is_visible(&self) -> napi::Result<bool> {
        self.inner.lock().unwrap().is_visible().map_err(map_error)
    }

    /// Check if element is enabled.
    /// 
    /// @returns {boolean} True if the element is enabled.
    #[napi]
    pub fn is_enabled(&self) -> napi::Result<bool> {
        self.inner.lock().unwrap().is_enabled().map_err(map_error)
    }

    /// Focus this element.
    #[napi]
    pub fn focus(&self) -> napi::Result<()> {
        self.inner.lock().unwrap().focus().map_err(map_error)
    }

    /// Get text content of this element.
    /// 
    /// @param {number} [maxDepth] - Maximum depth to search for text.
    /// @returns {string} The element's text content.
    #[napi]
    pub fn text(&self, max_depth: Option<u32>) -> napi::Result<String> {
        self.inner.lock().unwrap().text(max_depth.unwrap_or(1) as usize).map_err(map_error)
    }

    /// Type text into this element.
    /// 
    /// @param {string} text - The text to type.
    /// @param {boolean} [useClipboard] - Whether to use clipboard for pasting.
    #[napi]
    pub fn type_text(&self, text: String, use_clipboard: Option<bool>) -> napi::Result<()> {
        self.inner.lock().unwrap().type_text(&text, use_clipboard.unwrap_or(false)).map_err(map_error)
    }

    /// Press a key while this element is focused.
    /// 
    /// @param {string} key - The key to press.
    #[napi]
    pub fn press_key(&self, key: String) -> napi::Result<()> {
        self.inner.lock().unwrap().press_key(&key).map_err(map_error)
    }

    /// Set value of this element.
    /// 
    /// @param {string} value - The value to set.
    #[napi]
    pub fn set_value(&self, value: String) -> napi::Result<()> {
        self.inner.lock().unwrap().set_value(&value).map_err(map_error)
    }

    /// Perform a named action on this element.
    /// 
    /// @param {string} action - The action to perform.
    #[napi]
    pub fn perform_action(&self, action: String) -> napi::Result<()> {
        self.inner.lock().unwrap().perform_action(&action).map_err(map_error)
    }

    /// Scroll the element in a given direction.
    /// 
    /// @param {string} direction - The direction to scroll.
    /// @param {number} amount - The amount to scroll.
    #[napi]
    pub fn scroll(&self, direction: String, amount: f64) -> napi::Result<()> {
        self.inner.lock().unwrap().scroll(&direction, amount).map_err(map_error)
    }

    /// Activate the window containing this element.
    #[napi]
    pub fn activate_window(&self) -> napi::Result<()> {
        self.inner.lock().unwrap().activate_window().map_err(map_error)
    }

    /// Check if element is focused.
    /// 
    /// @returns {boolean} True if the element is focused.
    #[napi]
    pub fn is_focused(&self) -> napi::Result<bool> {
        self.inner.lock().unwrap().is_focused().map_err(map_error)
    }

    /// Check if element is keyboard focusable.
    /// 
    /// @returns {boolean} True if the element can receive keyboard focus.
    #[napi]
    pub fn is_keyboard_focusable(&self) -> napi::Result<bool> {
        self.inner.lock().unwrap().is_keyboard_focusable().map_err(map_error)
    }

    /// Drag mouse from start to end coordinates.
    /// 
    /// @param {number} startX - Starting X coordinate.
    /// @param {number} startY - Starting Y coordinate.
    /// @param {number} endX - Ending X coordinate.
    /// @param {number} endY - Ending Y coordinate.
    #[napi]
    pub fn mouse_drag(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> napi::Result<()> {
        self.inner.lock().unwrap().mouse_drag(start_x, start_y, end_x, end_y).map_err(map_error)
    }

    /// Press and hold mouse at coordinates.
    /// 
    /// @param {number} x - X coordinate.
    /// @param {number} y - Y coordinate.
    #[napi]
    pub fn mouse_click_and_hold(&self, x: f64, y: f64) -> napi::Result<()> {
        self.inner.lock().unwrap().mouse_click_and_hold(x, y).map_err(map_error)
    }

    /// Move mouse to coordinates.
    /// 
    /// @param {number} x - X coordinate.
    /// @param {number} y - Y coordinate.
    #[napi]
    pub fn mouse_move(&self, x: f64, y: f64) -> napi::Result<()> {
        self.inner.lock().unwrap().mouse_move(x, y).map_err(map_error)
    }

    /// Release mouse button.
    #[napi]
    pub fn mouse_release(&self) -> napi::Result<()> {
        self.inner.lock().unwrap().mouse_release().map_err(map_error)
    }

    /// Create a locator from this element.
    /// 
    /// @param {string} selector - The selector string.
    /// @returns {Locator} A new locator for finding elements.
    #[napi]
    pub fn locator(&self, selector: String) -> napi::Result<Locator> {
        let sel: terminator::selector::Selector = selector.as_str().into();
        let loc = self.inner.lock().unwrap().locator(sel).map_err(map_error)?;
        Ok(Locator::from(loc))
    }

    /// Get the containing application element.
    /// 
    /// @returns {Element | null} The containing application element, if available.
    #[napi]
    pub fn application(&self) -> napi::Result<Option<Element>> {
        self.inner.lock().unwrap().application()
            .map(|opt| opt.map(Element::from))
            .map_err(map_error)
    }

    /// Get the containing window element.
    /// 
    /// @returns {Element | null} The containing window element, if available.
    #[napi]
    pub fn window(&self) -> napi::Result<Option<Element>> {
        self.inner.lock().unwrap().window()
            .map(|opt| opt.map(Element::from))
            .map_err(map_error)
    }
}

/// Locator for finding UI elements by selector.
#[napi(js_name = "Locator")]
pub struct Locator {
    inner: Arc<Mutex<TerminatorLocator>>,
}

impl From<TerminatorLocator> for Locator {
    fn from(l: TerminatorLocator) -> Self {
        Locator {
            inner: Arc::new(Mutex::new(l)),
        }
    }
}

#[napi]
impl Locator {
    /// Get the first matching element.
    /// 
    /// @returns {Promise<Element>} The first matching element.
    #[napi]
    pub async fn first(&self) -> napi::Result<Element> {
        let loc = self.inner.lock().unwrap().clone();
        loc.first(None).await.map(Element::from).map_err(map_error)
    }

    /// Get all matching elements.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @param {number} [depth] - Maximum depth to search.
    /// @returns {Promise<Array<Element>>} List of matching elements.
    #[napi]
    pub async fn all(&self, timeout_ms: Option<f64>, depth: Option<u32>) -> napi::Result<Vec<Element>> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let depth = depth.map(|d| d as usize);
        let loc = self.inner.lock().unwrap().clone();
        loc.all(timeout, depth).await.map(|els| els.into_iter().map(Element::from).collect()).map_err(map_error)
    }

    /// Wait for the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The first matching element.
    #[napi]
    pub async fn wait(&self, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.wait(timeout).await.map(Element::from).map_err(map_error)
    }

    /// Set a default timeout for this locator.
    /// 
    /// @param {number} timeoutMs - Timeout in milliseconds.
    /// @returns {Locator} A new locator with the specified timeout.
    #[napi]
    pub fn timeout(&self, timeout_ms: f64) -> Locator {
        let loc = self.inner.lock().unwrap().clone().set_default_timeout(std::time::Duration::from_millis(timeout_ms as u64));
        Locator::from(loc)
    }

    /// Set the root element for this locator.
    /// 
    /// @param {Element} element - The root element.
    /// @returns {Locator} A new locator with the specified root element.
    #[napi]
    pub fn within(&self, element: &Element) -> Locator {
        let loc = self.inner.lock().unwrap().clone().within(element.inner.lock().unwrap().clone());
        Locator::from(loc)
    }

    /// Chain another selector.
    /// 
    /// @param {string} selector - The selector string.
    /// @returns {Locator} A new locator with the chained selector.
    #[napi]
    pub fn locator(&self, selector: String) -> napi::Result<Locator> {
        let sel: terminator::selector::Selector = selector.as_str().into();
        let loc = self.inner.lock().unwrap().clone().locator(sel);
        Ok(Locator::from(loc))
    }

    /// Click on the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<ClickResult>} Result of the click operation.
    #[napi]
    pub async fn click(&self, timeout_ms: Option<f64>) -> napi::Result<ClickResult> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.click(timeout).await.map(ClickResult::from).map_err(map_error)
    }

    /// Type text into the first matching element.
    /// 
    /// @param {string} text - The text to type.
    /// @param {boolean} [useClipboard] - Whether to use clipboard for pasting.
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<void>}
    #[napi]
    pub async fn type_text(&self, text: String, use_clipboard: Option<bool>, timeout_ms: Option<f64>) -> napi::Result<()> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.type_text(&text, use_clipboard.unwrap_or(false), timeout).await.map_err(map_error)
    }

    /// Press a key on the first matching element.
    /// 
    /// @param {string} key - The key to press.
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<void>}
    #[napi]
    pub async fn press_key(&self, key: String, timeout_ms: Option<f64>) -> napi::Result<()> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.press_key(&key, timeout).await.map_err(map_error)
    }

    /// Get text from the first matching element.
    /// 
    /// @param {number} [maxDepth] - Maximum depth to search for text.
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<string>} The element's text content.
    #[napi]
    pub async fn text(&self, max_depth: Option<u32>, timeout_ms: Option<f64>) -> napi::Result<String> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.text(max_depth.unwrap_or(1) as usize, timeout).await.map_err(map_error)
    }

    /// Get attributes from the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<UIElementAttributes>} The element's attributes.
    #[napi]
    pub async fn attributes(&self, timeout_ms: Option<f64>) -> napi::Result<UIElementAttributes> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.attributes(timeout).await.map(|attrs| {
            UIElementAttributes {
                role: attrs.role,
                name: attrs.name,
                label: attrs.label,
                value: attrs.value,
                description: attrs.description,
                properties: attrs.properties.into_iter()
                    .map(|(k, v)| (k, v.map(|v| v.to_string())))
                    .collect(),
                is_keyboard_focusable: attrs.is_keyboard_focusable,
            }
        }).map_err(map_error)
    }

    /// Get bounds from the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Bounds>} The element's bounds.
    #[napi]
    pub async fn bounds(&self, timeout_ms: Option<f64>) -> napi::Result<Bounds> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.bounds(timeout).await.map(Bounds::from).map_err(map_error)
    }

    /// Check if the element is visible.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<boolean>} True if the element is visible.
    #[napi]
    pub async fn is_visible(&self, timeout_ms: Option<f64>) -> napi::Result<bool> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.is_visible(timeout).await.map_err(map_error)
    }

    /// Wait for the element to be enabled.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The enabled element.
    #[napi]
    pub async fn expect_enabled(&self, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.expect_enabled(timeout).await.map(Element::from).map_err(map_error)
    }

    /// Wait for the element to be visible.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The visible element.
    #[napi]
    pub async fn expect_visible(&self, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.expect_visible(timeout).await.map(Element::from).map_err(map_error)
    }

    /// Wait for the element's text to equal the expected text.
    /// 
    /// @param {string} expectedText - The expected text.
    /// @param {number} [maxDepth] - Maximum depth to search for text.
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The element with matching text.
    #[napi]
    pub async fn expect_text_equals(&self, expected_text: String, max_depth: Option<u32>, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.expect_text_equals(&expected_text, max_depth.unwrap_or(1) as usize, timeout).await.map(Element::from).map_err(map_error)
    }

    /// Double click on the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<ClickResult>} Result of the click operation.
    #[napi]
    pub async fn double_click(&self, timeout_ms: Option<f64>) -> napi::Result<ClickResult> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.double_click(timeout).await.map(ClickResult::from).map_err(map_error)
    }

    /// Right click on the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<void>}
    #[napi]
    pub async fn right_click(&self, timeout_ms: Option<f64>) -> napi::Result<()> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.right_click(timeout).await.map_err(map_error)
    }

    /// Hover over the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<void>}
    #[napi]
    pub async fn hover(&self, timeout_ms: Option<f64>) -> napi::Result<()> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        let loc = self.inner.lock().unwrap().clone();
        loc.hover(timeout).await.map_err(map_error)
    }
}

// --- Result types ---

#[napi(object, js_name = "Bounds")]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[napi(object, js_name = "Coordinates")]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
}

#[napi(object, js_name = "ClickResult")]
pub struct ClickResult {
    pub method: String,
    pub coordinates: Option<Coordinates>,
    pub details: String,
}

#[napi(object, js_name = "CommandOutput")]
pub struct CommandOutput {
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Result of a screenshot operation
#[napi(object)]
pub struct ScreenshotResult {
    pub width: u32,
    pub height: u32,
    pub image_data: Vec<u8>,
}

#[napi(object, js_name = "UIElementAttributes")]
pub struct UIElementAttributes {
    pub role: String,
    pub name: Option<String>,
    pub label: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub properties: HashMap<String, Option<String>>,
    pub is_keyboard_focusable: Option<bool>,
}

impl From<(f64, f64, f64, f64)> for Bounds {
    fn from(t: (f64, f64, f64, f64)) -> Self {
        Bounds { x: t.0, y: t.1, width: t.2, height: t.3 }
    }
}

impl From<(f64, f64)> for Coordinates {
    fn from(t: (f64, f64)) -> Self {
        Coordinates { x: t.0, y: t.1 }
    }
}

impl From<terminator::ClickResult> for ClickResult {
    fn from(r: terminator::ClickResult) -> Self {
        ClickResult {
            method: r.method,
            coordinates: r.coordinates.map(Coordinates::from),
            details: r.details,
        }
    }
}

// --- Error types ---

/// Thrown when an element is not found.
#[napi(js_name = "ElementNotFoundError")]
pub struct ElementNotFoundError(pub String);

/// Thrown when an operation times out.
#[napi(js_name = "TimeoutError")]
pub struct TimeoutError(pub String);

/// Thrown when permission is denied.
#[napi(js_name = "PermissionDeniedError")]
pub struct PermissionDeniedError(pub String);

/// Thrown for platform-specific errors.
#[napi(js_name = "PlatformError")]
pub struct PlatformError(pub String);

/// Thrown for unsupported operations.
#[napi(js_name = "UnsupportedOperationError")]
pub struct UnsupportedOperationError(pub String);

/// Thrown for unsupported platforms.
#[napi(js_name = "UnsupportedPlatformError")]
pub struct UnsupportedPlatformError(pub String);

/// Thrown for invalid arguments.
#[napi(js_name = "InvalidArgumentError")]
pub struct InvalidArgumentError(pub String);

/// Thrown for internal errors.
#[napi(js_name = "InternalError")]
pub struct InternalError(pub String);

// Implement Display and Error for all error classes
macro_rules! impl_js_error {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($name), self.0)
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($name), self.0)
            }
        }
        impl std::error::Error for $name {}
    };
}
impl_js_error!(ElementNotFoundError);
impl_js_error!(TimeoutError);
impl_js_error!(PermissionDeniedError);
impl_js_error!(PlatformError);
impl_js_error!(UnsupportedOperationError);
impl_js_error!(UnsupportedPlatformError);
impl_js_error!(InvalidArgumentError);
impl_js_error!(InternalError);

fn map_error(e: AutomationError) -> napi::Error {
    use AutomationError::*;
    match e {
        ElementNotFound(msg) => napi::Error::new(Status::GenericFailure, format!("ElementNotFoundError: {msg}")),
        Timeout(msg) => napi::Error::new(Status::GenericFailure, format!("TimeoutError: {msg}")),
        PermissionDenied(msg) => napi::Error::new(Status::GenericFailure, format!("PermissionDeniedError: {msg}")),
        PlatformError(msg) => napi::Error::new(Status::GenericFailure, format!("PlatformError: {msg}")),
        UnsupportedOperation(msg) => napi::Error::new(Status::GenericFailure, format!("UnsupportedOperationError: {msg}")),
        UnsupportedPlatform(msg) => napi::Error::new(Status::GenericFailure, format!("UnsupportedPlatformError: {msg}")),
        InvalidArgument(msg) => napi::Error::new(Status::GenericFailure, format!("InvalidArgumentError: {msg}")),
        Internal(msg) => napi::Error::new(Status::GenericFailure, format!("InternalError: {msg}")),
    }
}
