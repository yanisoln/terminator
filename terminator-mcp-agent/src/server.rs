use std::env;
use chrono::Local;
use serde_json::json;
use terminator::{Desktop, Selector, Locator};
use rmcp::{Error as McpError, ServerHandler, tool};
use crate::utils::{
    get_timeout,
    PressKeyArgs,
    ExploreArgs,
    LocatorArgs,
    FindWindowArgs,
    RunCommandArgs,
    DesktopWrapper,
    GetElementTextArgs,
    TypeIntoElementArgs,
    CaptureScreenArgs,
};
use rmcp::model::{
    CallToolResult,
    Content,
    ProtocolVersion, 
    ServerCapabilities,
    ServerInfo,
    Implementation,
};


#[tool(tool_box)]
impl DesktopWrapper {
    pub async fn new() -> Result<Self, McpError> {
        #[cfg(target_os = "windows")]
        let desktop = match Desktop::new(false, false) {
            Ok(d) => d,
            Err(e) => return Err(
                McpError::internal_error(
                    "Failed to initialize terminator desktop",
                    serde_json::to_value(e.to_string()).ok())
                ),
        };
        
        #[cfg(target_os = "macos")]
        let desktop = match Desktop::new(true, true) {
            Ok(d) => d,
            Err(e) => return Err(
                McpError::internal_error(
                    "Failed to initialize terminator desktop",
                    serde_json::to_value(e.to_string()).ok())
                ),
        };

        Ok(Self { desktop })
    }

    #[tool(description = "Finds a top-level window by title.")]
    async fn find_window_by_title(
        &self,
        #[tool(param)] args: FindWindowArgs,
    ) -> Result<CallToolResult, McpError> {

        let window = self.desktop.find_window_by_criteria(
            Some(&args.title_contains),
            get_timeout(args.timeout_ms)
        ).await.map_err(|e| McpError::resource_not_found(
                "Failed to find window",
                Some(json!({"reason": e.to_string()})),
            ));

        Ok(CallToolResult::success(vec![Content::json(&window)?]))
    }

    #[tool(description = "Reads text content from a UI element.")]
    async fn get_element_text(
        &self,
        #[tool(param)] args: GetElementTextArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.create_locator_for_chain(&args.selector_chain)?;
        let element = locator.wait(get_timeout(args.timeout_ms)).await
            .map_err(|e| McpError::internal_error(
                "Failed to locate element", Some(json!({"reason": e.to_string()})))
            )?;
        let text = element.text(args.max_depth.unwrap_or(7) as usize)      // 7 is defualt depth of uiautomation
            .map_err(|e| McpError::resource_not_found(
                "Failed to get text",
                Some(json!({"reason": e.to_string()})),
            ));

        Ok(CallToolResult::success(vec![Content::json(&text)?]))
    }

    #[tool(description = "Types text into a UI element.")]
    async fn type_into_element(
        &self,
        #[tool(param)] args: TypeIntoElementArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.create_locator_for_chain(&args.selector_chain)?;
        let element = locator.wait(get_timeout(args.timeout_ms)).await
            .map_err(|e| McpError::internal_error(
                "Failed to locate element", Some(json!({"reason": e.to_string()})))
            )?;
        element.type_text(&args.text_to_type, false)
            .map_err(|e| McpError::resource_not_found(
                "Failed to type text",
                Some(json!({"reason": e.to_string()})),
            ))?;

        Ok(CallToolResult::success(vec![Content::json("Text typed successfully")?]))
    }

