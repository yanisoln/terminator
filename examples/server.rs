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

impl AttributesResponse {
    fn from_element(element: &UIElement) -> Self {
        let attrs = element.attributes();
        Self {
            role: attrs.role,
            label: attrs.label,
            value: attrs.value,
            description: attrs.description,
            properties: attrs.properties,
            id: element.id(),
        }
    }
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

// Custom error type for API responses
enum ApiError {
    Automation(AutomationError),
    BadRequest(String),
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

// New Helper: Resolve selector chain step-by-step
async fn resolve_element_from_chain(
    state: &Arc<AppState>,
    selector_chain: &[String],
) -> Result<UIElement, ApiError> {
    if selector_chain.is_empty() {
        return Err(ApiError::BadRequest("selector_chain cannot be empty".to_string()));
    }

    let selectors: Vec<Selector> = selector_chain.iter().map(|s| s.as_str().into()).collect();

    // Find the first element from the desktop root
    info!(selector = ?selectors[0], "Resolving first element in chain");
    let mut current_element = state.desktop.locator(selectors[0].clone())
        .wait() 
        .await
        .map_err(|e| {
            info!("Failed finding first element in chain: {:?}, error: {}", selectors[0], e);
            ApiError::Automation(e)
        })?;

    // Sequentially find subsequent elements within the previous one
    for (index, selector) in selectors.iter().skip(1).enumerate() {
        info!(selector = ?selector, parent_role = current_element.role(), parent_id = ?current_element.id(), "Resolving next element in chain (step {})", index + 2);
        
        // 1. Get the locator for the next step relative to the current element.
        let next_locator = current_element.locator(selector.clone())
            .map_err(|e| {
                info!("Failed creating locator for step {} in chain: {:?}, error: {}", index + 2, selector, e);
                ApiError::Automation(e)
            })?;
            
        // 2. Wait for the element using the new locator.
        current_element = next_locator
            .wait() 
            .await
            .map_err(|e| {
                 info!("Failed waiting for element in step {} in chain: {:?}, error: {}", index + 2, selector, e);
                 ApiError::Automation(e)
            })?;
    }
    
    info!("Successfully resolved element chain");
    Ok(current_element)
}

// Handler for finding an element
async fn find_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<ElementResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to find element by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    info!(element_id = ?element.id(), role = element.role(), "Element found via chain");
    Ok(Json(ElementResponse::from_element(&element)))
}

// Handler for clicking an element
async fn click_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<ClickResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to click element by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    match element.click() {
        Ok(result) => {
            info!("Element clicked successfully");
            Ok(Json(result.into()))
        }
        Err(e) => {
            info!("Failed to click resolved element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for typing text into an element
async fn type_text_into_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TypeTextRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, text = %payload.text, "Attempting to type text by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    match element.type_text(&payload.text) {
        Ok(_) => {
            info!("Text typed successfully");
            Ok(Json(BasicResponse { message: "Text typed successfully".to_string() }))
        }
        Err(e) => {
            info!("Failed to type text into resolved element: {}", e);
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
    info!(chain = ?payload.selector_chain, max_depth, "Attempting to get text by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    match element.text(max_depth) {
        Ok(text) => {
            info!("Text retrieved successfully");
            Ok(Json(TextResponse { text }))
        }
        Err(e) => {
            info!("Failed to get text from resolved element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for finding multiple elements (Assuming find all matching last step within first match of prior steps)
async fn find_elements(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<ElementsResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to find elements by chaining");
    
    if payload.selector_chain.len() < 1 {
         return Err(ApiError::BadRequest("selector_chain must have at least one selector".to_string()));
    }
    
    let parent_element = if payload.selector_chain.len() == 1 {
        // If only one selector, the "parent" is the desktop root conceptually
        // We find elements relative to the desktop root using the single selector
        None 
    } else {
        // Resolve the chain up to the second-to-last element
        Some(resolve_element_from_chain(&state, &payload.selector_chain[..payload.selector_chain.len()-1]).await?)
    };

    // Get the last selector for the final step
    let last_selector_str = payload.selector_chain.last().unwrap(); // Safe due to len check
    let last_selector: Selector = last_selector_str.as_str().into();

    let locator = match parent_element {
        Some(ref parent) => parent.locator(last_selector).map_err(ApiError::Automation)?,
        None => state.desktop.locator(last_selector), // Find from desktop root
    };

    // Use locator.all() to find matching elements within the final context
    match locator.all() {
        Ok(elements) => {
            info!("Found {} elements matching last step in chain", elements.len());
            let response_elements = elements
                .iter()
                .map(ElementResponse::from_element)
                .collect();
            Ok(Json(ElementsResponse { elements: response_elements }))
        }
        Err(e) => {
            info!("Failed to find elements in last step of chain: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for getting element attributes
async fn get_element_attributes(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<AttributesResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to get attributes by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    info!("Attributes retrieved successfully");
    Ok(Json(AttributesResponse::from_element(&element)))
}

// Handler for getting element bounds
async fn get_element_bounds(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<BoundsResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to get bounds by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    match element.bounds() {
        Ok((x, y, width, height)) => {
            info!("Bounds retrieved successfully");
            Ok(Json(BoundsResponse { x, y, width, height }))
        }
        Err(e) => {
            info!("Failed to get bounds for resolved element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for checking if an element is visible
async fn is_element_visible(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainedRequest>,
) -> Result<Json<BooleanResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, "Attempting to check visibility by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    match element.is_visible() {
        Ok(result) => {
            info!("Visibility check successful: {}", result);
            Ok(Json(BooleanResponse { result }))
        }
        Err(e) => {
            info!("Failed to check visibility for resolved element: {}", e);
            Err(ApiError::Automation(e))
        }
    }
}

// Handler for pressing a key on an element
async fn press_key_on_element(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PressKeyRequest>,
) -> Result<Json<BasicResponse>, ApiError> {
    info!(chain = ?payload.selector_chain, key = %payload.key, "Attempting to press key by chaining");
    let element = resolve_element_from_chain(&state, &payload.selector_chain).await?;
    match element.press_key(&payload.key) {
        Ok(_) => {
            info!("Key pressed successfully");
            Ok(Json(BasicResponse { message: "Key pressed successfully".to_string() }))
        }
        Err(e) => {
            info!("Failed to press key on resolved element: {}", e);
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
        .route("/find_element", post(find_element))
        .route("/find_elements", post(find_elements))
        .route("/click", post(click_element))
        .route("/type_text", post(type_text_into_element))
        .route("/get_text", post(get_element_text))
        .route("/get_attributes", post(get_element_attributes))
        .route("/get_bounds", post(get_element_bounds))
        .route("/is_visible", post(is_element_visible))
        .route("/press_key", post(press_key_on_element))
        .route("/open_application", post(open_application))
        .route("/open_url", post(open_url))
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
} 