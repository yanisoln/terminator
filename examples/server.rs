use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use terminator::{
    AutomationError,
    Desktop,
    Locator,
    Selector,
    UIElement,
};
use tracing::{info, Level};
use std::time::Duration;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use std::env; // Import env module

// Shared application state
struct AppState {
    desktop: Arc<Desktop>,
}

// Base request structure with selector chain
#[derive(Deserialize)]
struct ChainedRequest {
    selector_chain: Vec<String>,
}

// Request structure for typing text (with chain)
#[derive(Deserialize)]
struct TypeTextRequest {
    selector_chain: Vec<String>,
    text: String,
}

// Request structure for getting text (with chain)
#[derive(Deserialize)]
struct GetTextRequest {
    selector_chain: Vec<String>,
    max_depth: Option<usize>,
}

// Request structure for pressing a key (with chain)
#[derive(Deserialize)]
struct PressKeyRequest {
    selector_chain: Vec<String>,
    key: String,
}

// Request structure for opening an application
#[derive(Deserialize)]
struct OpenApplicationRequest {
    app_name: String,
}

// Request structure for opening a URL
#[derive(Deserialize)]
struct OpenUrlRequest {
    url: String,
    browser: Option<String>,
}

// Request structure for opening a file
#[derive(Deserialize)]
struct OpenFileRequest {
    file_path: String,
}

// Request structure for running a command
#[derive(Deserialize)]
struct RunCommandRequest {
    windows_command: Option<String>,
    unix_command: Option<String>,
}

// Request structure for capturing a specific monitor
#[derive(Deserialize)]
struct CaptureMonitorRequest {
    monitor_name: String,
}

// Request structure for OCR on an image path
#[derive(Deserialize)]
struct OcrImagePathRequest {
    image_path: String,
}

// Request structure for OCR on raw screenshot data (base64 encoded)
#[derive(Deserialize)]
struct OcrScreenshotRequest {
    image_base64: String,
    width: u32,
    height: u32,
}

// Request structure for expectations (can often reuse ChainedRequest)
// Add optional timeout
#[derive(Deserialize)]
struct ExpectRequest {
    selector_chain: Vec<String>,
    timeout_ms: Option<u64>,
}

// Specific request for expecting text
#[derive(Deserialize)]
struct ExpectTextRequest {
    selector_chain: Vec<String>,
    expected_text: String,
    max_depth: Option<usize>, // Needed for element.text() call within expect_text_equals
    timeout_ms: Option<u64>,
}

// Add this new request struct
#[derive(Deserialize)]
struct ActivateApplicationRequest {
    app_name: String,
}

// Add this request struct
#[derive(Deserialize)]
struct ActivateBrowserWindowRequest {
    title: String,
}

// Basic response structure
#[derive(Serialize)]
struct BasicResponse {
    message: String,
}

// Response structure for element details
#[derive(Serialize)]
struct ElementResponse {
    role: String,
    label: Option<String>,
    id: Option<String>,
}

impl ElementResponse {
    fn from_element(element: &UIElement) -> Self {
        let attrs = element.attributes();
        Self {
            role: attrs.role,
            label: attrs.label,
            id: element.id(),
        }
    }
}

// Response structure for click action
#[derive(Serialize)]
struct ClickResponse {
    method: String,
    coordinates: Option<(f64, f64)>,
    details: String,
}

impl From<terminator::ClickResult> for ClickResponse {
    fn from(result: terminator::ClickResult) -> Self {
        Self {
            method: result.method,
            coordinates: result.coordinates,
            details: result.details,
        }
    }
}

// Response structure for text content
#[derive(Serialize)]
struct TextResponse {
    text: String,
}

// Response structure for multiple elements
#[derive(Serialize)]
struct ElementsResponse {
    elements: Vec<ElementResponse>,
}

// Response structure for element attributes
#[derive(Serialize)]
struct AttributesResponse {
    role: String,
    label: Option<String>,
    value: Option<String>,
    description: Option<String>,
    properties: HashMap<String, Option<serde_json::Value>>,
    id: Option<String>,
}


