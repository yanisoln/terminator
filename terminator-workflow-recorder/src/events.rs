use serde::{Deserialize, Serialize};
use terminator::UIElement;
use std::time::SystemTime;


/// Represents a position on the screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}


/// Represents a rectangle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Represents the type of mouse button
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Represents the type of mouse event
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseEventType {
    Click,
    DoubleClick,
    RightClick,
    Down,
    Up,
    Move,
    Wheel,
    DragStart,
    DragEnd,
    Drop,
}

/// Represents a keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    /// The key code
    pub key_code: u32,
    
    /// Whether the key was pressed or released
    pub is_key_down: bool,
    
    /// Whether the Ctrl key was pressed
    pub ctrl_pressed: bool,
    
    /// Whether the Alt key was pressed
    pub alt_pressed: bool,
    
    /// Whether the Shift key was pressed
    pub shift_pressed: bool,
    
    /// Whether the Win key was pressed
    pub win_pressed: bool,
    
    /// Character representation of the key (if printable)
    pub character: Option<char>,
    
    /// Raw scan code
    pub scan_code: Option<u32>,
    
    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents a mouse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    /// The type of mouse event
    pub event_type: MouseEventType,
    
    /// The mouse button
    pub button: MouseButton,
    
    /// The position of the mouse
    pub position: Position,
    
    /// Scroll delta for wheel events
    pub scroll_delta: Option<(i32, i32)>,
    
    /// Drag start position (for drag events)
    pub drag_start: Option<Position>,
    
    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents clipboard actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipboardAction {
    Copy,
    Cut,
    Paste,
    Clear,
}

/// Represents a clipboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEvent {
    /// The clipboard action
    pub action: ClipboardAction,
    
    /// The content that was copied/cut/pasted (truncated if too long)
    pub content: Option<String>,
    
    /// The size of the content in bytes
    pub content_size: Option<usize>,
    
    /// The format of the clipboard data
    pub format: Option<String>,
    
    /// Whether the content was truncated due to size
    pub truncated: bool,
    
    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents text selection events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSelectionEvent {
    /// The selected text content
    pub selected_text: String,
    
    /// The start position of the selection (screen coordinates)
    pub start_position: Position,
    
    /// The end position of the selection (screen coordinates)
    pub end_position: Position,
    
    /// The selection method (mouse drag, keyboard shortcuts, etc.)
    pub selection_method: SelectionMethod,
    
    /// The length of the selection in characters
    pub selection_length: usize,
    
    /// Whether this is a partial selection within a larger text block
    pub is_partial_selection: bool,
    
    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents how text was selected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    MouseDrag,
    DoubleClick,    // Word selection
    TripleClick,    // Line/paragraph selection
    KeyboardShortcut, // Ctrl+A, Shift+arrows, etc.
    ContextMenu,
}

/// Represents drag and drop operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragDropEvent {
    /// The start position of the drag
    pub start_position: Position,
    
    /// The end position of the drop
    pub end_position: Position,
    
    /// The UI element being dragged (source)
    pub source_element: Option<UIElement>,
    
    /// The type of data being dragged
    pub data_type: Option<String>,
    
    /// The dragged content (if text)
    pub content: Option<String>,
    
    /// Whether the drag was successful
    pub success: bool,
    
    /// Event metadata (target UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents hotkey/shortcut events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyEvent {
    /// The key combination (e.g., "Ctrl+C", "Alt+Tab")
    pub combination: String,
    
    /// The action performed by the hotkey
    pub action: Option<String>,
    
    /// Whether this was a global or application-specific hotkey
    pub is_global: bool,
    
    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents a workflow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// A mouse event
    Mouse(MouseEvent),
    
    /// A keyboard event
    Keyboard(KeyboardEvent),
    
    /// A clipboard event
    Clipboard(ClipboardEvent),
    
    /// A text selection event
    TextSelection(TextSelectionEvent),
    
    /// A drag and drop event
    DragDrop(DragDropEvent),
    
    /// A hotkey event
    Hotkey(HotkeyEvent),
    
    /// A UI Automation property change event
    UiPropertyChanged(UiPropertyChangedEvent),
    
    /// A UI Automation focus change event
    UiFocusChanged(UiFocusChangedEvent),
}

