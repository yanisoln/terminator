use napi_derive::napi;
use terminator::Locator as TerminatorLocator;

use crate::{
    Element,
    Bounds,
    ClickResult,
    UIElementAttributes,
    map_error,
};

/// Locator for finding UI elements by selector.
#[napi(js_name = "Locator")]
pub struct Locator {
    inner: TerminatorLocator,
}

impl From<TerminatorLocator> for Locator {
    fn from(l: TerminatorLocator) -> Self {
        Locator { inner: l }
    }
}

#[napi]
impl Locator {
    /// Get the first matching element.
    /// 
    /// @returns {Promise<Element>} The first matching element.
    #[napi]
    pub async fn first(&self) -> napi::Result<Element> {
        self.inner.first(None).await.map(Element::from).map_err(map_error)
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
        self.inner.all(timeout, depth).await.map(|els| els.into_iter().map(Element::from).collect()).map_err(map_error)
    }

    /// Wait for the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The first matching element.
    #[napi]
    pub async fn wait(&self, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.wait(timeout).await.map(Element::from).map_err(map_error)
    }

    /// Set a default timeout for this locator.
    /// 
    /// @param {number} timeoutMs - Timeout in milliseconds.
    /// @returns {Locator} A new locator with the specified timeout.
    #[napi]
    pub fn timeout(&self, timeout_ms: f64) -> Locator {
        let loc = self.inner.clone().set_default_timeout(std::time::Duration::from_millis(timeout_ms as u64));
        Locator::from(loc)
    }

    /// Set the root element for this locator.
    /// 
    /// @param {Element} element - The root element.
    /// @returns {Locator} A new locator with the specified root element.
    #[napi]
    pub fn within(&self, element: &Element) -> Locator {
        let loc = self.inner.clone().within(element.inner.clone());
        Locator::from(loc)
    }

    /// Chain another selector.
    /// 
    /// @param {string} selector - The selector string.
    /// @returns {Locator} A new locator with the chained selector.
    #[napi]
    pub fn locator(&self, selector: String) -> napi::Result<Locator> {
        let sel: terminator::selector::Selector = selector.as_str().into();
        let loc = self.inner.clone().locator(sel);
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
        self.inner.click(timeout).await.map(ClickResult::from).map_err(map_error)
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
        self.inner.type_text(&text, use_clipboard.unwrap_or(false), timeout).await.map_err(map_error)
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
        self.inner.press_key(&key, timeout).await.map_err(map_error)
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
        self.inner.text(max_depth.unwrap_or(1) as usize, timeout).await.map_err(map_error)
    }

    /// Get attributes from the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<UIElementAttributes>} The element's attributes.
    #[napi]
    pub async fn attributes(&self, timeout_ms: Option<f64>) -> napi::Result<UIElementAttributes> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.attributes(timeout).await.map(|attrs| {
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
        self.inner.bounds(timeout).await.map(Bounds::from).map_err(map_error)
    }

    /// Check if the element is visible.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<boolean>} True if the element is visible.
    #[napi]
    pub async fn is_visible(&self, timeout_ms: Option<f64>) -> napi::Result<bool> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.is_visible(timeout).await.map_err(map_error)
    }

    /// Wait for the element to be enabled.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The enabled element.
    #[napi]
    pub async fn expect_enabled(&self, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.expect_enabled(timeout).await.map(Element::from).map_err(map_error)
    }

    /// Wait for the element to be visible.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<Element>} The visible element.
    #[napi]
    pub async fn expect_visible(&self, timeout_ms: Option<f64>) -> napi::Result<Element> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.expect_visible(timeout).await.map(Element::from).map_err(map_error)
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
        self.inner.expect_text_equals(&expected_text, max_depth.unwrap_or(1) as usize, timeout).await.map(Element::from).map_err(map_error)
    }

    /// Double click on the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<ClickResult>} Result of the click operation.
    #[napi]
    pub async fn double_click(&self, timeout_ms: Option<f64>) -> napi::Result<ClickResult> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.double_click(timeout).await.map(ClickResult::from).map_err(map_error)
    }

    /// Right click on the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<void>}
    #[napi]
    pub async fn right_click(&self, timeout_ms: Option<f64>) -> napi::Result<()> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.right_click(timeout).await.map_err(map_error)
    }

    /// Hover over the first matching element.
    /// 
    /// @param {number} [timeoutMs] - Timeout in milliseconds.
    /// @returns {Promise<void>}
    #[napi]
    pub async fn hover(&self, timeout_ms: Option<f64>) -> napi::Result<()> {
        use std::time::Duration;
        let timeout = timeout_ms.map(|ms| Duration::from_millis(ms as u64));
        self.inner.hover(timeout).await.map_err(map_error)
    }
} 