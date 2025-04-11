use crate::platforms::AccessibilityEngine;
use crate::{AutomationError, Selector, UIElement};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::ClickResult;

/// A high-level API for finding and interacting with UI elements
pub struct Locator {
    engine: Arc<dyn AccessibilityEngine>,
    selector: Selector,
    timeout: Duration,
    root: Option<UIElement>,
}

impl Locator {
    /// Create a new locator with the given selector
    pub(crate) fn new(engine: Arc<dyn AccessibilityEngine>, selector: Selector) -> Self {
        Self {
            engine,
            selector,
            timeout: Duration::from_secs(30),
            root: None,
        }
    }

    /// Set timeout for waiting operations
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the root element for this locator
    pub fn within(mut self, element: UIElement) -> Self {
        self.root = Some(element);
        self
    }

    /// Get the first element matching this locator
    pub fn first(&self) -> Result<Option<UIElement>, AutomationError> {
        let element = self
            .engine
            .find_element(&self.selector, self.root.as_ref())?;
        Ok(Some(element))
    }

    /// Get all elements matching this locator
    pub fn all(&self) -> Result<Vec<UIElement>, AutomationError> {
        // Check if we can use platform-specific find_elements method
        if let Ok(elements) = self
            .engine
            .find_elements(&self.selector, self.root.as_ref())
        {
            return Ok(elements);
        }

        // Fallback implementation - get the first element, then get its siblings
        // Note: This is a naive implementation and might not work correctly in all cases
        match self.first()? {
            Some(first) => {
                let result = vec![first];
                // In a proper implementation, we would need to search for siblings
                // or implement a custom ElementCollector that gathers all matches
                Ok(result)
            }
            None => Ok(vec![]),
        }
    }

    /// Wait for an element to be available
    pub async fn wait(&self) -> Result<UIElement, AutomationError> {
        let start = std::time::Instant::now();

        while start.elapsed() < self.timeout {
            if let Some(element) = self.first()? {
                return Ok(element);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Err(AutomationError::Timeout(format!(
            "Timed out waiting for selector: {:?}",
            self.selector
        )))
    }

    /// Get a nested locator
    pub fn locator(&self, selector: impl Into<Selector>) -> Locator {
        let selector = selector.into();
        Locator {
            engine: self.engine.clone(),
            selector: Selector::Chain(vec![self.selector.clone(), selector]),
            timeout: self.timeout,
            root: self.root.clone(),
        }
    }

    // Convenience methods for common actions

    /// Click on the first matching element
    pub async fn click(&self) -> Result<ClickResult, AutomationError> {
        self.wait().await?.click()
    }

    /// Type text into the first matching element
    pub async fn type_text(&self, text: &str) -> Result<(), AutomationError> {
        self.wait().await?.type_text(text)
    }

    /// Press a key on the first matching element
    pub async fn press_key(&self, key: &str) -> Result<(), AutomationError> {
        self.wait().await?.press_key(key)
    }

    /// Get text from the first matching element
    pub async fn text(&self, max_depth: usize) -> Result<String, AutomationError> {
        self.wait().await?.text(max_depth)
    }

    /// Waits for the element matched by the locator to be enabled.
    pub async fn expect_enabled(&self, timeout: Option<Duration>) -> Result<UIElement, AutomationError> {
        let effective_timeout = timeout.unwrap_or(self.timeout);
        let start = Instant::now();

        loop {
            // Try to find the element first
            // We use find_element directly instead of self.wait() to control the loop precisely
            match self.engine.find_element(&self.selector, self.root.as_ref()) {
                Ok(element) => {
                    // Element found, check if it's enabled
                    match element.is_enabled() {
                        Ok(true) => return Ok(element), // Success! Return the element
                        Ok(false) => { /* Condition not met, continue loop */ }
                        Err(e) => {
                            // Propagate errors other than "not found" immediately
                            // If the element disappears while checking, we might get an error here
                            if !matches!(e, AutomationError::ElementNotFound(_)) {
                                return Err(e);
                            }
                            // Otherwise, element might have changed, continue loop
                        }
                    }
                }
                Err(AutomationError::ElementNotFound(_)) => { /* Element not found yet, continue loop */ }
                Err(e) => return Err(e), // Propagate other find errors
            }

            // Check timeout
            if start.elapsed() >= effective_timeout {
                return Err(AutomationError::Timeout(format!(
                    "Timed out after {:?} waiting for element {:?} to be enabled",
                    effective_timeout, self.selector
                )));
            }

            // Wait before next check
            tokio::time::sleep(Duration::from_millis(100)).await; // Adjust poll interval as needed
        }
    }

    /// Waits for the element matched by the locator to be visible.
    pub async fn expect_visible(&self, timeout: Option<Duration>) -> Result<UIElement, AutomationError> {
        let effective_timeout = timeout.unwrap_or(self.timeout);
        let start = Instant::now();

        loop {
            match self.engine.find_element(&self.selector, self.root.as_ref()) {
                Ok(element) => {
                    match element.is_visible() {
                        Ok(true) => return Ok(element), // Success!
                        Ok(false) => { /* Condition not met, continue loop */ }
                         Err(e) => {
                            if !matches!(e, AutomationError::ElementNotFound(_)) {
                                return Err(e);
                            }
                         }
                    }
                }
                Err(AutomationError::ElementNotFound(_)) => { /* Element not found yet, continue loop */ }
                Err(e) => return Err(e),
            }

            if start.elapsed() >= effective_timeout {
                return Err(AutomationError::Timeout(format!(
                    "Timed out after {:?} waiting for element {:?} to be visible",
                    effective_timeout, self.selector
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Waits for the element's text content (potentially including children) to match the expected text.
    /// Note: Uses element.text(max_depth), adjust depth as needed.
    pub async fn expect_text_equals(&self, expected_text: &str, max_depth: usize, timeout: Option<Duration>) -> Result<UIElement, AutomationError> {
        let effective_timeout = timeout.unwrap_or(self.timeout);
        let start = Instant::now();

        loop {
            match self.engine.find_element(&self.selector, self.root.as_ref()) {
                Ok(element) => {
                    match element.text(max_depth) {
                         // Compare trimmed text to handle potential whitespace differences
                        Ok(actual_text) if actual_text.trim() == expected_text.trim() => return Ok(element), // Success!
                        Ok(_) => { /* Text doesn't match, continue loop */ }
                         Err(e) => {
                            if !matches!(e, AutomationError::ElementNotFound(_)) {
                                return Err(e);
                            }
                         }
                    }
                }
                Err(AutomationError::ElementNotFound(_)) => { /* Element not found yet, continue loop */ }
                Err(e) => return Err(e),
            }

            if start.elapsed() >= effective_timeout {
                 return Err(AutomationError::Timeout(format!(
                    "Timed out after {:?} waiting for element {:?} text to equal '{}'",
                    effective_timeout, self.selector, expected_text
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