/// Represents a recorded event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// The timestamp of the event (milliseconds since epoch)
    pub timestamp: u64,
    
    /// The event
    pub event: WorkflowEvent,
}

/// Represents a recorded workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedWorkflow {
    /// The name of the workflow
    pub name: String,
    
    /// The timestamp when the recording started
    pub start_time: u64,
    
    /// The timestamp when the recording ended
    pub end_time: Option<u64>,
    
    /// The recorded events
    pub events: Vec<RecordedEvent>,
}

impl RecordedWorkflow {
    /// Create a new recorded workflow
    pub fn new(name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            name,
            start_time: now,
            end_time: None,
            events: Vec::new(),
        }
    }
    
    /// Add an event to the workflow
    pub fn add_event(&mut self, event: WorkflowEvent) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        self.events.push(RecordedEvent {
            timestamp: now,
            event,
        });
    }
    
    /// Finish the recording
    pub fn finish(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        self.end_time = Some(now);
    }

    /// Serialize the workflow to JSON string
    /// This converts UIElement instances to serializable form
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let serializable: SerializableRecordedWorkflow = self.into();
        serde_json::to_string_pretty(&serializable)
    }

    /// Serialize the workflow to JSON bytes
    /// This converts UIElement instances to serializable form
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let serializable: SerializableRecordedWorkflow = self.into();
        serde_json::to_vec_pretty(&serializable)
    }

    /// Deserialize a workflow from JSON string
    /// Note: This creates a workflow with serializable UI elements,
    /// not the original UIElement instances
    pub fn from_json(json: &str) -> Result<SerializableRecordedWorkflow, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize a workflow from JSON bytes
    /// Note: This creates a workflow with serializable UI elements,
    /// not the original UIElement instances
    pub fn from_json_bytes(bytes: &[u8]) -> Result<SerializableRecordedWorkflow, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Save the workflow to a JSON file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a workflow from a JSON file
    /// Note: This creates a workflow with serializable UI elements,
    /// not the original UIElement instances
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<SerializableRecordedWorkflow, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let workflow = Self::from_json(&json)?;
        Ok(workflow)
    }
}

/// Represents UI Automation structure change types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StructureChangeType {
    ChildAdded,
    ChildRemoved,
    ChildrenInvalidated,
    ChildrenBulkAdded,
    ChildrenBulkRemoved,
    ChildrenReordered,
}

/// Represents a UI Automation structure change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStructureChangedEvent {
    /// The type of structure change
    pub change_type: StructureChangeType,
    
    /// The element where the structure change occurred
    pub element: Option<UIElement>,
    
    /// Runtime IDs of affected children (if applicable)
    pub runtime_ids: Option<Vec<i32>>,
    
    /// The application where the change occurred
    pub application: Option<String>,
    
    /// Additional details about the change
    pub details: Option<String>,
}

/// Represents a UI Automation property change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPropertyChangedEvent {
    /// The property that changed (as string for serialization)
    pub property_name: String,
    
    /// The property ID
    pub property_id: u32,
    
    /// The old value (if available)
    pub old_value: Option<String>,
    
    /// The new value
    pub new_value: Option<String>,
    
    /// Event metadata (UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Represents a UI Automation focus change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFocusChangedEvent {
    /// The previous element that had focus (if available)
    pub previous_element: Option<UIElement>,
    
    /// Event metadata (current focused UI element, application, etc.)
    pub metadata: EventMetadata,
}

/// Unified metadata for all workflow events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// The UI element associated with this event (if available)
    pub ui_element: Option<UIElement>,
    
    /// The application where this event occurred
    pub application: Option<String>,
    
    /// The window title where this event occurred
    pub window_title: Option<String>,
    
    /// The process ID of the application
    pub process_id: Option<u32>,
}

impl EventMetadata {
    /// Create new metadata from a UI element
    pub fn from_ui_element(ui_element: Option<UIElement>) -> Self {
        let (application, window_title, process_id) = if let Some(ref elem) = ui_element {
            (
                Some(elem.application_name()).filter(|s| !s.is_empty()),
                Some(elem.window_title()).filter(|s| !s.is_empty()),
                None,
            )
        } else {
            (None, None, None)
        };

        Self {
            ui_element,
            application,
            window_title,
            process_id,
        }
    }

