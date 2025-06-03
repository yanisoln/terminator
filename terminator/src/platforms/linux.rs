use crate::element::UIElementImpl;
use crate::platforms::AccessibilityEngine;
use crate::{ClickResult, CommandOutput, ScreenshotResult, UINode};
use crate::{AutomationError, Locator, Selector, UIElement, UIElementAttributes};
use std::fmt::Debug;
use std::time::Duration;
use std::default::Default;
use std::sync::Arc;
use std::process::Command;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, warn};
use tokio::runtime::Runtime;
use std::cell::RefCell;
thread_local! {
    static THREAD_LOCAL_RUNTIME: RefCell<Option<tokio::runtime::Runtime>> = RefCell::new(None);
}

use atspi::{
    connection::set_session_accessibility,
    proxy::accessible::{AccessibleProxy, ObjectRefExt},
    zbus::{proxy::CacheProperties, Connection},
    AccessibilityConnection, Role,
};
use atspi_proxies::{
    component::ComponentProxy,
    action::ActionProxy,
    text::TextProxy,
    device_event_controller::{DeviceEventControllerProxy, KeySynthType},
};
use atspi_common::{CoordType, state};
use zbus::fdo::DBusProxy;
use anyhow::anyhow;

fn block_on_in_thread_local<F: std::future::Future<Output = T>, T>(future: F) -> T {
    THREAD_LOCAL_RUNTIME.with(|rt_cell| {
        let mut rt_opt = rt_cell.borrow_mut();
        if rt_opt.is_none() {
            *rt_opt = Some(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));
        }
        rt_opt.as_ref().unwrap().block_on(future)
    })
}

// Linux-specific error handling
impl From<zbus::Error> for AutomationError {
    fn from(error: zbus::Error) -> Self {
        AutomationError::PlatformError(error.to_string())
    }
}

impl From<atspi_proxies::AtspiError> for AutomationError {
    fn from(error: atspi_proxies::AtspiError) -> Self {
        AutomationError::PlatformError(error.to_string())
    }
}

// Define our own State enum that matches the AT-SPI states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Disabled,
    Hidden,
    Focused,
    Focusable,
    Active,
}

const REGISTRY_DEST: &str = "org.a11y.atspi.Registry";
const REGISTRY_PATH: &str = "/org/a11y/atspi/accessible/root";
const ACCESSIBLE_INTERFACE: &str = "org.a11y.atspi.Accessible";