// Response structure for element bounds
#[derive(Serialize)]
struct BoundsResponse {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

// Response structure for boolean results
#[derive(Serialize)]
struct BooleanResponse {
    result: bool,
}

// Response structure for command output
#[derive(Serialize)]
struct CommandOutputResponse {
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
}

impl From<terminator::CommandOutput> for CommandOutputResponse {
    fn from(output: terminator::CommandOutput) -> Self {
        Self {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_status,
        }
    }
}

// Response structure for screenshot result (with base64 encoded image)
#[derive(Serialize)]
struct ScreenshotResponse {
    image_base64: String,
    width: u32,
    height: u32,
}

impl TryFrom<terminator::ScreenshotResult> for ScreenshotResponse {
    type Error = ApiError; // Or a more specific error

    fn try_from(result: terminator::ScreenshotResult) -> Result<Self, Self::Error> {
        // Encode image data to base64
        let base64_image = BASE64_STANDARD.encode(&result.image_data);
        Ok(Self {
            image_base64: base64_image,
            width: result.width,
            height: result.height,
        })
    }
}

// Response structure for OCR result
#[derive(Serialize)]
struct OcrResponse {
    text: String,
}

// Custom error type for API responses
#[derive(Debug)]
enum ApiError {
    Automation(AutomationError),
    BadRequest(String),
}

// Implement the From trait to allow automatic conversion
impl From<AutomationError> for ApiError {
    fn from(err: AutomationError) -> Self {
        ApiError::Automation(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Automation(err) => {
                tracing::error!("Automation error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Automation error: {}", err))
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, Json(BasicResponse { message: error_message })).into_response()
    }
}

async fn root() -> &'static str {
    "Terminator API Server Ready"
}

// Helper to get timeout duration from optional ms
fn get_timeout(timeout_ms: Option<u64>) -> Option<Duration> {
    timeout_ms.map(Duration::from_millis)
}

// Helper function to create a locator from the full chain (DIFFERENT from resolve_element_from_chain)
// This locator will be used for the expect methods which handle their own waiting.
fn create_locator_for_chain(
    state: &Arc<AppState>,
    selector_chain: &[String],
) -> Result<Locator, ApiError> {
    if selector_chain.is_empty() {
        return Err(ApiError::BadRequest("selector_chain cannot be empty".to_string()));
    }

    let selectors: Vec<Selector> = selector_chain.iter().map(|s| s.as_str().into()).collect();

    // Create locator for the first element
    let mut locator = state.desktop.locator(selectors[0].clone());

    // Chain subsequent locators
    for selector in selectors.iter().skip(1) {
        // Note: Locator::locator creates a new locator, not finding the element yet
        locator = locator.locator(selector.clone());
    }

    Ok(locator)
}

// Handler for finding an element
async fn first(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to find element");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    // Use the locator's wait() method - timeout is handled by locator's default or could be added to request
    match locator.wait().await {
         Ok(element) => {
            info!(element_id = ?element.id(), role = element.role(), "Element found");
            Ok(Json(ElementResponse::from_element(&element)))
         }
         Err(e) => {
            info!("Failed finding element: {}", e);
            Err(ApiError::Automation(e))
         }
    }
}

