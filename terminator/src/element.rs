use crate::errors::AutomationError;
use crate::selector::Selector;
use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};

use super::{ClickResult, Locator};

/// Response structure for exploration result
#[derive(Debug, Default)]
pub struct ExploredElementDetail {
    pub role: String,
    pub name: Option<String>, // Use 'name' consistently for the primary label/text
    pub id: Option<String>,
    pub bounds: Option<(f64, f64, f64, f64)>, // Include bounds for spatial context
    pub value: Option<String>,
    pub description: Option<String>,
    pub text: Option<String>,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub suggested_selector: String,
}

impl ExploredElementDetail {
    /// Create a new ExploredElementDetail from a UIElement
    pub fn from_element(element: &UIElement, parent_id: Option<String>) -> Result<Self, AutomationError> {
        let id = element.id_or_empty();
        Ok(Self {
            role: element.role(),
            name: element.name(),
            id: if id.is_empty() { None } else { Some(id.clone()) },
            bounds: element.bounds().ok(),
            value: element.attributes().value,
            description: element.attributes().description,
            text: element.text(1).ok(),
            parent_id,
            children_ids: Vec::new(),
            suggested_selector: format!("#{}", id),
        })
    }
}

/// Response structure for exploration result
#[derive(Debug)]
pub struct ExploreResponse {
    pub parent: UIElement, // The parent element explored
    pub children: Vec<ExploredElementDetail>, // List of direct children details
}

/// Represents a UI element in a desktop application
#[derive(Debug)]
pub struct UIElement {
    inner: Box<dyn UIElementImpl>,
}

/// Attributes associated with a UI element
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UIElementAttributes {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, Option<serde_json::Value>>,
    #[serde(default)]
    pub is_keyboard_focusable: Option<bool>,
}

/// Interface for platform-specific element implementations
pub(crate) trait UIElementImpl: Send + Sync + Debug {
    fn object_id(&self) -> usize;
    fn id(&self) -> Option<String>;
    fn role(&self) -> String;
    fn attributes(&self) -> UIElementAttributes;
    fn name(&self) -> Option<String> {
        self.attributes().name
    }
    fn children(&self) -> Result<Vec<UIElement>, AutomationError>;
    fn parent(&self) -> Result<Option<UIElement>, AutomationError>;
    fn bounds(&self) -> Result<(f64, f64, f64, f64), AutomationError>; // x, y, width, height
    fn click(&self) -> Result<ClickResult, AutomationError>;
    fn double_click(&self) -> Result<ClickResult, AutomationError>;
    fn right_click(&self) -> Result<(), AutomationError>;
    fn hover(&self) -> Result<(), AutomationError>;
    fn focus(&self) -> Result<(), AutomationError>;
    fn type_text(&self, text: &str, use_clipboard: bool) -> Result<(), AutomationError>;
    fn press_key(&self, key: &str) -> Result<(), AutomationError>;
    fn get_text(&self, max_depth: usize) -> Result<String, AutomationError>;
    fn set_value(&self, value: &str) -> Result<(), AutomationError>;
    fn is_enabled(&self) -> Result<bool, AutomationError>;
    fn is_visible(&self) -> Result<bool, AutomationError>;
    fn is_focused(&self) -> Result<bool, AutomationError>;
    fn perform_action(&self, action: &str) -> Result<(), AutomationError>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn create_locator(&self, selector: Selector) -> Result<Locator, AutomationError>;
    fn scroll(&self, direction: &str, amount: f64) -> Result<(), AutomationError>;

    // New method to activate the window containing the element
    fn activate_window(&self) -> Result<(), AutomationError>;

    // Add a method to clone the box
    fn clone_box(&self) -> Box<dyn UIElementImpl>;

    // New method for keyboard focusable
    fn is_keyboard_focusable(&self) -> Result<bool, AutomationError>;

    // New method for mouse drag
    fn mouse_drag(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Result<(), AutomationError>;

    // New methods for mouse control
    fn mouse_click_and_hold(&self, x: f64, y: f64) -> Result<(), AutomationError>;
    fn mouse_move(&self, x: f64, y: f64) -> Result<(), AutomationError>;
    fn mouse_release(&self) -> Result<(), AutomationError>;

    // New methods to get containing application and window
    fn application(&self) -> Result<Option<UIElement>, AutomationError>;
    fn window(&self) -> Result<Option<UIElement>, AutomationError>;

    // New method to highlight the element
    fn highlight(&self, color: Option<u32>, duration: Option<std::time::Duration>) -> Result<(), AutomationError>;
}

impl UIElement {
    /// Create a new UI element from a platform-specific implementation
    pub(crate) fn new(impl_: Box<dyn UIElementImpl>) -> Self {
        Self { inner: impl_ }
    }

