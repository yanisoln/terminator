use crate::{
    KeyboardEvent, MouseButton, MouseEvent, MouseEventType, Position, UiElement,
    WorkflowEvent, Result, WorkflowRecorderConfig
};
use std::{
    sync::{Arc, Mutex, mpsc},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::sync::broadcast;
use tracing::{debug, info, error};
use rdev::{EventType, Button, Key};
use uiautomation::{UIAutomation, UIElement as WinUIElement};
use uiautomation::types::{Point, UIProperty};
use windows::Win32::Foundation::POINT;

/// The Windows-specific recorder
pub struct WindowsRecorder {
    /// The event sender
    event_tx: broadcast::Sender<WorkflowEvent>,
    
    /// The configuration
    config: WorkflowRecorderConfig,
    
    /// The last mouse position
    last_mouse_pos: Arc<Mutex<Option<(i32, i32)>>>,

    /// Signal to stop the listener thread
    stop_indicator: Arc<AtomicBool>,
}

#[cfg(target_os = "windows")]
impl WindowsRecorder {
    /// Create a new Windows recorder
    pub fn new(
        config: WorkflowRecorderConfig,
        event_tx: broadcast::Sender<WorkflowEvent>,
    ) -> Result<Self> {
        info!("[LOG] windows.rs loaded");
        debug!("Initializing Windows recorder with config: {:?}", config);
        
        let last_mouse_pos = Arc::new(Mutex::new(None));
        let stop_indicator = Arc::new(AtomicBool::new(false));
        
        let mut recorder = Self {
            event_tx,
            config,
            last_mouse_pos,
            stop_indicator,
        };
        
        // Set up event listener
        debug!("Setting up event listener...");
        recorder.setup_event_listener()?;
        debug!("Event listener setup completed");
        
        Ok(recorder)
    }
    
    /// Set up the event listener
    fn setup_event_listener(&mut self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let last_mouse_pos = Arc::clone(&self.last_mouse_pos);
        let capture_ui_elements = self.config.capture_ui_elements;
        let stop_indicator_clone = Arc::clone(&self.stop_indicator);
        
        std::thread::spawn(move || {
            // Create a new UIAutomation instance in this thread
            let automation = match UIAutomation::new() {
                Ok(auto) => auto,
                Err(e) => {
                    error!("Failed to create UIAutomation instance: {}", e);
                    return;
                }
            };
            
            let callback_stop_indicator = stop_indicator_clone.clone();
            if let Err(error) = rdev::listen(move |event| {
                if callback_stop_indicator.load(Ordering::SeqCst) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let keyboard_event = KeyboardEvent {
                            key_code: key_to_u32(&key),
                            is_key_down: true,
                            ctrl_pressed: false, // TODO: Track modifier state
                            alt_pressed: false,
                            shift_pressed: false,
                            win_pressed: false,
                        };
                        let _ = event_tx.send(WorkflowEvent::Keyboard(keyboard_event));
                    }
                    EventType::KeyRelease(key) => {
                        let keyboard_event = KeyboardEvent {
                            key_code: key_to_u32(&key),
                            is_key_down: false,
                            ctrl_pressed: false, // TODO: Track modifier state
                            alt_pressed: false,
                            shift_pressed: false,
                            win_pressed: false,
                        };
                        let _ = event_tx.send(WorkflowEvent::Keyboard(keyboard_event));
                    }
                    EventType::ButtonPress(button) => {
                        if let Some((x, y)) = *last_mouse_pos.lock().unwrap() {
                            let button = match button {
                                Button::Left => MouseButton::Left,
                                Button::Right => MouseButton::Right,
                                Button::Middle => MouseButton::Middle,
                                _ => return,
                            };
                            
                            let mut ui_element = None;
                            if capture_ui_elements {
                                ui_element = get_ui_element_at_point(&automation, x, y);
                            }
                            
                            let mouse_event = MouseEvent {
                                event_type: MouseEventType::Down,
                                button,
                                position: Position { x, y },
                                ui_element,
                            };
                            let _ = event_tx.send(WorkflowEvent::Mouse(mouse_event));
                        }
                    }
                    EventType::ButtonRelease(button) => {
                        if let Some((x, y)) = *last_mouse_pos.lock().unwrap() {
                            let button = match button {
                                Button::Left => MouseButton::Left,
                                Button::Right => MouseButton::Right,
                                Button::Middle => MouseButton::Middle,
                                _ => return,
                            };
                            
                            let mut ui_element = None;
                            if capture_ui_elements {
                                ui_element = get_ui_element_at_point(&automation, x, y);
                            }
                            
                            let mouse_event = MouseEvent {
                                event_type: MouseEventType::Up,
                                button,
                                position: Position { x, y },
                                ui_element,
                            };
                            let _ = event_tx.send(WorkflowEvent::Mouse(mouse_event));
                        }
                    }
                    EventType::MouseMove { x, y } => {
                        let x = x as i32;
                        let y = y as i32;
                        *last_mouse_pos.lock().unwrap() = Some((x, y));
                        
                        let mouse_event = MouseEvent {
                            event_type: MouseEventType::Move,
                            button: MouseButton::Left,
                            position: Position { x, y },
                            ui_element: None,
                        };
                        let _ = event_tx.send(WorkflowEvent::Mouse(mouse_event));
                    }
                    EventType::Wheel { delta_x: _, delta_y: _ } => {
                        if let Some((x, y)) = *last_mouse_pos.lock().unwrap() {
                            let mouse_event = MouseEvent {
                                event_type: MouseEventType::Wheel,
                                button: MouseButton::Middle,
                                position: Position { x, y },
                                ui_element: None,
                            };
                            let _ = event_tx.send(WorkflowEvent::Mouse(mouse_event));
                        }
                    }
                }
            }) {
                error!("Failed to listen for events: {:?}", error);
            }
            info!("Rdev event listener thread has finished or encountered an error.");
        });
        
        Ok(())
    }
    
    /// Stop recording
    pub fn stop(&self) -> Result<()> {
        debug!("Attempting to stop Windows recorder...");
        self.stop_indicator.store(true, Ordering::SeqCst);
        info!("Windows recorder stop signal sent. Event processing will cease. The underlying rdev listener may remain active until program termination.");
        Ok(())
    }
}

