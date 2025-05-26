use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use pyo3_asyncio_0_21::tokio as pyo3_tokio;
use ::terminator_core::locator::Locator as TerminatorLocator;
use crate::exceptions::automation_error_to_pyerr;
use crate::types::{UIElementAttributes, Bounds, ClickResult};
use crate::element::UIElement;

/// Locator for finding UI elements by selector.
#[gen_stub_pyclass]
#[pyclass(name = "Locator")]
pub struct Locator {
    pub inner: TerminatorLocator,
}

#[gen_stub_pymethods]
#[pymethods]
impl Locator {
    #[pyo3(name = "first", text_signature = "($self)")]
    /// (async) Get the first matching element.
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
    /// (async) Get all matching elements.
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
    /// (async) Wait for the first matching element.
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
    /// (async) Click on the first matching element.
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
    /// (async) Type text into the first matching element.
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
    /// (async) Press a key on the first matching element.
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
    /// (async) Get text from the first matching element.
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
    /// (async) Get attributes from the first matching element.
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
    /// (async) Get bounds from the first matching element.
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
    /// (async) Check if the element is visible.
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
    /// (async) Wait for the element to be enabled.
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
    /// (async) Wait for the element to be visible.
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
    /// (async) Wait for the element's text to equal the expected text.
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
    /// (async) Double click on the first matching element.
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
    /// (async) Right click on the first matching element.
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
    /// (async) Hover over the first matching element.
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

    /// (async) Explore the first matching element and its direct children.
    /// 
    /// Args:
    ///     timeout_ms (Optional[int]): Timeout in milliseconds.
    /// 
    /// Returns:
    ///     ExploreResponse: Details about the element and its children.
    pub fn explore<'py>(&self, py: Python<'py>, timeout_ms: Option<u64>) -> PyResult<Bound<'py, PyAny>> {
        let locator = self.inner.clone();
        pyo3_tokio::future_into_py::<_, crate::types::ExploreResponse>(py, async move {
            let response = locator.explore(timeout_ms.map(std::time::Duration::from_millis)).await.map_err(|e| automation_error_to_pyerr(e))?;
            Ok(crate::types::ExploreResponse {
                parent: UIElement { inner: response.parent },
                children: response.children.into_iter().map(|child| crate::types::ExploredElementDetail {
                    role: child.role,
                    name: child.name,
                    id: child.id,
                    bounds: child.bounds.map(|(x, y, width, height)| Bounds { x, y, width, height }),
                    value: child.value,
                    description: child.description,
                    text: child.text,
                    parent_id: child.parent_id,
                    children_ids: child.children_ids,
                    suggested_selector: child.suggested_selector,
                }).collect(),
            })
        })
    }
} 