    /// Get the element's ID
    #[instrument(skip(self))]
    pub fn id(&self) -> Option<String> {
        
        let id = self.inner.id();
        

        
        id
    }

    /// Get the element's role (e.g., "button", "textfield")
    pub fn role(&self) -> String {
        self.inner.role()
    }

    /// Get all attributes of the element
    pub fn attributes(&self) -> UIElementAttributes {
        self.inner.attributes()
    }

    /// Get child elements
    pub fn children(&self) -> Result<Vec<UIElement>, AutomationError> {
        self.inner.children()
    }

    /// Get parent element
    pub fn parent(&self) -> Result<Option<UIElement>, AutomationError> {
        self.inner.parent()
    }

    /// Get element bounds (x, y, width, height)
    pub fn bounds(&self) -> Result<(f64, f64, f64, f64), AutomationError> {
        self.inner.bounds()
    }

    /// Click on this element
    #[instrument(skip(self))]
    pub fn click(&self) -> Result<ClickResult, AutomationError> {
        
        let result = self.inner.click();
        

        
        result
    }

    /// Double-click on this element
    #[instrument(skip(self))]
    pub fn double_click(&self) -> Result<ClickResult, AutomationError> {
        
        let result = self.inner.double_click();
        
   
        
        result
    }

    /// Right-click on this element
    #[instrument(skip(self))]
    pub fn right_click(&self) -> Result<(), AutomationError> {
        
        let result = self.inner.right_click();
        

        result
    }

    /// Hover over this element
    pub fn hover(&self) -> Result<(), AutomationError> {
        self.inner.hover()
    }

    /// Focus this element
    pub fn focus(&self) -> Result<(), AutomationError> {
        self.inner.focus()
    }

    /// Type text into this element
    pub fn type_text(&self, text: &str, use_clipboard: bool) -> Result<(), AutomationError> {
        self.inner.type_text(text, use_clipboard)
    }

    /// Press a key while this element is focused
    pub fn press_key(&self, key: &str) -> Result<(), AutomationError> {
        self.inner.press_key(key)
    }

    /// Get text content of this element
    pub fn text(&self, max_depth: usize) -> Result<String, AutomationError> {
        self.inner.get_text(max_depth)
    }

    /// Set value of this element
    pub fn set_value(&self, value: &str) -> Result<(), AutomationError> {
        self.inner.set_value(value)
    }

    /// Check if element is enabled
    #[instrument(skip(self))]
    pub fn is_enabled(&self) -> Result<bool, AutomationError> {
        let is_enabled = self.inner.is_enabled()?;
     
        Ok(is_enabled)
    }

    /// Check if element is visible
    pub fn is_visible(&self) -> Result<bool, AutomationError> {
        self.inner.is_visible()
    }

    /// Check if element is focused
    pub fn is_focused(&self) -> Result<bool, AutomationError> {
        self.inner.is_focused()
    }

    /// Perform a named action on this element
    pub fn perform_action(&self, action: &str) -> Result<(), AutomationError> {
        self.inner.perform_action(action)
    }

    /// Get the underlying implementation as a specific type
    pub(crate) fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }

    /// Find elements matching the selector within this element
    pub fn locator(&self, selector: impl Into<Selector>) -> Result<Locator, AutomationError> {
        let selector = selector.into();
        self.inner.create_locator(selector)
    }

    /// Scroll the element in a given direction
    pub fn scroll(&self, direction: &str, amount: f64) -> Result<(), AutomationError> {
        self.inner.scroll(direction, amount)
    }

    /// Activate the window containing this element (bring to foreground)
    pub fn activate_window(&self) -> Result<(), AutomationError> {
        self.inner.activate_window()
    }

    /// Get the element's name
    #[instrument(skip(self))]
    pub fn name(&self) -> Option<String> {
        let name = self.inner.name();
        name
    }

    /// Check if element is keyboard focusable
    pub fn is_keyboard_focusable(&self) -> Result<bool, AutomationError> {
        self.inner.is_keyboard_focusable()
    }