/// Convert a Key to a u32
fn key_to_u32(key: &Key) -> u32 {
    match key {
        Key::KeyA => 0x41,
        Key::KeyB => 0x42,
        Key::KeyC => 0x43,
        Key::KeyD => 0x44,
        Key::KeyE => 0x45,
        Key::KeyF => 0x46,
        Key::KeyG => 0x47,
        Key::KeyH => 0x48,
        Key::KeyI => 0x49,
        Key::KeyJ => 0x4A,
        Key::KeyK => 0x4B,
        Key::KeyL => 0x4C,
        Key::KeyM => 0x4D,
        Key::KeyN => 0x4E,
        Key::KeyO => 0x4F,
        Key::KeyP => 0x50,
        Key::KeyQ => 0x51,
        Key::KeyR => 0x52,
        Key::KeyS => 0x53,
        Key::KeyT => 0x54,
        Key::KeyU => 0x55,
        Key::KeyV => 0x56,
        Key::KeyW => 0x57,
        Key::KeyX => 0x58,
        Key::KeyY => 0x59,
        Key::KeyZ => 0x5A,
        Key::Num0 => 0x30,
        Key::Num1 => 0x31,
        Key::Num2 => 0x32,
        Key::Num3 => 0x33,
        Key::Num4 => 0x34,
        Key::Num5 => 0x35,
        Key::Num6 => 0x36,
        Key::Num7 => 0x37,
        Key::Num8 => 0x38,
        Key::Num9 => 0x39,
        Key::Escape => 0x1B,
        Key::Backspace => 0x08,
        Key::Tab => 0x09,
        Key::Return => 0x0D,
        Key::Space => 0x20,
        Key::LeftArrow => 0x25,
        Key::UpArrow => 0x26,
        Key::RightArrow => 0x27,
        Key::DownArrow => 0x28,
        Key::Delete => 0x2E,
        Key::Home => 0x24,
        Key::End => 0x23,
        Key::PageUp => 0x21,
        Key::PageDown => 0x22,
        Key::F1 => 0x70,
        Key::F2 => 0x71,
        Key::F3 => 0x72,
        Key::F4 => 0x73,
        Key::F5 => 0x74,
        Key::F6 => 0x75,
        Key::F7 => 0x76,
        Key::F8 => 0x77,
        Key::F9 => 0x78,
        Key::F10 => 0x79,
        Key::F11 => 0x7A,
        Key::F12 => 0x7B,
        Key::ShiftLeft => 0xA0,
        Key::ShiftRight => 0xA1,
        Key::ControlLeft => 0xA2,
        Key::ControlRight => 0xA3,
        Key::Alt => 0xA4,
        Key::AltGr => 0xA5,
        Key::MetaLeft => 0x5B,
        Key::MetaRight => 0x5C,
        _ => 0,
    }
}