    #[tool(description = "Clicks a UI element.")]
    async fn click_element(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {

        let locator = self.create_locator_for_chain(&args.selector_chain)?;
        let element = locator.wait(get_timeout(args.timeout_ms)).await
            .map_err(|e| McpError::internal_error(
                "Failed to locate element", Some(json!({"reason": e.to_string()})))
            )?;
        element.click().map_err(|e| McpError::resource_not_found(
                "Failed to click on element",
                Some(json!({"reason": e.to_string()})),
            ))?;

        Ok(CallToolResult::success(vec![Content::json("Element clicked successfully")?]))
    }

    #[tool(description = "Sends a key press to a UI element.")]
    async fn press_key(
        &self,
        #[tool(param)] args: PressKeyArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.create_locator_for_chain(&args.selector_chain)?;
        let element = locator.wait(get_timeout(args.timeout_ms)).await
            .map_err(|e| McpError::internal_error(
                "Failed to locate element", Some(json!({"reason": e.to_string()})))
            )?;
        element.press_key(&args.key).map_err(|e| McpError::resource_not_found(
                "Failed to press key",
                Some(json!({"reason": e.to_string()})),
            ))?;
            
        Ok(CallToolResult::success(vec![Content::json("Key pressed successfully")?]))
    }

    #[tool(description = "Executes a shell command.")]
    async fn run_command(
        &self,
        #[tool(param)] args: RunCommandArgs, 
    ) -> Result<CallToolResult, McpError> {
        let output = self.desktop.run_command(args.unix_command.as_deref(), args.unix_command.as_deref()).await
            .map_err(|e| McpError::internal_error("Failed to run command", Some(json!({"reason": e.to_string()})))
            )?;

        Ok(CallToolResult::success(vec![Content::json(json!({
            "exit_status": output.exit_status,
            "stdout": output.stdout,
            "stderr": output.stderr,
        }))?]))
    }

    #[tool(description = "Explores UI elements and their children.")]
    async fn explore(
        &self,
        #[tool(param)] args: ExploreArgs,
    ) -> Result<CallToolResult, McpError> {
        unimplemented!()
        // let locator = self.create_locator_for_chain(args.selector_chain.as_deref().unwrap())?;
        // let elements = locator.explore(get_timeout(args.timeout_ms)).await
        //     .map_err(|e| McpError::internal_error(
        //         "Failed to explore element", Some(json!({"reason": e.to_string()})))
        //     )?;
        //
        // let serializable_output = crate::utils::ExploreResponse {
        //     parent: elements.parent,
        //     children: elements.children,
        // };
        // Ok(CallToolResult::success(vec![Content::json(&json!({
        //     "parent": elements.parent.clone(),
        //     "children": elements.children.clone(),
        // }))?]))
    }

    #[tool(description = "Activates the window containing the specified element, bringing it to the foreground.")]
    async fn activate_element(
        &self,
        #[tool(param)] args: LocatorArgs,
    ) -> Result<CallToolResult, McpError> {
        let locator = self.create_locator_for_chain(&args.selector_chain)?;
        let element = locator.wait(get_timeout(args.timeout_ms)).await
            .map_err(|e| McpError::internal_error(
                "Failed to locate element", Some(json!({"reason": e.to_string()})))
            )?;
        element.activate_window().map_err(|e| McpError::resource_not_found(
                "Failed to activate window with that element",
                Some(json!({"reason": e.to_string()})),
            ))?;
        Ok(CallToolResult::success(vec![Content::json("Window with that element activated successfully")?]))
    }

    #[tool(description = "Captures a screenshot of the primary monitor and returns the recognized text content (OCR).")]
    async fn capture_screen(
        &self,
        #[tool(param)] _args: CaptureScreenArgs,
    ) -> Result<CallToolResult, McpError> {

        let screenshot = self.desktop.capture_screen()
            .await.map_err(|e| McpError::internal_error(
                "Failed to capture screen",
                Some(json!({"reason": e.to_string()})),
            ))?;

        let ocr_text = self.desktop.ocr_screenshot(&screenshot)
            .await.map_err(|e| McpError::internal_error(
                "Failed to perform OCR",
                Some(json!({"reason": e.to_string()})),
            ))?;

        Ok(CallToolResult::success(vec![Content::json(&ocr_text)?]))
    }

    // keep in wrapperr to avoid creating new instance
    fn create_locator_for_chain(
        &self,
        selector_chain: &[String],
    ) -> Result<Locator, McpError> {

        if selector_chain.is_empty() {
            return Err(
                McpError::invalid_params(
                    "selector_chain cannot be empty",
                    None
                ))
        }

        let selectors: Vec<Selector> = selector_chain.iter().map(|s| s.as_str().into()).collect();
        let mut locator = self.desktop.locator(selectors[0].clone());

        // Chain subsequent locators
        for selector in selectors.iter().skip(1) {
            locator = locator.locator(selector.clone());
        }

        Ok(locator)
    }
}

#[tool(tool_box)]
impl ServerHandler for DesktopWrapper {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(get_server_instructions().to_string()),
        }
    }
}