    /// Create empty metadata
    pub fn empty() -> Self {
        Self {
            ui_element: None,
            application: None,
            window_title: None,
            process_id: None,
        }
    }
}

/// Serializable version of UIElement for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableUIElement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<(f64, f64, f64, f64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_title: Option<String>,
}

impl From<&UIElement> for SerializableUIElement {
    fn from(element: &UIElement) -> Self {
        let attrs = element.attributes();
        let bounds = element.bounds().ok();
        
        // Helper function to filter empty strings
        fn filter_empty(s: Option<String>) -> Option<String> {
            s.filter(|s| !s.is_empty())
        }
        
        Self {
            id: filter_empty(element.id()),
            role: element.role(),
            name: filter_empty(attrs.name),
            bounds,
            value: filter_empty(attrs.value),
            description: filter_empty(attrs.description),
            application: filter_empty(Some(element.application_name())),
            window_title: filter_empty(Some(element.window_title())),
        }
    }
}

/// Serializable version of EventMetadata for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEventMetadata {
    /// The UI element associated with this event (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_element: Option<SerializableUIElement>,
    
    /// The application where this event occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    
    /// The window title where this event occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_title: Option<String>,
    
    /// The process ID of the application
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
}

impl From<&EventMetadata> for SerializableEventMetadata {
    fn from(metadata: &EventMetadata) -> Self {
        Self {
            ui_element: metadata.ui_element.as_ref().map(|elem| elem.into()),
            application: metadata.application.clone(),
            window_title: metadata.window_title.clone(),
            process_id: metadata.process_id,
        }
    }
}

/// Serializable version of KeyboardEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableKeyboardEvent {
    pub key_code: u32,
    pub is_key_down: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
    pub shift_pressed: bool,
    pub win_pressed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_code: Option<u32>,
    pub metadata: SerializableEventMetadata,
}

