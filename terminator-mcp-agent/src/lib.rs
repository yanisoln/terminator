use tokio::runtime::Runtime;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use rmcp::Service;
use rmcp::model::ServerInfo;
use rmcp::{Error as McpError, ServiceExt, model::*, tool};
use rmcp::serve_server;
use rmcp::ServerHandler;
use terminator::{AutomationError, Desktop, Locator, Selector, UIElement};
use serde_json::json;
// use modelcontextprotocol::server::{ToolCallback, ToolResult};


#[derive(Serialize, Deserialize)]
pub struct FindWindowArgs {
    /// A substring of the window title to search for (case-insensitive).
    pub title_contains: String,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct LocatorArgs {
    /// An array of selector strings to locate the element.
    pub selector_chain: Vec<String>,
    /// Optional timeout in milliseconds for the action.
    pub timeout_ms: Option<u64>,
}


#[derive(Serialize, Deserialize)]
pub struct TypeIntoElementArgs {
    /// An array of selector strings to locate the element.
    pub selector_chain: Vec<String>,
    /// Optional timeout in milliseconds for the action.
    pub timeout_ms: Option<u64>,
    /// The text to type into the element.
    pub text_to_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetElementTextArgs {
    /// An array of selector strings to locate the element.
    pub selector_chain: Vec<String>,
    /// Optional timeout in milliseconds for the action.
    pub timeout_ms: Option<u64>,
    /// Maximum depth to search for text within child elements.
    pub max_depth: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct PressKeyArgs {
    /// An array of selector strings to locate the element.
    pub selector_chain: Vec<String>,
    /// Optional timeout in milliseconds for the action.
    pub timeout_ms: Option<u64>,
    /// The key or key combination to press (e.g., 'Enter', 'Ctrl+A').
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct RunCommandArgs {
    /// The command to run on Windows.
    pub windows_command: Option<String>,
    /// The command to run on Linux/macOS.
    pub unix_command: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ExploreArgs {
    /// Optional selector chain to explore from a specific element.
    pub selector_chain: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct CaptureScreenArgs {}


pub fn create_server() -> Result<McpServer> {
    let server_info = ServerInfo {
        name: "terminator-mcp-agent".to_string(),
        version: "0.1.0".to_string(),
        description: "An MCP server providing desktop automation via Terminator.".to_string(),
    };

    let server = McpServer::new(server_info)?;
    Ok(server)
}

pub async fn find_window(client: &DesktopUseClient, title_contains: &str, timeout_ms: Option<u64>) -> Result<ElementResponse> {
    let window_locator = client.find_window(title_contains, timeout_ms).await?;
    let window_element = window_locator.first().await?;
    Ok(window_element)
}

pub fn register_tools(server: &mut McpServer, desktop: Desktop) {
    server.tool("findWindow", move |args: crate::FindWindowArgs| {
        let desktop = desktop.clone(); // Clone the Desktop instance for async use
        async move {
            match find_window(&desktop, &args.title_contains, args.timeout_ms).await {
                Ok(window) => ToolResult::success(json!(window)),
                Err(err) => ToolResult::error(err.to_string()),
            }
        }
    });
}