// Thread-safe wrapper for AccessibleProxy
#[derive(Debug, Clone)]
pub struct ThreadSafeLinuxUIElement(pub Arc<AccessibleProxy<'static>>);
unsafe impl Send for ThreadSafeLinuxUIElement {}
unsafe impl Sync for ThreadSafeLinuxUIElement {}

// Linux Engine struct
#[derive(Clone)]
pub struct LinuxEngine {
    connection: Arc<Connection>,
    root: ThreadSafeLinuxUIElement,
}

// Update LinuxUIElement to store Arc<Connection>, destination: String, and path: String instead of a proxy. Update struct definition only in this step.
#[derive(Debug, Clone)]
pub struct LinuxUIElement {
    connection: Arc<Connection>,
    destination: String,
    path: String,
}

impl LinuxEngine {
    pub fn new(_use_background_apps: bool, _activate_app: bool) -> Result<Self, AutomationError> {
        block_on_in_thread_local(async {
            set_session_accessibility(true).await?;
            let accessibility_connection = AccessibilityConnection::new().await?;
            let connection = Arc::new(accessibility_connection.connection().clone());
            let registry = AccessibleProxy::builder(&connection)
                .destination(REGISTRY_DEST)?
                .path(REGISTRY_PATH)?
                .interface(ACCESSIBLE_INTERFACE)?
                .cache_properties(CacheProperties::No)
                .build()
                .await?;

            let root = ThreadSafeLinuxUIElement(Arc::new(registry));
            Ok(Self {
                connection,
                root,
            })
        })
    }
}

#[async_trait::async_trait]
impl AccessibilityEngine for LinuxEngine {
    fn get_root_element(&self) -> UIElement {
        UIElement::new(Box::new(LinuxUIElement {
            connection: Arc::clone(&self.connection),
            destination: self.root.0.inner().destination().to_string(),
            path: self.root.0.inner().path().to_string(),
        }))
    }

    fn get_element_by_id(&self, _id: i32) -> Result<UIElement, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "get_element_by_id not yet implemented for Linux".to_string(),
        ))
    }

    fn get_focused_element(&self) -> Result<UIElement, AutomationError> {
        block_on_in_thread_local(async {
            let focused = self.root.0.get_state().await?;
            if focused.contains(state::State::Focused) {
                Ok(UIElement::new(Box::new(LinuxUIElement {
                    connection: Arc::clone(&self.connection),
                    destination: self.root.0.inner().destination().to_string(),
                    path: self.root.0.inner().path().to_string(),
                })))
            } else {
                Err(AutomationError::ElementNotFound("No focused element found".to_string()))
            }
        })
    }

    fn get_applications(&self) -> Result<Vec<UIElement>, AutomationError> {
        block_on_in_thread_local(async {
            let apps = self.root.0.get_children().await
                .map_err(|e: zbus::Error| AutomationError::PlatformError(e.to_string()))?;
            let mut elements = Vec::new();
            for app in apps {
                let proxy = app.into_accessible_proxy(self.connection.as_ref())
                    .await
                    .map_err(|e: zbus::Error| AutomationError::PlatformError(e.to_string()))?;
                if proxy.child_count().await? > 0 {
                    elements.push(UIElement::new(Box::new(LinuxUIElement {
                        connection: Arc::clone(&self.connection),
                        destination: proxy.inner().destination().to_string(),
                        path: proxy.inner().path().to_string(),
                    })));
                }
            }
            Ok(elements)
        })
    }

    fn get_application_by_name(&self, name: &str) -> Result<UIElement, AutomationError> {
        let selector = Selector::Role {
            role: "application".to_string(),
            name: Some(name.to_string()),
        };
        self.find_element(&selector, None, None)
    }

    fn get_application_by_pid(&self, pid: i32, _timeout: Option<Duration>) -> Result<UIElement, AutomationError> {
        let selector = Selector::Role {
            role: "application".to_string(),
            name: Some(pid.to_string()),
        };
        self.find_element(&selector, None, None)
    }

    fn find_element(
        &self,
        _selector: &Selector,
        _root: Option<&UIElement>,
        _timeout: Option<Duration>,
    ) -> Result<UIElement, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn find_elements(
        &self,
        _selector: &Selector,
        _root: Option<&UIElement>,
        _timeout: Option<Duration>,
        _depth: Option<usize>,
    ) -> Result<Vec<UIElement>, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn open_application(&self, app_name: &str) -> Result<UIElement, AutomationError> {
        block_on_in_thread_local(async {
            // Try to find existing application first
            let selector = Selector::Role {
                role: "application".to_string(),
                name: Some(app_name.to_string()),
            };
            if let Ok(app) = self.find_element(&selector, None, None) {
                return Ok(app);
            }
            // Launch the application
            let output = Command::new(app_name)
                .spawn()
                .map_err(|e| AutomationError::PlatformError(e.to_string()))?;
            // Wait for the application to appear
            let pid = output.id() as i32;
            let selector = Selector::Role {
                role: "application".to_string(),
                name: Some(pid.to_string()),
            };
            for _ in 0..10 {
                if let Ok(app) = self.find_element(&selector, None, None) {
                    return Ok(app);
                }
                sleep(Duration::from_millis(500)).await;
            }
            Err(AutomationError::ElementNotFound(format!("Application '{}' not found after launch", app_name)))
        })
    }

    fn activate_application(&self, app_name: &str) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let selector = Selector::Role {
                role: "application".to_string(),
                name: Some(app_name.to_string()),
            };
            if let Ok(app) = self.find_element(&selector, None, None) {
                // Try to find a window to activate
                let mut windows = Vec::new();
                for child in app.children()? {
                    if child.role() == "window" {
                        windows.push(child);
                    }
                }
                if let Some(window) = windows.first() {
                    window.focus()?;
                    return Ok(());
                }
                // If no window found, try focusing the application itself
                app.focus()?;
                Ok(())
            } else {
                Err(AutomationError::ElementNotFound(format!("Application '{}' not found", app_name)))
            }
        })
    }

    fn open_url(&self, _url: &str, _browser: Option<&str>) -> Result<UIElement, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn open_file(&self, _file_path: &str) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    async fn run_command(
        &self,
        _windows_command: Option<&str>,
        _unix_command: Option<&str>,
    ) -> Result<CommandOutput, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    async fn capture_screen(&self) -> Result<ScreenshotResult, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    async fn capture_monitor_by_name(&self, _name: &str) -> Result<ScreenshotResult, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    async fn ocr_image_path(&self, _image_path: &str) -> Result<String, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    async fn ocr_screenshot(&self, _screenshot: &ScreenshotResult) -> Result<String, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn activate_browser_window_by_title(&self, _title: &str) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    async fn get_current_browser_window(&self) -> Result<UIElement, AutomationError> {
        Err(AutomationError::UnsupportedOperation(
            "get_current_browser_window not yet implemented for Linux".to_string(),
        ))
    }

    async fn get_current_window(&self) -> Result<UIElement, AutomationError> {
        Err(AutomationError::UnsupportedOperation("get_current_window not yet implemented for Linux".to_string()))
    }

    async fn get_current_application(&self) -> Result<UIElement, AutomationError> {
        Err(AutomationError::UnsupportedOperation("get_current_application not yet implemented for Linux".to_string()))
    }

    fn get_window_tree(
        &self, 
        pid: u32, 
        title: Option<&str>, 
        _config: crate::platforms::TreeBuildConfig
    ) -> Result<crate::UINode, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            format!("get_window_tree for PID {} and title {:?} not yet implemented for Linux", pid, title)
        ))
    }

    async fn get_active_monitor_name(&self) -> Result<String, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UIElementImpl for LinuxUIElement {
    fn object_id(&self) -> usize {
        self.path.len()
    }

    fn id(&self) -> Option<String> {
        Some(self.path.clone())
    }

    fn role(&self) -> String {
        block_on_in_thread_local(async {
            let builder = AccessibleProxy::builder(&self.connection);
            let builder = match builder.destination(self.destination.as_str()) {
                Ok(b) => b,
                Err(_) => return String::new(),
            };
            let builder = match builder.path(self.path.as_str()) {
                Ok(b) => b,
                Err(_) => return String::new(),
            };
            let proxy = match builder.build().await {
                Ok(p) => p,
                Err(_) => return String::new(),
            };
            proxy.get_role().await.map(|r| r.to_string()).unwrap_or_default()
        })
    }

    fn attributes(&self) -> UIElementAttributes {
        let mut attrs = UIElementAttributes::default();
        attrs.role = self.role();
        attrs.name = self.name();
        if let Ok(states) = self.is_enabled() {
            attrs.value = Some(states.to_string());
        }
        attrs
    }

    fn name(&self) -> Option<String> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str()).ok()?
                .path(self.path.as_str()).ok()?
                .build().await.ok()?;
            proxy.name().await.ok()
        })
    }

    fn children(&self) -> Result<Vec<UIElement>, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let children = proxy.get_children().await
                .map_err(|e: zbus::Error| AutomationError::PlatformError(e.to_string()))?;
            let mut elements = Vec::new();
            for child in children {
                let proxy = child.into_accessible_proxy(self.connection.as_ref())
                    .await
                    .map_err(|e: zbus::Error| AutomationError::PlatformError(e.to_string()))?;
                elements.push(UIElement::new(Box::new(LinuxUIElement {
                    connection: Arc::clone(&self.connection),
                    destination: proxy.inner().destination().to_string(),
                    path: proxy.inner().path().to_string(),
                })));
            }
            Ok(elements)
        })
    }

    fn parent(&self) -> Result<Option<UIElement>, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            if let Ok(parent) = proxy.parent().await {
                let proxy = parent.into_accessible_proxy(self.connection.as_ref())
                    .await
                    .map_err(|e: zbus::Error| AutomationError::PlatformError(e.to_string()))?;
                Ok(Some(UIElement::new(Box::new(LinuxUIElement {
                    connection: Arc::clone(&self.connection),
                    destination: proxy.inner().destination().to_string(),
                    path: proxy.inner().path().to_string(),
                }))))
            } else {
                Ok(None)
            }
        })
    }

    fn bounds(&self) -> Result<(f64, f64, f64, f64), AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            Ok((extents.0 as f64, extents.1 as f64, extents.2 as f64, extents.3 as f64))
        })
    }

    fn click(&self) -> Result<ClickResult, AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let x = extents.0 + (extents.2 / 2);
            let y = extents.1 + (extents.3 / 2);
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(x, y, "b1p").await?;
            device_controller.generate_mouse_event(x, y, "b1r").await?;
            Ok(ClickResult {
                method: "click".to_string(),
                coordinates: Some((x as f64, y as f64)),
                details: format!("Clicked at ({}, {})", x, y),
            })
        })
    }

    fn double_click(&self) -> Result<ClickResult, AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let x = extents.0 + (extents.2 / 2);
            let y = extents.1 + (extents.3 / 2);
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(x, y, "b1p").await?;
            device_controller.generate_mouse_event(x, y, "b1c").await?;
            device_controller.generate_mouse_event(x, y, "b1c").await?;
            device_controller.generate_mouse_event(x, y, "b1r").await?;
            Ok(ClickResult {
                method: "double_click".to_string(),
                coordinates: Some((x as f64, y as f64)),
                details: format!("Double clicked at ({}, {})", x, y),
            })
        })
    }

    fn right_click(&self) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let x = extents.0 + (extents.2 / 2);
            let y = extents.1 + (extents.3 / 2);
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(x, y, "b3p").await?;
            device_controller.generate_mouse_event(x, y, "b3r").await?;
            Ok(())
        })
    }

    fn hover(&self) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let x = extents.0 + (extents.2 / 2);
            let y = extents.1 + (extents.3 / 2);
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(x, y, "abs").await?;
            Ok(())
        })
    }

    fn focus(&self) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            if !states.contains(state::State::Focusable) {
                return Err(AutomationError::UnsupportedOperation("Element is not focusable".to_string()));
            }
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_keyboard_event(0, "", KeySynthType::Press).await?;
            device_controller.generate_keyboard_event(0, "", KeySynthType::Release).await?;
            Ok(())
        })
    }

    fn type_text(&self, text: &str, _use_clipboard: bool) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            if !states.contains(state::State::Focusable) {
                return Err(AutomationError::UnsupportedOperation("Element is not focusable".to_string()));
            }
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_keyboard_event(0, "", KeySynthType::Press).await?;
            device_controller.generate_keyboard_event(0, "", KeySynthType::Release).await?;
            if let Ok(text_proxy) = TextProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await
            {
                let char_count = text_proxy.character_count().await?;
                text_proxy.get_text(0, char_count).await?;
                text_proxy.get_text(0, text.len() as i32).await?;
                Ok(())
            } else {
                for c in text.chars() {
                    device_controller.generate_keyboard_event(c as i32, &c.to_string(), KeySynthType::String).await?;
                }
                Ok(())
            }
        })
    }

    fn press_key(&self, key: &str) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let keysym = match key.to_lowercase().as_str() {
                "enter" => 0xff0d,
                "tab" => 0xff09,
                "space" => 0x020,
                "backspace" => 0xff08,
                "delete" => 0xffff,
                "escape" => 0xff1b,
                "up" => 0xff52,
                "down" => 0xff54,
                "left" => 0xff51,
                "right" => 0xff53,
                _ => key.chars().next().map(|c| c as i32).unwrap_or(0),
            };
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_keyboard_event(keysym, "", KeySynthType::Press).await?;
            device_controller.generate_keyboard_event(keysym, "", KeySynthType::Release).await?;
            Ok(())
        })
    }

    fn mouse_click_and_hold(&self, x: f64, y: f64) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let abs_x = (extents.0 as f64 + x) as i32;
            let abs_y = (extents.1 as f64 + y) as i32;
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(abs_x, abs_y, "b1p").await?;
            Ok(())
        })
    }

    fn mouse_move(&self, x: f64, y: f64) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let abs_x = (extents.0 as f64 + x) as i32;
            let abs_y = (extents.1 as f64 + y) as i32;
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(abs_x, abs_y, "abs").await?;
            Ok(())
        })
    }

    fn mouse_release(&self) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let component = ComponentProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let extents = component.get_extents(CoordType::Screen).await?;
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_mouse_event(extents.0, extents.1, "b1r").await?;
            Ok(())
        })
    }

    fn mouse_drag(&self, _start_x: f64, _start_y: f64, _end_x: f64, _end_y: f64) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn get_text(&self, _max_depth: usize) -> Result<String, AutomationError> {
        block_on_in_thread_local(async {
            let text_proxy = TextProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let char_count = text_proxy.character_count().await?;
            Ok(text_proxy.get_text(0, char_count).await?)
        })
    }

    fn is_keyboard_focusable(&self) -> Result<bool, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            Ok(states.contains(state::State::Focusable))
        })
    }

    fn perform_action(&self, action: &str) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let action_proxy = ActionProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let actions = action_proxy.get_actions().await?;
            if let Some(action_index) = actions.iter().position(|a| a.name == action) {
                action_proxy.do_action(action_index as i32).await?;
                Ok(())
            } else {
                Err(AutomationError::UnsupportedOperation(
                    format!("Action '{}' not supported for this element", action)
                ))
            }
        })
    }

    fn create_locator(&self, _selector: Selector) -> Result<Locator, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn scroll(&self, _direction: &str, _amount: f64) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn application(&self) -> Result<Option<UIElement>, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            if let Ok(application) = proxy.get_application().await {
                Ok(Some(UIElement::new(Box::new(LinuxUIElement {
                    connection: Arc::clone(&self.connection),
                    destination: application.name.to_string(),
                    path: application.path.to_string(),
                }))))
            } else {
                Ok(None)
            }
        })

    }

    fn window(&self) -> Result<Option<UIElement>, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn highlight(&self, _color: Option<u32>, _duration: Option<std::time::Duration>) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn activate_window(&self) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn process_id(&self) -> Result<u32, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            if let Ok(application) = proxy.get_application().await {
                // Create DBusProxy to get the PID from the application
                let dbus_proxy = DBusProxy::new(&self.connection).await?;
                if let Ok(unique_name) = dbus_proxy.get_name_owner((&application.name).into()).await {
                    if let Ok(pid) = dbus_proxy.get_connection_unix_process_id(unique_name.into()).await {
                        return Ok(pid);
                    }
                }
            }
            Err(AutomationError::PlatformError(
                format!("Failed to get process ID for application: {}", self.destination)
            ))
        })
    }

    fn clone_box(&self) -> Box<dyn UIElementImpl> {
        Box::new(LinuxUIElement {
            connection: Arc::clone(&self.connection),
            destination: self.destination.clone(),
            path: self.path.clone(),
        })
    }

    fn is_enabled(&self) -> Result<bool, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            Ok(states.contains(state::State::Enabled))
        })
    }

    fn is_visible(&self) -> Result<bool, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            Ok(states.contains(state::State::Visible))
        })
    }

    fn is_focused(&self) -> Result<bool, AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            Ok(states.contains(state::State::Focused))
        })
    }

    fn set_value(&self, value: &str) -> Result<(), AutomationError> {
        block_on_in_thread_local(async {
            let proxy = AccessibleProxy::builder(&self.connection)
                .destination(self.destination.as_str())?
                .path(self.path.as_str())?
                .build()
                .await?;
            let states = proxy.get_state().await?;
            if !states.contains(state::State::Focusable) {
                return Err(AutomationError::UnsupportedOperation("Element is not focusable".to_string()));
            }
            let device_controller = DeviceEventControllerProxy::new(&self.connection).await?;
            device_controller.generate_keyboard_event(0, "", KeySynthType::Press).await?;
            device_controller.generate_keyboard_event(0, "", KeySynthType::Release).await?;
            for c in value.chars() {
                device_controller.generate_keyboard_event(c as i32, &c.to_string(), KeySynthType::String).await
                    .map_err(|e: zbus::Error| AutomationError::PlatformError(e.to_string()))?;
            }
            Ok(())
        })
    }

    fn capture(&self) -> Result<ScreenshotResult, AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn close(&self) -> Result<(), AutomationError> {
        Err(AutomationError::UnsupportedPlatform(
            "Linux implementation is not yet available".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_linux_engine_creation() {
        let engine_result = LinuxEngine::new(false, false);
        assert!(engine_result.is_ok(), "Should be able to create Linux engine");

        if let Ok(engine) = engine_result {
            let root = engine.get_root_element();
            assert!(root.id().is_some(), "Root element should have an ID");
            assert_eq!(root.role(), "desktop frame", "Root element should have 'desktop frame' role");
        }
    }

}

