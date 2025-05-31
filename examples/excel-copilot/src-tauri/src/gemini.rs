use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiMessage {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "inlineData")]
    pub inline_data: Option<InlineData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String, // base64 encoded
}

impl GeminiPart {
    pub fn text(text: String) -> Self {
        Self {
            text: Some(text),
            inline_data: None,
        }
    }
    
    pub fn pdf(base64_data: String) -> Self {
        Self {
            text: None,
            inline_data: Some(InlineData {
                mime_type: "application/pdf".to_string(),
                data: base64_data,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiContent {
    pub parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum GeminiResponsePart {
    Text { text: String },
    FunctionCall { 
        #[serde(rename = "functionCall")]
        function_call: FunctionCall 
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    #[serde(rename = "functionCallingConfig")]
    pub function_calling_config: FunctionCallingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCallingConfig {
    pub mode: String, // "ANY" or "AUTO"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInstruction {
    pub parts: Vec<GeminiPart>,
}

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    base_url: String,
    conversation_history: Vec<GeminiMessage>,
    tools: Vec<FunctionDeclaration>,
    system_instruction: SystemInstruction,
}

/// Information about a tool call made by Gemini
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub function_name: String,
    pub arguments: Value,
    pub result: String,
}

/// Detailed response information from Gemini
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiResponseDetails {
    pub content: String,
    pub tool_calls: Vec<ToolCallInfo>,
    pub iterations: i32,
    pub has_tool_calls: bool,
}

impl GeminiClient {
    /// Create a new Gemini client with the provided API key
    pub fn new(api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let base_url = "https://generativelanguage.googleapis.com/v1beta/models".to_string();
        
        let mut gemini_client = Self {
            client,
            api_key,
            base_url,
            conversation_history: Vec::new(),
            tools: Vec::new(),
            system_instruction: SystemInstruction { parts: Vec::new() },
        };

        // Setup Excel tools
        gemini_client.setup_excel_tools();
        
        // Setup system instruction
        gemini_client.setup_system_instruction();
        
        Ok(gemini_client)
    }

    /// Configure Excel automation tools for Gemini
    fn setup_excel_tools(&mut self) {
        self.tools = vec![
            FunctionDeclaration {
                name: "read_excel_cell".to_string(),
                description: "Read the value from a specific cell in Excel".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "cell_address": {
                            "type": "string",
                            "description": "The cell address in Excel notation (e.g., A1, B2, C10)"
                        }
                    },
                    "required": ["cell_address"]
                }),
            },
            FunctionDeclaration {
                name: "write_excel_cell".to_string(),
                description: "Write a value to a specific cell in Excel".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "cell_address": {
                            "type": "string",
                            "description": "The cell address in Excel notation (e.g., A1, B2, C10)"
                        },
                        "value": {
                            "type": "string",
                            "description": "The value to write to the cell"
                        }
                    },
                    "required": ["cell_address", "value"]
                }),
            },
            FunctionDeclaration {
                name: "read_excel_range".to_string(),
                description: "Read values from a range of cells in Excel".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "start_cell": {
                            "type": "string",
                            "description": "The starting cell address (e.g., A1)"
                        },
                        "end_cell": {
                            "type": "string",
                            "description": "The ending cell address (e.g., C5)"
                        }
                    },
                    "required": ["start_cell", "end_cell"]
                }),
            },
            FunctionDeclaration {
                name: "get_excel_sheet_overview".to_string(),
                description: "Get a complete overview of the current Excel sheet showing all non-empty cells and their values".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            FunctionDeclaration {
                name: "set_excel_formula".to_string(),
                description: "Set a formula in an Excel cell".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "cell_address": {
                            "type": "string",
                            "description": "The cell address where to set the formula"
                        },
                        "formula": {
                            "type": "string",
                            "description": "The formula to set (e.g., =SUM(A1:A10), =AVERAGE(B1:B5))"
                        }
                    },
                    "required": ["cell_address", "formula"]
                }),
            },
        ];
    }

    /// Setup the system instruction for the Excel Copilot (Probably need to be improved)
    fn setup_system_instruction(&mut self) {
        let system_prompt = r#"You are Excel Copilot, an intelligent assistant specialized in Microsoft Excel automation and data manipulation. You always respond in English and help users accomplish their Excel tasks efficiently and accurately.

**YOUR PRIMARY RESPONSIBILITIES:**
1. **Follow user instructions precisely** - Do exactly what the user asks, no more, no less
2. **Use Excel tools proactively** - Leverage Excel functions to retrieve information and perform actions
3. **Be thorough and accurate** - Verify your actions and provide detailed, useful responses
4. **Explain your process** - Describe what you're doing and why during complex operations
5. **Analyze attached PDF documents** - If PDFs are attached, analyze them to extract relevant information and use it in Excel tasks
6. DO ALL THE WORK THAT THE USER ASKS FOR, NOT JUST PART OF IT.

**AVAILABLE EXCEL TOOLS:**
- `read_excel_cell`: Read the value from a specific cell (e.g., A1, B5, Z10)
- `write_excel_cell`: Write a value to a specific cell
- `read_excel_range`: Read data from a range of cells (e.g., A1:C10)
- `get_excel_sheet_overview`: Get a complete overview of all non-empty cells in the sheet
- `set_excel_formula`: Set formulas in cells (e.g., =SUM(A1:A10), =AVERAGE(B1:B5))

**PDF DOCUMENT MANAGEMENT:**
- If a PDF is attached to the message, analyze it to extract relevant data
- Use PDF information to automatically populate Excel
- Create tables based on PDF data
- Suggest formulas based on PDF content

**CRITICAL OPERATIONAL PROTOCOL:**
1. **ALWAYS start with sheet overview** - Use `get_excel_sheet_overview` at the beginning of each task to understand the current state
2. **ANALYZE ATTACHED PDFs** - If PDFs are provided, examine them to understand their content
3. **READ BEFORE WRITING** - Always check existing data before making modifications
4. **VERIFY FORMULAS IMMEDIATELY** - After setting ANY formula, re-read the cell to check for errors
5. **NEVER ACCEPT FORMULA ERRORS** - If you see ANY error (#NAME?, #REF!, #VALUE!, #DIV/0!), you MUST fix it:
   - #NAME? = Incorrect function name or syntax - Check spelling and syntax
   - #REF! = Invalid cell reference - Fix cell references
   - #VALUE! = Wrong data type - Check data types in referenced cells
   - #DIV/0! = Division by zero - Add error handling
6. **USE CORRECT SYNTAX** - Excel formulas MUST start with = and use proper cell references
7. **GET UPDATED CONTEXT** - After ANY modification, get a new sheet overview to see the current state

**FORMULA ERROR HANDLING PROTOCOL:**
- Step 1: Set the formula
- Step 2: Immediately re-read the cell
- Step 3: If result contains #NAME?, #REF?, #VALUE!, or #DIV/0! - ANALYZE AND FIX:
  * Check function spelling (SUM not sum, AVERAGE not average)
  * Verify cell references exist and contain numeric data
  * Ensure proper syntax with commas and parentheses
- Step 4: Set the corrected formula
- Step 5: Verify it calculates a NUMBER, not an error
- Step 6: NEVER leave error values in cells

**RESPONSE STYLE:**
- Be concise but informative
- Use clear, professional English
- Provide actionable insights when analyzing data
- Acknowledge when tasks are completed successfully
- Always report the actual calculated values from formulas, not just "success"
- If a formula returns an error, explain what's wrong and how you'll fix it, but don't let it as it is.
- Explicitly mention the content of attached PDFs and how they influence your actions

**IMPORTANT CONSTRAINTS:**
- Only perform actions explicitly requested by the user
- Always use the provided tools rather than making assumptions about Excel content
- If you encounter errors, explain them clearly and fix them immediately
- Respect data integrity - be careful with destructive operations
- Always verify that formulas produce the expected numeric results
- NEVER leave error values (#NAME?, #REF!, etc.) in any cell
- At the end of tasks when you think you're done, ALWAYS read the Excel file content and look for errors (e.g., columns or rows you missed), if there are any, you must correct them

You are now ready to help with Excel tasks. Always prioritize the user's specific needs and use your tools efficiently to accomplish their goals."#;

        self.system_instruction = SystemInstruction {
            parts: vec![GeminiPart::text(system_prompt.to_string())],
        };
    }

    /// Send a message to Gemini with detailed tool calling support
    pub async fn send_message_with_tools_detailed<F>(&mut self, message: &str, tool_executor: F) -> Result<GeminiResponseDetails, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(&str, &Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>> + Send + Sync,
    {
        // Add user message to conversation history
        let user_message = GeminiMessage {
            role: "user".to_string(),
            parts: vec![GeminiPart::text(message.to_string())],
        };
        self.conversation_history.push(user_message.clone());
        println!("Conversation history: {:?}", self.conversation_history);
        
        // Max iterations to prevent infinite loops
        let max_iterations = 50;
        let mut iteration = 0;
        let mut all_tool_calls = Vec::new();

        // Create a TEMPORARY conversation context for this request only
        // This will include tool calls and results, but won't pollute the permanent history
        let mut temp_conversation = self.conversation_history.clone();

        loop {
            iteration += 1;
            if iteration > max_iterations {
                return Err(format!("Maximum tool call iterations ({}) reached", max_iterations).into());
            }

            // Prepare the request with system instruction
            let url = format!("{}/gemini-2.0-flash:generateContent?key={}", self.base_url, self.api_key);
            
            let request_body = json!({
                "system_instruction": self.system_instruction,
                "contents": temp_conversation, // Use temporary context, not permanent history
                "tools": [{
                    "functionDeclarations": self.tools
                }],
                "toolConfig": {
                    "functionCallingConfig": {
                        "mode": "AUTO"
                    }
                },
                "generationConfig": {
                    "responseMimeType": "text/plain",
                    "temperature": 0.1,
                    "maxOutputTokens": 16000
                }
            });

            // Send request
            let response = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            if response.status().is_success() {
                let gemini_response: GeminiResponse = response.json().await?;
                
                if let Some(candidate) = gemini_response.candidates.first() {
                    let mut has_function_calls = false;
                    let mut response_text = String::new();
                    let mut current_iteration_tool_calls = Vec::new();
                    
                    // Process all parts in the response
                    for part in &candidate.content.parts {
                        match part {
                            GeminiResponsePart::Text { text } => {
                                response_text.push_str(text);
                            }
                            GeminiResponsePart::FunctionCall { function_call } => {
                                has_function_calls = true;
                                
                                // Execute the function call
                                let function_result = tool_executor(&function_call.name, &function_call.args).await?;
                                
                                // Record the tool call for this iteration
                                let tool_call_info = ToolCallInfo {
                                    function_name: function_call.name.clone(),
                                    arguments: function_call.args.clone(),
                                    result: function_result.clone(),
                                };
                                
                                current_iteration_tool_calls.push(tool_call_info.clone());
                                all_tool_calls.push(tool_call_info);
                            }
                        }
                    }
                    
                    if has_function_calls {
                        // Add model's function call to TEMPORARY context only
                        let model_function_call_message = GeminiMessage {
                            role: "model".to_string(),
                            parts: candidate.content.parts.clone().into_iter().map(|part| {
                                match part {
                                    GeminiResponsePart::Text { text } => GeminiPart::text(text),
                                    GeminiResponsePart::FunctionCall { function_call } => {
                                        GeminiPart::text(serde_json::to_string(&function_call).unwrap_or_default())
                                    }
                                }
                            }).collect(),
                        };
                        temp_conversation.push(model_function_call_message);
                        
                        // Add function responses to TEMPORARY context only  
                        for tool_call in &current_iteration_tool_calls {
                            let function_response_message = GeminiMessage {
                                role: "user".to_string(),
                                parts: vec![GeminiPart::text(tool_call.result.clone())],
                            };
                            temp_conversation.push(function_response_message);
                        }
                        
                        // Continue the conversation to get final response
                        continue;
                    } else {
                        // No function calls, this is the final text response
                        if !response_text.is_empty() {
                            // ONLY add the clean final text response to permanent history
                            let assistant_message = GeminiMessage {
                                role: "model".to_string(),
                                parts: vec![GeminiPart::text(response_text.clone())],
                            };
                            self.conversation_history.push(assistant_message);
                            
                            let has_tool_calls = !all_tool_calls.is_empty();
                            return Ok(GeminiResponseDetails {
                                content: response_text,
                                tool_calls: all_tool_calls,
                                iterations: iteration,
                                has_tool_calls,
                            });
                        } else {
                            // Empty response, but we may have executed functions
                            if !all_tool_calls.is_empty() {
                                let summary = format!("Task completed successfully. {} tools executed.", all_tool_calls.len());
                                
                                // Add only a clean summary to permanent history
                                let assistant_message = GeminiMessage {
                                    role: "model".to_string(),
                                    parts: vec![GeminiPart::text(summary.clone())],
                                };
                                self.conversation_history.push(assistant_message);
                                
                                return Ok(GeminiResponseDetails {
                                    content: summary,
                                    tool_calls: all_tool_calls,
                                    iterations: iteration,
                                    has_tool_calls: true,
                                });
                            } else {
                                return Err("Empty response from Gemini without function calls".into());
                            }
                        }
                    }
                }
                
                return Err("No candidate in Gemini response".into());
            } else {
                let error_text = response.text().await?;
                return Err(format!("Gemini API error: {}", error_text).into());
            }
        }
    }

    /// Clear conversation history
    pub fn clear_conversation(&mut self) {
        // Clear all conversation history but keep system instruction
        self.conversation_history.clear();
    }

    /// Reset client completely
    pub fn reset_completely(&mut self) {
        // Clear everything and re-setup system instruction
        self.conversation_history.clear();
        self.setup_system_instruction();
    }

    /// Send a message with tool calling support (legacy compatibility)
    pub async fn send_message_with_tools<F>(&mut self, message: &str, tool_executor: F) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(&str, &Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>> + Send + Sync,
    {
        let details = self.send_message_with_tools_detailed(message, tool_executor).await?;
        Ok(details.content)
    }

    /// Send a simple message without tools
    pub async fn send_message(&mut self, message: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Add user message to conversation history
        let user_message = GeminiMessage {
            role: "user".to_string(),
            parts: vec![GeminiPart::text(message.to_string())],
        };
        self.conversation_history.push(user_message);

        // Prepare the request without tools but with system instruction
        let url = format!("{}/gemini-2.5-flash-preview-04-17:generateContent?key={}", self.base_url, self.api_key);
        
        let request_body = json!({
            "system_instruction": self.system_instruction,
            "contents": self.conversation_history,
            "generationConfig": {
                "responseMimeType": "text/plain"
            }
        });

        // Send request
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let gemini_response: GeminiResponse = response.json().await?;
            
            if let Some(candidate) = gemini_response.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    if let GeminiResponsePart::Text { text } = part {
                        let response_text = text.clone();
                        
                        // Add assistant response to conversation history
                        let assistant_message = GeminiMessage {
                            role: "model".to_string(),
                            parts: vec![GeminiPart::text(response_text.clone())],
                        };
                        self.conversation_history.push(assistant_message);
                        
                        return Ok(response_text);
                    }
                }
            }
            
            Err("Empty response from Gemini".into())
        } else {
            let error_text = response.text().await?;
            Err(format!("Gemini API error: {}", error_text).into())
        }
    }

    /// Send a message with PDF attachments
    pub async fn send_message_with_pdf<F>(&mut self, message: &str, pdf_files: Vec<String>, tool_executor: F) -> Result<GeminiResponseDetails, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(&str, &Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>> + Send + Sync,
    {
        // Create parts for the message
        let mut parts = vec![GeminiPart::text(message.to_string())];
        
        // Add PDF attachments
        for pdf_path in pdf_files {
            match std::fs::read(&pdf_path) {
                Ok(pdf_data) => {
                    let base64_data = general_purpose::STANDARD.encode(pdf_data);
                    parts.push(GeminiPart::pdf(base64_data));
                    println!("Added PDF attachment: {}", pdf_path);
                }
                Err(e) => {
                    println!("Warning: Failed to read PDF {}: {}", pdf_path, e);
                }
            }
        }

        // Add user message with attachments to conversation history
        let user_message = GeminiMessage {
            role: "user".to_string(),
            parts,
        };
        
        // Use the existing detailed method but modify the conversation temporarily
        let _original_history = self.conversation_history.clone();
        self.conversation_history.push(user_message);
        
        let result = self.send_message_with_tools_detailed("", tool_executor).await;
        
        // Restore original history structure (the detailed method will have added the proper messages)
        result
    }
} 