use tokio::runtime::Runtime;
use terminator_mcp_agent::{create_server, register_tools};
use terminator::Desktop;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Desktop instance
    #[cfg(target_os = "windows")]
    let desktop = Desktop::new(false, false);

    // Create the MCP server
    let mut server = create_server()?;

    // Register tools
    register_tools(&mut server, desktop);

    // Connect the server to stdio transport
    server.connect("stdio").await?;

    Ok(())
}