/// Get the UI element at the given point
#[cfg(target_os = "windows")]
fn get_ui_element_at_point(automation: &UIAutomation, x: i32, y: i32) -> Option<UiElement> {
    debug!("Getting UI element at point ({}, {})", x, y);
    
    match automation.element_from_point(Point::from(POINT { x, y })) {
        Ok(element) => {
            debug!("Found UI element, gathering properties...");
            
            let name = element.get_name().ok();
            let automation_id = element.get_automation_id().ok();
            let class_name = element.get_classname().ok();
            let control_type = element.get_control_type().ok().map(|ct| ct.to_string());
            let process_id = element.get_process_id().ok().map(|pid| pid as u32);
            
            let is_enabled = element.is_enabled().ok();
            let has_keyboard_focus = element.has_keyboard_focus().ok();
            let value = element.get_property_value(UIProperty::Name).ok().map(|v| v.to_string());
            
            let bounding_rect = element.get_bounding_rectangle().ok().map(|rect| {
                crate::events::Rect {
                    x: rect.get_left() as i32,
                    y: rect.get_top() as i32,
                    width: rect.get_width() as i32,
                    height: rect.get_height() as i32,
                }
            });
            
            let hierarchy_path = get_element_hierarchy_path(&element);
            
            let (window_title, application_name) = if let Some(pid) = process_id {
                debug!("Getting window info for process {}", pid);
                get_window_info_for_process(pid)
            } else {
                (None, None)
            };
            
            debug!(
                "UI element properties: name={:?}, automation_id={:?}, class={:?}, type={:?}, process={:?}, app={:?}, window={:?}",
                name, automation_id, class_name, control_type, process_id, application_name, window_title
            );
            
            Some(UiElement {
                name,
                automation_id,
                class_name,
                control_type,
                process_id,
                application_name,
                window_title,
                bounding_rect,
                is_enabled,
                has_keyboard_focus,
                hierarchy_path,
                value,
            })
        }
        Err(e) => {
            debug!("Failed to get UI element: {}", e);
            None
        }
    }
}

/// Get the hierarchy path for an element
#[cfg(target_os = "windows")]
fn get_element_hierarchy_path(element: &WinUIElement) -> Option<String> {
    let mut path = Vec::new();
    let mut current = Some(element.clone());
    
    while let Some(elem) = current {
        let name = elem.get_name().ok().unwrap_or_default();
        let control_type = elem.get_control_type().ok().map(|ct| ct.to_string()).unwrap_or_default();
        let automation_id = elem.get_automation_id().ok().unwrap_or_default();
        
        let identifier = if !automation_id.is_empty() {
            format!("{}[{}]", control_type, automation_id)
        } else if !name.is_empty() {
            format!("{}[{}]", control_type, name)
        } else {
            control_type
        };
        
        path.push(identifier);
        
        current = elem.get_cached_parent().ok();
    }
    
    path.reverse();
    
    if path.is_empty() {
        None
    } else {
        Some(path.join("/"))
    }
}