fn get_server_instructions() -> String {
    let current_date_time = Local::now().to_string();
    let current_os = env::consts::OS;
    let current_working_dir = env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "Unknown".to_string()); // Get current working directory

    format!(
        r#"
You are an AI assistant capable of controlling a computer desktop using the available tools.
You can interact with UI elements, run commands, and read text.

**Workflow:**
1.  **Identify Target Window:** Use 'find_window' to locate the main window of the application you need to interact with (e.g., by title). Note its selector (e.g., the `suggested_selector` field in the result, which often looks like '#12345...').
2.  **(Optional but Recommended) Explore:** Use 'explore' with the window's selector chain (e.g., ['window:"My App"'] or ['#windowId']) to understand its structure and find specific child elements (buttons, inputs, etc.). Pay close attention to the `suggested_selector` provided for each child element.
3.  **Interact:** Use tools like 'click_element', 'get_element_text', 'type_into_element', or 'press_key' with the appropriate **selector chain**.
    *   A selector chain is an array starting with the window selector, followed by selectors for child elements, e.g., ['window:"My App"', '#saveButtonId'].
    *   **Crucially, prefer using the exact `suggested_selector` (like '#12345...') returned by 'explore' or 'find_window'.** This ID is calculated based on multiple properties and is usually the most reliable way to target an element.
    *   If a suggested selector fails, you can try simpler selectors like `text:"Save"` or `role:"button"`, but these might match multiple elements or be less reliable.
    *   **Selector Failures:** If interaction fails (e.g., timeout), ensure the element is visible, try increasing the `timeout_ms` parameter (e.g., 10000 for 10 seconds), re-explore the parent element, or verify you have the correct window selector.
4.  **Execute Commands:** Use 'run_command' for non-UI tasks.
    *   **This is the preferred method for running shell commands (like `ls`, `dir`, `git status`, etc.)** instead of trying to type into a terminal UI element, which can be unreliable. Specify the command for Windows (`windows_command`) or Unix (`unix_command`).

**Available Tools:**

- **find_window**: Finds a top-level window by its title. Returns the window element's details, including a `suggested_selector`.
- **explore**: Lists child elements within a given element (or the screen). Use its output to find the `suggested_selector` for child elements needed in other tools.
- **click_element**: Clicks a UI element specified by its selector chain.
- **get_element_text**: Reads text content from a UI element specified by its selector chain.
- **type_into_element**: Types text into a UI element (like an input field) specified by its selector chain. Requires a reliable selector for the input element.
- **press_key**: Sends a key press (like 'Enter', 'Tab', 'Ctrl+C') to a UI element specified by its selector chain.
- **run_command**: Executes a shell command directly on the system (specify `windows_command` or `unix_command`). Ideal for terminal tasks.

Contextual information:
- The current date and time is {}.
- Current operating system: {}.
- Current working directory: {}.

**Important:** Always provide the full selector chain when interacting with elements inside a window. Start the chain with the window selector. **Prioritize using the `suggested_selector` from `explore` results.** Use `run_command` for shell operations.
"#,
        current_date_time, current_os, current_working_dir
    )
}