// Handler for clicking an element
async fn click_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<ClickResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to click element");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    match locator.click().await { // locator.click() already includes wait()
        Ok(result) => {
            info!("Element clicked successfully");
            Ok(Json(result.into()))
        }
        Err(e) => {
            info!("Failed to click element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for typing text into an element
async fn type_text_into_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TypeTextRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, text = %payload.text, "Attempting to type text");
     let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    match locator.type_text(&payload.text).await { // locator.type_text() includes wait()
        Ok(_) => {
            info!("Text typed successfully");
            Ok(Json(BasicResponse { message: "Text typed successfully".to_string() }))
        }
        Err(e) => {
            info!("Failed to type text into element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for getting text from an element
async fn get_element_text(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GetTextRequest>,
) -> Result<Json<TextResponse>, ApiError> {
    let max_depth = payload.max_depth.unwrap_or(5);
    info!(chain = ?payload.selector_chain, max_depth, "Attempting to get text");
     let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    match locator.text(max_depth).await { // locator.text() includes wait()
        Ok(text) => {
            info!("Text retrieved successfully");
            Ok(Json(TextResponse { text }))
        }
        Err(e) => {
            info!("Failed to get text from element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for finding multiple elements (Assuming find all matching last step within first match of prior steps)
async fn all(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<ElementsResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to find elements");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    // Use locator.all() - Note: This typically doesn't wait like other actions.
    // If waiting is desired before finding all, the client would need separate 'wait' then 'find_elements' calls,
    // or we could add a specific 'wait_and_find_all' endpoint. Keeping it simple for now.
    match locator.all() {
        Ok(elements) => {
            info!("Found {} elements matching chain", elements.len());
            let response_elements = elements
                .iter()
                .map(ElementResponse::from_element)
                .collect();
            Ok(Json(ElementsResponse { elements: response_elements }))
        }
        Err(e) => {
            info!("Failed to find elements: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for getting element attributes
async fn get_element_attributes(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<AttributesResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to get attributes");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    // Wait for the element first, handling potential errors
    let element = locator.wait().await?;

    // Get attributes (this doesn't return a Result)
    let attrs = element.attributes();
    let element_id = element.id(); // Get ID from the element we already have

    info!("Attributes retrieved successfully");

    // Construct and return the response
    Ok(Json(AttributesResponse {
        role: attrs.role,
        label: attrs.label,
        value: attrs.value,
        description: attrs.description,
        properties: attrs.properties,
        id: element_id, // Use the ID obtained earlier
    }))
}

// Handler for getting element bounds
async fn get_element_bounds(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<BoundsResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to get bounds");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    match locator.wait().await?.bounds() { // Wait first, then get bounds
        Ok((x, y, width, height)) => {
            info!("Bounds retrieved successfully");
            Ok(Json(BoundsResponse { x, y, width, height }))
        }
        Err(e) => {
            info!("Failed to get bounds: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for checking if an element is visible
async fn is_element_visible(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<BooleanResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to check visibility");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    // Use expect_visible with a very short timeout (or zero) for a non-waiting check
    // Or, call the underlying element method after waiting. Let's wait then check.
    match locator.wait().await?.is_visible() {
        Ok(result) => {
            info!("Visibility check successful: {}", result);
            Ok(Json(BooleanResponse { result }))
        }
        Err(e) => {
             // Distinguish between element not found during wait vs. error calling is_visible
            if matches!(e, AutomationError::Timeout(_)) {
                 info!("Element not found while checking visibility: {}", e);
                 // Return false if the element wasn't found within default timeout
                 Ok(Json(BooleanResponse { result: false }))
            } else {
                 info!("Failed to check visibility: {}", e);
                 Err(ApiError::Automation(e))
            }
        }
    }
}

// Handler for pressing a key on an element
async fn press_key_on_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PressKeyRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, key = %payload.key, "Attempting to press key");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    match locator.press_key(&payload.key).await { // locator.press_key() includes wait()
        Ok(_) => {
            info!("Key pressed successfully");
            Ok(Json(BasicResponse { message: "Key pressed successfully".to_string() }))
        }
        Err(e) => {
            info!("Failed to press key: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for opening an application (returns basic info, no handle)
async fn open_application(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OpenApplicationRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(app_name = %payload.app_name, "Attempting to open application");
    match state.desktop.open_application(&payload.app_name) {
        Ok(_) => {
            info!("Application opened successfully");
            Ok(Json(BasicResponse { message: format!("Application '{}' opened", payload.app_name) }))
        }
        Err(e) => {
            info!("Failed to open application: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for opening a URL (returns basic info, no handle)
async fn open_url(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OpenUrlRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(url = %payload.url, browser = ?payload.browser, "Attempting to open URL");
    match state.desktop.open_url(&payload.url, payload.browser.as_deref()) {
        Ok(_) => {
            info!("URL opened successfully");
            Ok(Json(BasicResponse { message: format!("URL '{}' opened", payload.url) }))
        }
        Err(e) => {
            info!("Failed to open URL: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// --- NEW EXPECT HANDLERS ---

async fn expect_element_visible(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExpectRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, timeout = ?payload.timeout_ms, "Attempting to expect element visible");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;
    let timeout = get_timeout(payload.timeout_ms); // Uses Locator's default if None

    match locator.expect_visible(timeout).await {
        Ok(element) => {
            info!("Element found and is visible");
            Ok(Json(ElementResponse::from_element(&element)))
        }
        Err(e) => {
            info!("Expect visible failed: {}", e);
            Err(ApiError::Automation(e)) // Includes Timeout error
        }
    }
}

async fn expect_element_enabled(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExpectRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, timeout = ?payload.timeout_ms, "Attempting to expect element enabled");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;
     let timeout = get_timeout(payload.timeout_ms);

    match locator.expect_enabled(timeout).await {
        Ok(element) => {
            info!("Element found and is enabled");
            Ok(Json(ElementResponse::from_element(&element)))
        }
        Err(e) => {
            info!("Expect enabled failed: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

async fn expect_element_text_equals(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExpectTextRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, text = %payload.expected_text, depth = ?payload.max_depth, timeout = ?payload.timeout_ms, "Attempting to expect text equals");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;
    let max_depth = payload.max_depth.unwrap_or(5); // Use default or provided depth
    let timeout = get_timeout(payload.timeout_ms);

    match locator.expect_text_equals(&payload.expected_text, max_depth, timeout).await {
        Ok(element) => {
            info!("Element found and text matches");
            Ok(Json(ElementResponse::from_element(&element)))
        }
        Err(e) => {
            info!("Expect text equals failed: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// --- NEW HANDLER for activating app window ---

async fn activate_app_window(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to activate app window");
    let locator = create_locator_for_chain(&state, &payload.selector_chain)?;

    // First, find the element using wait() to ensure it exists.
    let element = locator.wait().await?;

    // Then, call the activate_window method.
    // This method needs to be added to the UIElement and UIElementImpl trait.
    match element.activate_window() {
        Ok(_) => {
            info!("App window activated successfully");
            Ok(Json(BasicResponse { message: "App window activated successfully".to_string() }))
        }
        Err(e) => {
            info!("Failed to activate app window: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Add this new handler
// Handler for activating an application by name
async fn activate_application_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivateApplicationRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(app_name = %payload.app_name, "Attempting to activate application");
    match state.desktop.activate_application(&payload.app_name) {
        Ok(_) => {
            info!("Application activated successfully");
            Ok(Json(BasicResponse { message: format!("Application '{}' activated", payload.app_name) }))
        }
        Err(e) => {
            info!("Failed to activate application: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// --- NEW TOP-LEVEL HANDLERS ---

// Handler for opening a file
async fn open_file(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OpenFileRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(file_path = %payload.file_path, "Attempting to open file");
    match state.desktop.open_file(&payload.file_path) {
        Ok(_) => {
            info!("File opened successfully");
            Ok(Json(BasicResponse { message: format!("File '{}' opened successfully", payload.file_path) }))
        }
        Err(e) => {
            info!("Failed to open file: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for running a command
async fn run_command(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RunCommandRequest>,
) -> Result<Json<CommandOutputResponse>, ApiError> {
    info!(windows = ?payload.windows_command, unix = ?payload.unix_command, "Attempting to run command");
    // Ensure at least one command is provided based on OS or handle error?
    // Current implementation relies on the core library to handle None correctly.
    match state.desktop.run_command(payload.windows_command.as_deref(), payload.unix_command.as_deref()) {
        Ok(output) => {
            info!("Command executed successfully");
            Ok(Json(output.into()))
        }
        Err(e) => {
            info!("Failed to run command: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for capturing the primary screen
async fn capture_screen(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ScreenshotResponse>, ApiError> {
    info!("Attempting to capture primary screen");
    match state.desktop.capture_screen() {
        Ok(screenshot_result) => {
            info!("Screen captured successfully");
            ScreenshotResponse::try_from(screenshot_result)
                .map(Json) // Wrap the successful conversion in Json
                .map_err(|e| {
                    // Handle potential conversion error (though unlikely here)
                    info!("Failed to encode screenshot: {:?}", e);
                    ApiError::BadRequest("Failed to encode screenshot data".to_string())
                 })
        }
        Err(e) => {
            info!("Failed to capture screen: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for capturing a specific monitor
async fn capture_monitor(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CaptureMonitorRequest>,
) -> Result<Json<ScreenshotResponse>, ApiError> {
    info!(monitor_name = %payload.monitor_name, "Attempting to capture monitor");
    match state.desktop.capture_monitor_by_name(&payload.monitor_name) {
         Ok(screenshot_result) => {
            info!("Monitor captured successfully");
             ScreenshotResponse::try_from(screenshot_result)
                .map(Json)
                .map_err(|e| {
                    info!("Failed to encode screenshot: {:?}", e);
                    ApiError::BadRequest("Failed to encode screenshot data".to_string())
                 })
        }
        Err(e) => {
            info!("Failed to capture monitor: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for performing OCR on an image path
async fn ocr_image_path(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OcrImagePathRequest>,
) -> Result<Json<OcrResponse>, ApiError> {
    info!(image_path = %payload.image_path, "Attempting OCR on image path");
    match state.desktop.ocr_image_path(&payload.image_path).await {
        Ok(text) => {
            info!("OCR performed successfully");
            Ok(Json(OcrResponse { text }))
        }
        Err(e) => {
            info!("Failed to perform OCR: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for performing OCR on raw screenshot data
async fn ocr_screenshot_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OcrScreenshotRequest>,
) -> Result<Json<OcrResponse>, ApiError> {
    info!("Attempting OCR on raw screenshot data");

    // Decode the base64 image data
    let image_data = BASE64_STANDARD.decode(&payload.image_base64)
        .map_err(|e| {
            info!("Failed to decode base64 image: {}", e);
            ApiError::BadRequest("Invalid base64 image data".to_string())
        })?;

    // Reconstruct a ScreenshotResult (or similar structure needed by the core function)
    let screenshot_result = terminator::ScreenshotResult {
        image_data,
        width: payload.width,
        height: payload.height,
    };

    // Call the core OCR function for screenshots
    match state.desktop.ocr_screenshot(&screenshot_result).await {
        Ok(text) => {
            info!("OCR on screenshot data performed successfully");
            Ok(Json(OcrResponse { text }))
        }
        Err(e) => {
            info!("Failed to perform OCR on screenshot data: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Add this handler
// Handler for activating a browser window by title
async fn activate_browser_window_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ActivateBrowserWindowRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(title = %payload.title, "Attempting to activate browser window by title");
    match state.desktop.activate_browser_window_by_title(&payload.title) {
        Ok(_) => {
            info!("Browser window activated successfully");
            Ok(Json(BasicResponse { message: format!("Browser window containing title '{}' activated", payload.title) }))
        }
        Err(e) => {
            info!("Failed to activate browser window: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // use debug mode
    tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    // Initialize the Desktop instance
    let desktop = Desktop::new(false, true).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    let shared_state = Arc::new(AppState {
        desktop: Arc::new(desktop),
    });

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/first", post(first))
        .route("/all", post(all))
        .route("/click", post(click_element))
        .route("/type_text", post(type_text_into_element))
        .route("/get_text", post(get_element_text))
        .route("/get_attributes", post(get_element_attributes))
        .route("/get_bounds", post(get_element_bounds))
        .route("/is_visible", post(is_element_visible))
        .route("/press_key", post(press_key_on_element))
        .route("/open_application", post(open_application))
        .route("/open_url", post(open_url))
        .route("/activate_app", post(activate_app_window))
        .route("/activate_application", post(activate_application_handler))
        .route("/expect_visible", post(expect_element_visible))
        .route("/expect_enabled", post(expect_element_enabled))
        .route("/expect_text_equals", post(expect_element_text_equals))
        .route("/open_file", post(open_file))
        .route("/run_command", post(run_command))
        .route("/capture_screen", post(capture_screen))
        .route("/capture_monitor", post(capture_monitor))
        .route("/ocr_image_path", post(ocr_image_path))
        .route("/ocr_screenshot", post(ocr_screenshot_handler))
        .route("/activate_browser_window", post(activate_browser_window_handler))
        .with_state(shared_state);

    // Determine port
    let default_port = 9375;
    let port: u16 = env::var("PORT")
        .ok() // Convert Result to Option
        .and_then(|s| s.parse().ok()) // Try parsing the string to u16
        .unwrap_or(default_port); // Use default if env var not set or invalid

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
} 