    /// Drag mouse from start to end coordinates
    pub fn mouse_drag(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Result<(), AutomationError> {
        self.inner.mouse_drag(start_x, start_y, end_x, end_y)
    }

    /// Press and hold mouse at (x, y)
    pub fn mouse_click_and_hold(&self, x: f64, y: f64) -> Result<(), AutomationError> {
        self.inner.mouse_click_and_hold(x, y)
    }

    /// Move mouse to (x, y)
    pub fn mouse_move(&self, x: f64, y: f64) -> Result<(), AutomationError> {
        self.inner.mouse_move(x, y)
    }

    /// Release mouse button
    pub fn mouse_release(&self) -> Result<(), AutomationError> {
        self.inner.mouse_release()
    }

    /// Get the containing application element
    pub fn application(&self) -> Result<Option<UIElement>, AutomationError> {
        self.inner.application()
    }

    /// Get the containing window element (e.g., tab, dialog)
    pub fn window(&self) -> Result<Option<UIElement>, AutomationError> {
        self.inner.window()
    }

    /// Highlights the element with a colored border.
    /// 
    /// # Arguments
    /// * `color` - Optional BGR color code (32-bit integer). Default: 0x0000FF (red)
    /// * `duration` - Optional duration for the highlight.
    pub fn highlight(&self, color: Option<u32>, duration: Option<std::time::Duration>) -> Result<(), AutomationError> {
        self.inner.highlight(color, duration)
    }

    /// Convenience methods to reduce verbosity with optional properties
    
    /// Get element ID or empty string if not available
    pub fn id_or_empty(&self) -> String {
        self.id().unwrap_or_default()
    }

    /// Get element name or empty string if not available  
    pub fn name_or_empty(&self) -> String {
        self.name().unwrap_or_default()
    }

    /// Get element name or fallback string if not available
    pub fn name_or(&self, fallback: &str) -> String {
        self.name().unwrap_or_else(|| fallback.to_string())
    }

    /// Get element value or empty string if not available
    pub fn value_or_empty(&self) -> String {
        self.attributes().value.unwrap_or_default()
    }

    /// Get element description or empty string if not available
    pub fn description_or_empty(&self) -> String {
        self.attributes().description.unwrap_or_default()
    }

    /// Get application name safely
    pub fn application_name(&self) -> String {
        self.application()
            .ok()
            .flatten()
            .and_then(|app| app.name())
            .unwrap_or_default()
    }

    /// Get window title safely
    pub fn window_title(&self) -> String {
        self.window()
            .ok()
            .flatten()
            .and_then(|win| win.name())
            .unwrap_or_default()
    }

    /// Explore this element and its direct children
    /// // mark deprecated
    #[deprecated(since = "0.3.5")]
    pub fn explore(&self) -> Result<ExploreResponse, AutomationError> {
        let mut children = Vec::new();
        for child in self.children()? {
            children.push(ExploredElementDetail::from_element(&child, self.id())?);
        }

        Ok(ExploreResponse {
            parent: self.clone(),
            children,
        })
    }
}

impl PartialEq for UIElement {
    fn eq(&self, other: &Self) -> bool {
        self.inner.object_id() == other.inner.object_id()
    }
}

impl Eq for UIElement {}

impl std::hash::Hash for UIElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.object_id().hash(state);
    }
}

impl Clone for UIElement {
    fn clone(&self) -> Self {
        // We can't directly clone the inner Box<dyn UIElementImpl>,
        // but we can create a new UIElement with the same identity
        // that will behave the same way
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

/// Utility functions for working with UI elements
pub mod utils {
    use super::*;

    /// Get the display text for an element (name, value, or role as fallback)
    pub fn display_text(element: &UIElement) -> String {
        element.name()
            .or_else(|| element.attributes().value)
            .unwrap_or_else(|| element.role())
    }

    /// Check if element has any text content
    pub fn has_text_content(element: &UIElement) -> bool {
        element.name().is_some() 
            || element.attributes().value.is_some()
            || element.text(1).unwrap_or_default().trim().len() > 0
    }

    /// Get a human-readable identifier for the element
    pub fn element_identifier(element: &UIElement) -> String {
        if let Some(name) = element.name() {
            format!("{} ({})", name, element.role())
        } else if let Some(id) = element.id() {
            format!("#{} ({})", id, element.role())
        } else {
            element.role()
        }
    }

    /// Create a minimal attributes struct with just the essentials
    pub fn essential_attributes(element: &UIElement) -> UIElementAttributes {
        UIElementAttributes {
            role: element.role(),
            name: element.name(),
            value: element.attributes().value,
            ..Default::default()
        }
    }
}
