use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{State, AppHandle};
use tauri_plugin_dialog::DialogExt;

use crate::excel::{ExcelWorkbook, ExcelSheet, CellPosition, CellRange};
use crate::gemini::GeminiClient;
use crate::excel_interaction::{get_excel_automation};
use anyhow::Result;
use serde_json::Value;

/// Convert column index to Excel letter notation (A, B, C, ..., Z, AA, AB, ...)
fn column_index_to_letter(mut index: usize) -> String {
    let mut result = String::new();
    loop {
        result.insert(0, (b'A' + (index % 26) as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    result
}

/// Global state for storing the current Excel workbook
pub type AppState = Mutex<Option<ExcelWorkbook>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelData {
    pub file_path: Option<String>,
    pub sheets: Vec<ExcelSheet>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub function_name: String,
    pub arguments: Value,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseDetails {
    pub has_tool_calls: bool,
    pub iterations: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub response_details: Option<ResponseDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRangeData {
    pub range_notation: String,
    pub cells: Vec<CellData>,
    pub display_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub notation: String,
    pub value: String,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnData {
    pub column_notation: String,
    pub cells: Vec<CellData>,
    pub display_text: String,
}

impl From<ExcelWorkbook> for ExcelData {
    fn from(workbook: ExcelWorkbook) -> Self {
        Self {
            file_path: workbook.file_path.clone(),
            sheets: workbook.sheets.iter().cloned().collect(),
            summary: workbook.get_content_summary(),
        }
    }
}

#[derive(Default)]
pub struct AppStateStruct {
    pub excel_workbook: Mutex<Option<ExcelWorkbook>>,
    pub gemini_client: Mutex<Option<GeminiClient>>,
    pub chat_history: Mutex<Vec<ChatMessage>>,
    pub current_file_path: Mutex<Option<String>>,
}

/// Open an Excel file using the system dialog
#[tauri::command]
pub async fn open_excel_file(
    app_handle: AppHandle,
    state: State<'_, AppStateStruct>,
) -> Result<String, String> {
    let files = app_handle.dialog().file()
        .set_title("Open Excel file")
        .add_filter("Excel Files", &["xlsx", "xls"])
        .blocking_pick_file();

    if let Some(file_path) = files {
        let path_str = file_path.to_string();
        
        // Update current file path
        {
            let mut current_path = state.current_file_path.lock().unwrap();
            *current_path = Some(path_str.clone());
        }

        // Try to open Excel using the system default application
        match std::process::Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(&path_str)
            .spawn()
        {
            Ok(_) => {
                // Also load for internal representation if possible
                match ExcelWorkbook::from_file(&path_str) {
                    Ok(workbook) => {
                        let mut excel_state = state.excel_workbook.lock().unwrap();
                        *excel_state = Some(workbook);
                    }
                    Err(_) => {
                        // This is fine - we opened in Excel but couldn't parse internally
                    }
                }
                Ok(format!("✅ Excel file opened: {}", path_str))
            }
            Err(e) => Err(format!("❌ Error opening file: {}", e))
        }
    } else {
        Err("❌ No file selected".to_string())
    }
}

/// Create a new Excel workbook
#[tauri::command]
pub async fn create_new_excel(
    state: State<'_, AppStateStruct>,
) -> Result<String, String> {
    let mut excel_state = state.excel_workbook.lock().unwrap();

    let workbook = ExcelWorkbook::new();
    *excel_state = Some(workbook);

    // Create a temporary file and open it in Excel
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("new_excel_sheet.xlsx");
    
    {
        let mut current_path = state.current_file_path.lock().unwrap();
        *current_path = Some(temp_file.to_string_lossy().to_string());
    }

    // Save the empty workbook
    if let Some(ref workbook) = *excel_state {
        workbook.save_to_file(&temp_file)
            .map_err(|e| format!("Error creating file: {}", e))?;
    }

    // Open in Excel
    match std::process::Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg(&temp_file.to_string_lossy().to_string())
        .spawn()
    {
        Ok(_) => Ok("✅ New Excel file created and opened".to_string()),
        Err(e) => Err(format!("❌ Error opening file: {}", e))
    }
}

/// Save the current Excel workbook
#[tauri::command]
pub async fn save_excel_file(
    state: State<'_, AppStateStruct>,
    file_path: Option<String>,
) -> Result<String, String> {
    let excel_state = state.excel_workbook.lock().unwrap();
    if let Some(workbook) = excel_state.as_ref() {
        let path = if let Some(path) = file_path {
            path
        } else {
            let current_path = state.current_file_path.lock().unwrap();
            if let Some(ref path) = *current_path {
                path.clone()
            } else {
                return Err("❌ No file path specified".to_string());
            }
        };

        workbook.save_to_file(&path)
            .map_err(|e| format!("Error saving file: {}", e))?;

        {
            let mut current_path = state.current_file_path.lock().unwrap();
            *current_path = Some(path.clone());
        }

        Ok(format!("File saved: {}", path))
    } else {
        Err("No Excel file loaded".to_string())
    }
}

/// Get the content of the current Excel workbook
#[tauri::command]
pub async fn get_excel_content(
    state: State<'_, AppStateStruct>,
) -> Result<HashMap<String, Vec<Vec<String>>>, String> {
    let excel_state = state.excel_workbook.lock().unwrap();
    if let Some(workbook) = excel_state.as_ref() {
        let mut content = HashMap::new();
        
        for sheet in &workbook.sheets {
            content.insert(sheet.name.clone(), sheet.data.clone());
        }
        
        Ok(content)
    } else {
        Err("No Excel file loaded".to_string())
    }
}

/// Configure the Gemini AI client with API key
#[tauri::command]
pub async fn setup_gemini_client(
    state: State<'_, AppStateStruct>,
    api_key: String,
) -> Result<String, String> {
    let mut gemini_state = state.gemini_client.lock().unwrap();
    
    let client = GeminiClient::new(api_key)
        .map_err(|e| format!("Error creating Gemini client: {}", e))?;
    
    *gemini_state = Some(client);
    Ok("✅ Gemini client configured successfully".to_string())
}

/// Send a chat message to Gemini with tool calling support
#[tauri::command]
pub async fn chat_with_gemini(
    state: State<'_, AppStateStruct>,
    message: String,
) -> Result<String, String> {
    // Clone the client to avoid holding the lock across await
    let client_opt = {
        let gemini_state = state.gemini_client.lock().unwrap();
        gemini_state.clone()
    };
    
    if let Some(mut client) = client_opt {
        // Add user message to history
        {
            let mut chat_history = state.chat_history.lock().unwrap();
            chat_history.push(ChatMessage {
                role: "user".to_string(),
                content: message.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                tool_calls: None,
                response_details: None,
            });
        }

        // Create tool executor that uses the original state directly
        let tool_executor = |function_name: &str, args: &Value| {
            let function_name_owned = function_name.to_string();
            let args_owned = args.clone();
            
            // Get the current file path directly from state when needed
            let current_file_path = {
                let current_path = state.current_file_path.lock().unwrap();
                current_path.as_ref().map(|p| p.to_string())
            };
            
            Box::pin(async move {
                // Execute tool with just the file path instead of full state
                execute_excel_tool_with_path(&function_name_owned, &args_owned, current_file_path.as_deref()).await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("{}", e).into() })
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>
        };


        match client.send_message_with_tools_detailed(&message, tool_executor).await {
            Ok(response) => {
                // Convert tool calls to our format
                let tool_calls: Vec<ToolCall> = response.tool_calls.iter().map(|tc| ToolCall {
                    function_name: tc.function_name.clone(),
                    arguments: tc.arguments.clone(),
                    result: tc.result.clone(),
                }).collect();

                // Add final assistant response to history
                {
                    let mut chat_history = state.chat_history.lock().unwrap();
                    chat_history.push(ChatMessage {
                        role: "model".to_string(),
                        content: response.content.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                        response_details: Some(ResponseDetails {
                            has_tool_calls: response.has_tool_calls,
                            iterations: response.iterations,
                        }),
                    });
                }

                // Update the client state with the updated client that contains conversation history
                {
                    let mut gemini_state = state.gemini_client.lock().unwrap();
                    *gemini_state = Some(client);
                }

                Ok(response.content)
            }
            Err(e) => {
                // Add error message to history
                {
                    let mut chat_history = state.chat_history.lock().unwrap();
                    chat_history.push(ChatMessage {
                        role: "model".to_string(),
                        content: format!("Error: {}", e),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        tool_calls: None,
                        response_details: None,
                    });
                }
                
                Err(format!("Gemini API error: {}", e))
            }
        }
    } else {
        Err("Gemini client not configured. Please set up your API key first.".to_string())
    }
}

/// Execute Excel tools via automation with file path
async fn execute_excel_tool_with_path(
    function_name: &str, 
    args: &Value,
    file_path: Option<&str>
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let automation = get_excel_automation().await
        .map_err(|e| format!("Failed to get Excel automation: {}", e))?;

    let result = match function_name {
        "read_excel_cell" => {
            let cell_address = args["cell_address"].as_str()
                .ok_or("Missing or invalid cell_address parameter")?;
            
            let cell = automation.read_cell(cell_address).await
                .map_err(|e| format!("Failed to read cell: {}", e))?;
            
            format!("SUCCESS: Read cell {} - Value: '{}'. The cell contains the value '{}' and is ready for further operations.", 
                      cell.address, cell.value, cell.value)
        }
        "write_excel_cell" => {
            let cell_address = args["cell_address"].as_str()
                .ok_or("Missing or invalid cell_address parameter")?;
            let value = args["value"].as_str()
                .ok_or("Missing or invalid value parameter")?;
            
            automation.write_cell(cell_address, value).await
                .map_err(|e| format!("Failed to write cell: {}", e))?;
                
            // Save the file after writing
            automation.save_current_file().await
                .map_err(|e| format!("Failed to save file after write: {}", e))?;
                
            // Verify the write by reading back the value
            let verification = automation.read_cell(cell_address).await
                .map_err(|e| format!("Failed to verify write: {}", e))?;
            
            format!("SUCCESS: Wrote '{}' to cell {}. VERIFICATION: Cell {} now contains '{}'. The write operation is complete and verified.", 
                      value, cell_address, verification.address, verification.value)
        }
        "read_excel_range" => {
            let start_cell = args["start_cell"].as_str()
                .ok_or("Missing or invalid start_cell parameter")?;
            let end_cell = args["end_cell"].as_str()
                .ok_or("Missing or invalid end_cell parameter")?;
            
            let range = automation.read_range(start_cell, end_cell).await
                .map_err(|e| format!("Failed to read range: {}", e))?;
            
            if range.values.is_empty() {
                format!("SUCCESS: Read range {}:{} - The range is empty (0 rows). No data found in the specified range.", start_cell, end_cell)
            } else {
                let mut result = format!("SUCCESS: Read range {}:{} - Found {} rows of data:\n", start_cell, end_cell, range.values.len());
                for (row_idx, row) in range.values.iter().enumerate() {
                    result.push_str(&format!("Row {}: {}\n", row_idx + 1, row.join(" | ")));
                }
                result.push_str(&format!("Range contains {} rows and {} columns.", range.values.len(), range.values.get(0).map(|r| r.len()).unwrap_or(0)));
                result
            }
        }
        "set_excel_formula" => {
            let cell_address = args["cell_address"].as_str()
                .ok_or("Missing or invalid cell_address parameter")?;
            let formula = args["formula"].as_str()
                .ok_or("Missing or invalid formula parameter")?;
            
            automation.set_formula(cell_address, formula).await
                .map_err(|e| format!("Failed to set formula: {}", e))?;
                
            automation.save_current_file().await
                .map_err(|e| format!("Failed to save file after formula: {}", e))?;
                
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            let result_cell = automation.read_cell(cell_address).await
                .map_err(|e| format!("Failed to read formula result: {}", e))?;
            
            if result_cell.value.starts_with('#') {
                format!("ERROR DETECTED: Formula '{}' in cell {} resulted in '{}' error. This indicates a problem with the formula that needs to be fixed.", 
                          formula, cell_address, result_cell.value)
            } else {
                format!("SUCCESS: Set formula '{}' in cell {}. RESULT: The formula calculated to '{}'. Formula operation completed successfully.", 
                          formula, cell_address, result_cell.value)
            }
        }
        "get_excel_sheet_overview" => {
            get_complete_excel_status_from_file(file_path).await
                .map_err(|e| format!("Failed to get sheet overview: {}", e))?
        }
        _ => {
            return Err(format!("ERROR: Unknown function '{}'. Available functions: read_excel_cell, write_excel_cell, read_excel_range, get_excel_sheet_overview, set_excel_formula", function_name).into());
        }
    };

    // Always append complete Excel status after every operation
    let mut final_result = result;
    
    // For all operations, always append current Excel state
    match get_complete_excel_status_from_file(file_path).await {
        Ok(excel_status) => {
            final_result.push_str("\n\n");
            final_result.push_str(&excel_status);
        }
        Err(e) => {
            final_result.push_str(&format!("\n\n Warning: Could not read current Excel file status: {}", e));
        }
    }

    Ok(final_result)
}

/// Get complete Excel status by reading the file directly from disk
async fn get_complete_excel_status_from_file(file_path: Option<&str>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(path) = file_path {
        match crate::excel::ExcelWorkbook::from_file(path) {
            Ok(workbook) => {
                let mut status = String::from("📊 CURRENT EXCEL FILE STATUS:\n");
                status.push_str("=====================================\n");
                
                if workbook.sheets.is_empty() {
                    status.push_str("No sheets found in the workbook.\n");
                    return Ok(status);
                }
                
                for (sheet_idx, sheet) in workbook.sheets.iter().enumerate() {
                    status.push_str(&format!("📋 SHEET {} - '{}'\n", sheet_idx + 1, sheet.name));
                    status.push_str("-------------------------------------\n");
                    
                    let mut non_empty_cells = Vec::new();
                    
                    // Find all non-empty cells in this sheet
                    for (row_idx, row) in sheet.data.iter().enumerate() {
                        for (col_idx, cell_value) in row.iter().enumerate() {
                            if !cell_value.trim().is_empty() {
                                let col_letter = column_index_to_letter(col_idx);
                                let cell_address = format!("{}{}", col_letter, row_idx + 1);
                                non_empty_cells.push((cell_address, cell_value.clone()));
                            }
                        }
                    }
                    
                    if non_empty_cells.is_empty() {
                        status.push_str("Sheet is empty - no data found.\n");
                    } else {
                        status.push_str(&format!("Found {} non-empty cells:\n", non_empty_cells.len()));
                        
                        for (address, value) in non_empty_cells.iter() {
                            // Trim long values for readability
                            let display_value = if value.len() > 50 {
                                format!("{}...", &value[..47])
                            } else {
                                value.clone()
                            };
                            status.push_str(&format!(" {}: '{}'\n", address, display_value));
                        }
                    }
                    
                    status.push_str("\n");
                }
                
                // Summary
                let total_sheets = workbook.sheets.len();
                let total_cells: usize = workbook.sheets.iter()
                    .map(|sheet| {
                        sheet.data.iter()
                            .map(|row| row.iter().filter(|cell| !cell.trim().is_empty()).count())
                            .sum::<usize>()
                    })
                    .sum();
                
                status.push_str("=====================================\n");
                status.push_str(&format!("SUMMARY: {} sheet(s), {} non-empty cell(s) total\n", total_sheets, total_cells));
                status.push_str("=====================================");
                
                Ok(status)
            }
            Err(e) => {
                Err(format!("Failed to read Excel file '{}': {}", path, e).into())
            }
        }
    } else {
        Err("No file path provided - cannot read Excel status".into())
    }
}

/// Get chat history
#[tauri::command]
pub async fn get_chat_history(
    state: State<'_, AppStateStruct>,
) -> Result<Vec<ChatMessage>, String> {
    let chat_history = state.chat_history.lock().unwrap();
    Ok(chat_history.clone())
}

/// Clear chat history and reset conversation
#[tauri::command]
pub async fn clear_chat_history(
    state: State<'_, AppStateStruct>,
) -> Result<String, String> {
    {
        let mut chat_history = state.chat_history.lock().unwrap();
        chat_history.clear();
    }
    
    {
        let mut gemini_state = state.gemini_client.lock().unwrap();
        if let Some(client) = gemini_state.as_mut() {
            client.clear_conversation();
        }
    }
    
    Ok("✅ Chat history cleared".to_string())
}

/// Send a chat message to Gemini with PDF attachments
#[tauri::command]
pub async fn chat_with_gemini_pdf(
    state: State<'_, AppStateStruct>,
    message: String,
    pdf_files: Vec<String>,
) -> Result<String, String> {
    // Clone the client to avoid holding the lock across await
    let client_opt = {
        let gemini_state = state.gemini_client.lock().unwrap();
        gemini_state.clone()
    };
    
    if let Some(mut client) = client_opt {
        // Create tool executor
        let tool_executor = |function_name: &str, args: &Value| {
            let function_name_owned = function_name.to_string();
            let args_owned = args.clone();
            
            let current_file_path = {
                let current_path = state.current_file_path.lock().unwrap();
                current_path.as_ref().map(|p| p.to_string())
            };
            
            Box::pin(async move {
                execute_excel_tool_with_path(&function_name_owned, &args_owned, current_file_path.as_deref()).await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("{}", e).into() })
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>
        };

        // Clone pdf_files to avoid move issues
        let pdf_files_clone = pdf_files.clone();
        
        // Use PDF-enabled method
        match client.send_message_with_pdf(&message, pdf_files_clone, tool_executor).await {
            Ok(response) => {
                // Convert tool calls to our format
                let tool_calls: Vec<ToolCall> = response.tool_calls.iter().map(|tc| ToolCall {
                    function_name: tc.function_name.clone(),
                    arguments: tc.arguments.clone(),
                    result: tc.result.clone(),
                }).collect();

                // Add user message to history
                {
                    let mut chat_history = state.chat_history.lock().unwrap();
                    chat_history.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!("{} [with {} PDF attachment(s)]", message, pdf_files.len()),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        tool_calls: None,
                        response_details: None,
                    });

                    // Add assistant response
                    chat_history.push(ChatMessage {
                        role: "model".to_string(),
                        content: response.content.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                        response_details: Some(ResponseDetails {
                            has_tool_calls: response.has_tool_calls,
                            iterations: response.iterations,
                        }),
                    });
                }

                // Update the client state
                {
                    let mut gemini_state = state.gemini_client.lock().unwrap();
                    *gemini_state = Some(client);
                }

                Ok(response.content)
            }
            Err(e) => Err(format!("Gemini PDF API error: {}", e))
        }
    } else {
        Err("Gemini client not configured. Please set up your API key first.".to_string())
    }
}

/// Select PDF files for attachment
#[tauri::command]
pub async fn select_pdf_files(app_handle: AppHandle) -> Result<Vec<String>, String> {
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    
    app_handle.dialog().file()
        .set_title("Select PDF files")
        .add_filter("PDF Files", &["pdf"])
        .pick_files(move |file_paths| {
            let _ = tx.send(file_paths);
        });

    let files = rx.recv()
        .map_err(|e| format!("Failed to receive file dialog result: {}", e))?;

    if let Some(file_paths) = files {
        let paths: Vec<String> = file_paths.iter()
            .map(|path| path.to_string())
            .collect();
        Ok(paths)
    } else {
        Ok(Vec::new())
    }
}

// Excel interaction commands using Terminator
#[tauri::command]
pub async fn excel_read_cell(
    cell_address: String,
) -> Result<String, String> {
    let automation = get_excel_automation().await
        .map_err(|e| format!("Excel automation error: {}", e))?;
    
    let cell = automation.read_cell(&cell_address).await
        .map_err(|e| format!("Error reading cell: {}", e))?;
    
    Ok(format!("Cell {}: '{}'", cell.address, cell.value))
}

#[tauri::command]
pub async fn excel_write_cell(
    cell_address: String,
    value: String,
) -> Result<String, String> {
    let automation = get_excel_automation().await
        .map_err(|e| format!("Excel automation error: {}", e))?;
    
    automation.write_cell(&cell_address, &value).await
        .map_err(|e| format!("Error writing to cell: {}", e))?;
    
    Ok(format!("Written '{}' to cell {}", value, cell_address))
}

#[tauri::command]
pub async fn excel_read_range(
    start_cell: String,
    end_cell: String,
) -> Result<Vec<Vec<String>>, String> {
    let automation = get_excel_automation().await
        .map_err(|e| format!("Excel automation error: {}", e))?;
    
    let range = automation.read_range(&start_cell, &end_cell).await
        .map_err(|e| format!("Error reading range: {}", e))?;
    
    Ok(range.values)
}

#[tauri::command]
pub async fn excel_set_formula(
    cell_address: String,
    formula: String,
) -> Result<String, String> {
    let automation = get_excel_automation().await
        .map_err(|e| format!("Excel automation error: {}", e))?;
    
    automation.set_formula(&cell_address, &formula).await
        .map_err(|e| format!("Error setting formula: {}", e))?;
    
    Ok(format!("Formula '{}' set in cell {}", formula, cell_address))
} 