/// Get window information for the given process ID
#[cfg(target_os = "windows")]
fn get_window_info_for_process(process_id: u32) -> (Option<String>, Option<String>) {
    let (tx, rx) = mpsc::channel();
    let pid_clone = process_id;

    std::thread::spawn(move || {
        let app_name = match get_process_name_by_pid_recorder(pid_clone as i32) {
            Ok(name) => Some(name),
            Err(e) => {
                error!("Failed to get process name for PID {} (within get_window_info_for_process thread): {}", pid_clone, e);
                None
            }
        };

        let window_title = match UIAutomation::new() {
            Ok(automation) => {
                match automation.get_root_element() {
                    Ok(root) => {
                        let condition = match automation.create_property_condition(
                            UIProperty::ProcessId,
                            (pid_clone as i32).into(),
                            None,
                        ) {
                            Ok(cond) => cond,
                            Err(e) => {
                                error!("Failed to create process ID condition for PID {}: {}", pid_clone, e);
                                let _ = tx.send((None, app_name));
                                return;
                            }
                        };

                        match root.find_first(uiautomation::types::TreeScope::Children, &condition)
                            .or_else(|e| {
                                debug!("Could not find window for PID {} in children (error: {}), trying subtree.", pid_clone, e);
                                root.find_first(uiautomation::types::TreeScope::Subtree, &condition)
                            }) {
                            Ok(app_element) => app_element.get_name().ok(),
                            Err(e) => {
                                error!(
                                    "Failed to find app element for PID {}: {}",
                                    pid_clone,
                                    e
                                );
                                None
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get root UI element for PID {}: {}", pid_clone, e);
                        None
                    }
                }
            }
            Err(e) => {
                error!("Failed to create UIAutomation instance for PID {}: {}", pid_clone, e);
                None
            }
        };
        let _ = tx.send((window_title, app_name));
    });

    let timeout_duration = Duration::from_secs(3);

    match rx.recv_timeout(timeout_duration) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            error!("Timeout getting window info for PID {}", process_id);
            let app_name_fallback = match get_process_name_by_pid_recorder(process_id as i32) {
                Ok(name) => Some(name),
                Err(e) => {
                    error!("Fallback failed to get process name for PID {} after timeout: {}", process_id, e);
                    None
                }
            };
            (None, app_name_fallback)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            error!("Channel disconnected while getting window info for PID {}. This might indicate the spawned thread panicked.", process_id);
            let app_name_fallback = match get_process_name_by_pid_recorder(process_id as i32) {
                Ok(name) => Some(name),
                Err(e) => {
                    error!("Fallback failed to get process name for PID {} after disconnect: {}", process_id, e);
                    None
                }
            };
            (None, app_name_fallback)
        }
    }
}

// Helper function to get process name by PID using PowerShell, with a timeout
fn get_process_name_by_pid_recorder(pid: i32) -> std::result::Result<String, String> {
    let (tx, rx) = mpsc::channel();
    let command_str = format!(
        "Get-Process -Id {} | Select-Object -ExpandProperty ProcessName",
        pid
    );

    std::thread::spawn(move || {
        let result = match std::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "hidden", "-Command", &command_str])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if process_name.is_empty() {
                        Err(format!("Process name not found for PID {} (stdout was empty)", pid))
                    } else {
                        Ok(process_name)
                    }
                } else {
                    let err_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    Err(format!(
                        "PowerShell command failed to get process name for PID {}: {}",
                        pid, err_msg
                    ))
                }
            }
            Err(e) => Err(format!(
                "Failed to execute PowerShell to get process name for PID {}: {}",
                pid, e
            )),
        };
        let _ = tx.send(result);
    });

    let timeout_duration = Duration::from_secs(2);

    match rx.recv_timeout(timeout_duration) {
        Ok(result_from_thread) => result_from_thread,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(format!("Timeout getting process name for PID {} via PowerShell", pid)),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(format!("Channel disconnected while getting process name for PID {} via PowerShell. Thread might have panicked.", pid)),
    }
} 



