use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};
use crate::locale_utils::{get_decimal_separator, get_locale_info};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiMessage {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum GeminiPart {
    Text { 
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>
    },
    InlineData { 
        #[serde(skip_serializing_if = "Option::is_none", rename = "inlineData")]
        inline_data: Option<InlineData>
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCall
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponse
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String, // base64 encoded
}

impl GeminiPart {
    pub fn text(text: String) -> Self {
        Self::Text {
            text: Some(text),
        }
    }
    
    pub fn pdf(base64_data: String) -> Self {
        Self::InlineData {
            inline_data: Some(InlineData {
                mime_type: "application/pdf".to_string(),
                data: base64_data,
            }),
        }
    }
    
    pub fn function_call(name: String, args: Value) -> Self {
        Self::FunctionCall {
            function_call: FunctionCall {
                name,
                args,
            },
        }
    }
    
    pub fn function_response(name: String, response: Value) -> Self {
        Self::FunctionResponse {
            function_response: FunctionResponse {
                name,
                response,
            },
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
pub struct FunctionResponse {
    pub name: String,
    pub response: Value,
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
    copilot_enabled: bool,
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
    pub fn new(api_key: String, copilot_enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let base_url = "https://generativelanguage.googleapis.com/v1beta/models".to_string();
        
        let mut gemini_client = Self {
            client,
            api_key,
            base_url,
            conversation_history: Vec::new(),
            tools: Vec::new(),
            system_instruction: SystemInstruction { parts: Vec::new() },
            copilot_enabled,
        };

        // Setup Excel tools (conditionally include Copilot tools)
        gemini_client.setup_excel_tools();
        
        // Setup system instruction
        gemini_client.setup_system_instruction();
        
        Ok(gemini_client)
    }

    /// Configure Excel automation tools for Gemini
    fn setup_excel_tools(&mut self) {
        let mut tools = vec![
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

        // Add Copilot tools only if enabled
        if self.copilot_enabled {
            tools.extend(vec![
                FunctionDeclaration {
                    name: "send_request_to_excel_copilot".to_string(),
                    description: "Send a request to Microsoft Excel Copilot for advanced tasks like formatting, charts, conditional formatting, data analysis, etc. Note: This will work on currently selected cells or entire sheet.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "request": {
                                "type": "string",
                                "description": "The request to send to Excel Copilot (e.g., 'Create a bar chart from this data', 'Format these cells with bold and red background', 'Apply conditional formatting to highlight values greater than 100')"
                            }
                        },
                        "required": ["request"]
                    }),
                },
                FunctionDeclaration {
                    name: "format_cells_with_copilot".to_string(),
                    description: "Format Excel cells using Copilot with specific formatting instructions. This will first select the specified range, then use Copilot to format it.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "range": {
                                "type": "string",
                                "description": "The cell range to format (e.g., A1:C10, B1:D8)"
                            },
                            "format_description": {
                                "type": "string",
                                "description": "Description of the formatting to apply (e.g., 'bold text with blue background', 'currency format', 'percentage with 2 decimals')"
                            }
                        },
                        "required": ["range", "format_description"]
                    }),
                },
                FunctionDeclaration {
                    name: "create_chart_with_copilot".to_string(),
                    description: "Create charts in Excel using Copilot. This will first select the specified data range, then ask Copilot to create the chart.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "data_range": {
                                "type": "string",
                                "description": "The data range for the chart (e.g., A1:B10, B1:D8)"
                            },
                            "chart_type": {
                                "type": "string",
                                "description": "Type of chart to create (e.g., 'bar chart', 'line chart', 'pie chart', 'scatter plot')"
                            }
                        },
                        "required": ["data_range", "chart_type"]
                    }),
                },
                FunctionDeclaration {
                    name: "apply_conditional_formatting_with_copilot".to_string(),
                    description: "Apply conditional formatting to Excel cells using Copilot. This will first select the specified range, then apply the conditional formatting.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "range": {
                                "type": "string",
                                "description": "The cell range to apply conditional formatting (e.g., A1:D20, B1:E15)"
                            },
                            "condition": {
                                "type": "string",
                                "description": "The condition for formatting (e.g., 'values greater than 100', 'duplicate values', 'top 10 values')"
                            }
                        },
                        "required": ["range", "condition"]
                    }),
                },
            ]);
        }

        self.tools = tools;
    }

    /// Setup the system instruction for the Excel Copilot (Probably need to be improved)
    fn setup_system_instruction(&mut self) {
        // Get locale information
        let locale_info = get_locale_info();
        let decimal_sep = get_decimal_separator();
        
        // Create locale-specific formatting rules
        let number_format_rules = if decimal_sep == ',' {
            format!(r#"**NUMBER FORMAT NORMALIZATION (LOCALE: COMMA DECIMAL):**
Current system locale: {}
- When writing numeric values to Excel, normalize according to your locale:
  - Convert "+2.259,84" to "2259.84" (remove + sign, thousands separators, convert comma to dot for Excel)
  - Convert "-1.458,25" to "-1458.25" (remove thousands separators, convert comma to dot for Excel)
  - Convert "1.234.567,89" to "1234567.89" (remove thousands separators, convert comma to dot for Excel)
  - Always convert final comma decimal separator to dot (.) for Excel compatibility
  - Remove thousands separators (dots in your locale)
- Display numbers to user using local format: 1234,67 but write to Excel as: 1234.67
- NEVER add apostrophes (') to numeric values - this makes them text"#, locale_info)
        } else {
            format!(r#"**NUMBER FORMAT NORMALIZATION (LOCALE: DOT DECIMAL):**
Current system locale: {}
- When writing numeric values to Excel, normalize according to your locale:
  - Convert "+2,259.84" to "2259.84" (remove + sign and comma thousands separators)
  - Convert "-1,458.25" to "-1458.25" (remove comma thousands separators, keep - sign)
  - Convert "1,234,567.89" to "1234567.89" (remove comma thousands separators)
  - Use dot (.) as decimal separator, remove comma thousands separators
  - This ensures Excel recognizes values as numbers for calculations
- NEVER add apostrophes (') to numeric values - this makes them text"#, locale_info)
        };

        let base_prompt = r#"You are Excel Copilot. You interact with Excel files using tools. Always respond in English or French as requested.

**TOOLS AVAILABLE:**
- Basic: read_excel_cell, write_excel_cell, read_excel_range, get_excel_sheet_overview, set_excel_formula"#;

        let copilot_section = if self.copilot_enabled {
            r#"
- MS Copilot: send_request_to_excel_copilot, format_cells_with_copilot, create_chart_with_copilot, apply_conditional_formatting_with_copilot

**MS EXCEL COPILOT CONSTRAINTS:**
- CRITICAL: Copilot ONLY works on files saved in OneDrive
- Data range MUST have at least 3 rows and 2 columns
- Headers are REQUIRED in the first row with different names
- Before using Copilot tools, ALWAYS verify data meets these requirements

**MS EXCEL COPILOT INTERACTION SEQUENCE:**
- The system automatically follows this sequence: Select Range → Click "Copilot" → Click "Ask Copilot" → Wait 5s → Send Request → Apply Changes
- For range-based tools (format_cells_with_copilot, create_chart_with_copilot, apply_conditional_formatting_with_copilot), the range is selected FIRST
- For general requests (send_request_to_excel_copilot), it works on currently selected area or entire sheet

**WHEN TO USE MS COPILOT:**
- Complex formatting requests → format_cells_with_copilot (auto-selects range)
- Chart creation → create_chart_with_copilot (auto-selects data range)
- Conditional formatting → apply_conditional_formatting_with_copilot (auto-selects range)
- Data analysis, pivot tables, advanced features → send_request_to_excel_copilot

**COPILOT DATA VALIDATION:**
- Before any Copilot operation, check data has ≥3 rows, ≥2 columns
- Verify headers exist in first row
- If requirements not met, explain what's needed

**COPILOT BEST PRACTICES:**
- Always specify clear ranges for formatting and charts (e.g., "B1:D8")
- Use descriptive requests like "Create a bar chart from this data" rather than "Create bar chart from B1:D8"
- For formatting, use natural language: "bold text with blue background" instead of technical formatting codes"#
        } else {
            ""
        };

        let common_rules = format!(r#"

**CRITICAL RULES:**
1. ALWAYS use tools to interact with Excel - NEVER fake results
2. Start every task with get_excel_sheet_overview
3. NEVER invent cell contents or data
4. If you need info, READ it with tools first
5. Don't talk between toolcalls, just use them
6. Fix formula errors immediately (#NAME?, #REF!, etc.)
7. After writing formulas, re-read the cell to verify

{}

**WHEN TO USE BASIC TOOLS:**
- Simple cell reading/writing → read_excel_cell, write_excel_cell
- Basic formulas → set_excel_formula
- Data overview → get_excel_sheet_overview

**FORBIDDEN:**
- Simulating tool outputs like "Reading A1... value is 42"
- Pretending you called a function
- Inventing Excel data
- Leaving error values in cells
- Writing numbers with apostrophes or commas to Excel

**PROTOCOL:**
- Need data? → Call appropriate read tool
- Need simple write? → Call write_excel_cell (with normalized numbers)"#, number_format_rules);

        let final_protocol = if self.copilot_enabled {
            r#"
- Need formatting/charts with specific range? → Use MS Copilot range-based tools (check requirements first)
- Need general analysis? → Use send_request_to_excel_copilot (check requirements first)
- Lack context? → READ THE FILE
- See errors? → Fix them immediately

Use tools efficiently. Be direct. No hallucination. Leverage MS Copilot for advanced Excel features with proper range selection and data validation."#
        } else {
            r#"
- Lack context? → READ THE FILE
- See errors? → Fix them immediately

Use tools efficiently. Be direct. No hallucination. Copilot features are disabled - use basic tools only."#
        };

        let system_prompt = format!("{}{}{}{}", base_prompt, copilot_section, common_rules, final_protocol);

        self.system_instruction = SystemInstruction {
            parts: vec![GeminiPart::text(system_prompt)],
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
        self.conversation_history.push(user_message);
        
        // Track all tool calls made during this conversation turn
        let mut all_tool_calls = Vec::new();
        let mut total_iterations = 0;
        
        // Start the conversation turn
        let final_response = self.process_conversation_turn(&tool_executor, &mut all_tool_calls, &mut total_iterations).await?;
        
        // Add the final response to conversation history
        let final_message = GeminiMessage {
            role: "model".to_string(),
            parts: vec![GeminiPart::text(final_response.clone())],
        };
        self.conversation_history.push(final_message);
        
        // Calculate has_tool_calls before moving all_tool_calls
        let has_tool_calls = !all_tool_calls.is_empty();
        
        Ok(GeminiResponseDetails {
            content: final_response,
            tool_calls: all_tool_calls,
            iterations: total_iterations,
            has_tool_calls,
        })
    }

    /// Process a complete conversation turn, handling function calls inline
    async fn process_conversation_turn<F>(&mut self, tool_executor: &F, all_tool_calls: &mut Vec<ToolCallInfo>, total_iterations: &mut i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(&str, &Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>> + Send + Sync,
    {
        let max_total_iterations = 50;
        let mut accumulated_text = String::new();
        
        // Working conversation that includes intermediate function calls
        let mut working_conversation = self.conversation_history.clone();
        
        loop {
            *total_iterations += 1;
            if *total_iterations > max_total_iterations {
                return Err(format!("Maximum iterations ({}) reached", max_total_iterations).into());
            }
            
            // Make API call to Gemini
            let url = format!("{}/gemini-2.0-flash:generateContent?key={}", self.base_url, self.api_key);
            
            let request_body = json!({
                "system_instruction": self.system_instruction,
                "contents": working_conversation,
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

            let response = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(format!("Gemini API error: {}", error_text).into());
            }

            let gemini_response: GeminiResponse = response.json().await?;
            let candidate = gemini_response.candidates.first()
                .ok_or("No candidate in Gemini response")?;

            // DEBUG: Log what we received from Gemini
            println!("=== GEMINI RESPONSE DEBUG (Iteration {}) ===", *total_iterations);
            println!("Finish reason: {:?}", candidate.finish_reason);
            println!("Number of parts: {}", candidate.content.parts.len());
            for (i, part) in candidate.content.parts.iter().enumerate() {
                match part {
                    GeminiResponsePart::Text { text } => {
                        println!("Part {}: TEXT: {}", i, text.chars().take(100).collect::<String>());
                    }
                    GeminiResponsePart::FunctionCall { function_call } => {
                        println!("Part {}: FUNCTION_CALL: {}", i, function_call.name);
                    }
                }
            }
            println!("==========================================");

            // Process the response parts
            let mut has_function_calls = false;
            let mut text_content = String::new();
            let mut function_calls_in_response = Vec::new();
            
            // Collect all text and function calls from this response
            for part in &candidate.content.parts {
                match part {
                    GeminiResponsePart::Text { text } => {
                        text_content.push_str(text);
                    }
                    GeminiResponsePart::FunctionCall { function_call } => {
                        has_function_calls = true;
                        function_calls_in_response.push(function_call.clone());
                    }
                }
            }
            
            // Add text to accumulated response (this preserves all text from the model)
            if !text_content.trim().is_empty() {
                accumulated_text.push_str(&text_content);
            }
            
            // Add the model's response to working conversation (represents what model said in this turn)
            let model_parts: Vec<GeminiPart> = candidate.content.parts.iter().map(|part| {
                match part {
                    GeminiResponsePart::Text { text } => GeminiPart::text(text.clone()),
                    GeminiResponsePart::FunctionCall { function_call } => {
                        // Include the actual function call, not as text
                        GeminiPart::function_call(function_call.name.clone(), function_call.args.clone())
                    }
                }
            }).collect();
            
            if !model_parts.is_empty() {
                let model_message = GeminiMessage {
                    role: "model".to_string(),
                    parts: model_parts,
                };
                working_conversation.push(model_message);
            }
            
            // If there are function calls, execute them and add results for next iteration
            if has_function_calls {
                println!("=== EXECUTING {} FUNCTION CALLS ===", function_calls_in_response.len());
                
                // Collect all function responses before adding them to the conversation
                let mut function_response_parts = Vec::new();
                
                for function_call in function_calls_in_response {
                    println!("Executing function: {}", function_call.name);
                    
                    // Execute the function
                    let function_result = tool_executor(&function_call.name, &function_call.args).await?;
                    
                    println!("Function result length: {}", function_result.len());
                    
                    // Record the tool call
                    let tool_call_info = ToolCallInfo {
                        function_name: function_call.name.clone(),
                        arguments: function_call.args.clone(),
                        result: function_result.clone(),
                    };
                    all_tool_calls.push(tool_call_info);
                    
                    // Collect the function response part
                    function_response_parts.push(GeminiPart::function_response(
                        function_call.name.clone(),
                        json!({ "content": function_result })
                    ));
                }
                
                // Add all function responses in a single message
                let function_results_message = GeminiMessage {
                    role: "user".to_string(),
                    parts: function_response_parts,
                };
                working_conversation.push(function_results_message);
                
                println!("=== CONTINUING TO NEXT ITERATION ===");
                // Continue to let the model see the function results and potentially continue its response
                continue;
            } else {
                println!("=== NO FUNCTION CALLS - BREAKING ===");
                // No function calls means the model has finished its response
                break;
            }
        }
        
        // Return the accumulated text from all iterations
        let final_response = if accumulated_text.trim().is_empty() {
            if !all_tool_calls.is_empty() {
                format!("Task completed successfully. {} tools executed.", all_tool_calls.len())
            } else {
                "Task completed.".to_string()
            }
        } else {
            accumulated_text.trim().to_string()
        };
        
        Ok(final_response)
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
        self.conversation_history.push(user_message);
        
        // Track all tool calls made during this conversation turn
        let mut all_tool_calls = Vec::new();
        let mut total_iterations = 0;
        
        // Start the conversation turn
        let final_response = self.process_conversation_turn(&tool_executor, &mut all_tool_calls, &mut total_iterations).await?;
        
        // Add the final response to conversation history
        let final_message = GeminiMessage {
            role: "model".to_string(),
            parts: vec![GeminiPart::text(final_response.clone())],
        };
        self.conversation_history.push(final_message);
        
        // Calculate has_tool_calls before moving all_tool_calls
        let has_tool_calls = !all_tool_calls.is_empty();
        
        Ok(GeminiResponseDetails {
            content: final_response,
            tool_calls: all_tool_calls,
            iterations: total_iterations,
            has_tool_calls,
        })
    }
} 