impl From<&KeyboardEvent> for SerializableKeyboardEvent {
    fn from(event: &KeyboardEvent) -> Self {
        Self {
            key_code: event.key_code,
            is_key_down: event.is_key_down,
            ctrl_pressed: event.ctrl_pressed,
            alt_pressed: event.alt_pressed,
            shift_pressed: event.shift_pressed,
            win_pressed: event.win_pressed,
            character: event.character,
            scan_code: event.scan_code,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of MouseEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMouseEvent {
    pub event_type: MouseEventType,
    pub button: MouseButton,
    pub position: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_delta: Option<(i32, i32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drag_start: Option<Position>,
    pub metadata: SerializableEventMetadata,
}

impl From<&MouseEvent> for SerializableMouseEvent {
    fn from(event: &MouseEvent) -> Self {
        Self {
            event_type: event.event_type,
            button: event.button,
            position: event.position,
            scroll_delta: event.scroll_delta,
            drag_start: event.drag_start,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of ClipboardEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableClipboardEvent {
    pub action: ClipboardAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub truncated: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&ClipboardEvent> for SerializableClipboardEvent {
    fn from(event: &ClipboardEvent) -> Self {
        Self {
            action: event.action.clone(),
            content: event.content.clone(),
            content_size: event.content_size,
            format: event.format.clone(),
            truncated: event.truncated,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of TextSelectionEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTextSelectionEvent {
    pub selected_text: String,
    pub start_position: Position,
    pub end_position: Position,
    pub selection_method: SelectionMethod,
    pub selection_length: usize,
    pub is_partial_selection: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&TextSelectionEvent> for SerializableTextSelectionEvent {
    fn from(event: &TextSelectionEvent) -> Self {
        Self {
            selected_text: event.selected_text.clone(),
            start_position: event.start_position,
            end_position: event.end_position,
            selection_method: event.selection_method.clone(),
            selection_length: event.selection_length,
            is_partial_selection: event.is_partial_selection,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of DragDropEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDragDropEvent {
    pub start_position: Position,
    pub end_position: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_element: Option<SerializableUIElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub success: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&DragDropEvent> for SerializableDragDropEvent {
    fn from(event: &DragDropEvent) -> Self {
        Self {
            start_position: event.start_position,
            end_position: event.end_position,
            source_element: event.source_element.as_ref().map(|elem| elem.into()),
            data_type: event.data_type.clone(),
            content: event.content.clone(),
            success: event.success,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of HotkeyEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableHotkeyEvent {
    pub combination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    pub is_global: bool,
    pub metadata: SerializableEventMetadata,
}

impl From<&HotkeyEvent> for SerializableHotkeyEvent {
    fn from(event: &HotkeyEvent) -> Self {
        Self {
            combination: event.combination.clone(),
            action: event.action.clone(),
            is_global: event.is_global,
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of UiPropertyChangedEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableUiPropertyChangedEvent {
    pub property_name: String,
    pub property_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    pub metadata: SerializableEventMetadata,
}

impl From<&UiPropertyChangedEvent> for SerializableUiPropertyChangedEvent {
    fn from(event: &UiPropertyChangedEvent) -> Self {
        Self {
            property_name: event.property_name.clone(),
            property_id: event.property_id,
            old_value: event.old_value.clone(),
            new_value: event.new_value.clone(),
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of UiFocusChangedEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableUiFocusChangedEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_element: Option<SerializableUIElement>,
    pub metadata: SerializableEventMetadata,
}

impl From<&UiFocusChangedEvent> for SerializableUiFocusChangedEvent {
    fn from(event: &UiFocusChangedEvent) -> Self {
        Self {
            previous_element: event.previous_element.as_ref().map(|elem| elem.into()),
            metadata: (&event.metadata).into(),
        }
    }
}

/// Serializable version of WorkflowEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableWorkflowEvent {
    Mouse(SerializableMouseEvent),
    Keyboard(SerializableKeyboardEvent),
    Clipboard(SerializableClipboardEvent),
    TextSelection(SerializableTextSelectionEvent),
    DragDrop(SerializableDragDropEvent),
    Hotkey(SerializableHotkeyEvent),
    UiPropertyChanged(SerializableUiPropertyChangedEvent),
    UiFocusChanged(SerializableUiFocusChangedEvent),
}

impl From<&WorkflowEvent> for SerializableWorkflowEvent {
    fn from(event: &WorkflowEvent) -> Self {
        match event {
            WorkflowEvent::Mouse(e) => SerializableWorkflowEvent::Mouse(e.into()),
            WorkflowEvent::Keyboard(e) => SerializableWorkflowEvent::Keyboard(e.into()),
            WorkflowEvent::Clipboard(e) => SerializableWorkflowEvent::Clipboard(e.into()),
            WorkflowEvent::TextSelection(e) => SerializableWorkflowEvent::TextSelection(e.into()),
            WorkflowEvent::DragDrop(e) => SerializableWorkflowEvent::DragDrop(e.into()),
            WorkflowEvent::Hotkey(e) => SerializableWorkflowEvent::Hotkey(e.into()),
            WorkflowEvent::UiPropertyChanged(e) => SerializableWorkflowEvent::UiPropertyChanged(e.into()),
            WorkflowEvent::UiFocusChanged(e) => SerializableWorkflowEvent::UiFocusChanged(e.into()),
        }
    }
}

/// Serializable version of RecordedEvent for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRecordedEvent {
    pub timestamp: u64,
    pub event: SerializableWorkflowEvent,
}

impl From<&RecordedEvent> for SerializableRecordedEvent {
    fn from(event: &RecordedEvent) -> Self {
        Self {
            timestamp: event.timestamp,
            event: (&event.event).into(),
        }
    }
}

/// Serializable version of RecordedWorkflow for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRecordedWorkflow {
    pub name: String,
    pub start_time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    pub events: Vec<SerializableRecordedEvent>,
}

impl From<&RecordedWorkflow> for SerializableRecordedWorkflow {
    fn from(workflow: &RecordedWorkflow) -> Self {
        Self {
            name: workflow.name.clone(),
            start_time: workflow.start_time,
            end_time: workflow.end_time,
            events: workflow.events.iter().map(|e| e.into()).collect(),
        }
    }
}
