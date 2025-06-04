// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod excel;
mod excel_interaction;
mod gemini;
mod commands;
mod locale_utils;

use commands::{
    open_excel_file, create_new_excel, save_excel_file, get_excel_content,
    setup_gemini_client, chat_with_gemini, chat_with_gemini_pdf, get_chat_history, clear_chat_history,
    excel_read_cell, excel_write_cell, excel_read_range, excel_set_formula,
    select_pdf_files, get_locale_info,
    AppStateStruct,
    open_google_sheets,
    google_sheets_send_prompt,
    google_sheets_send_data,
    google_sheets_interact,
    check_google_sheets_availability
};

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppStateStruct::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // File operations
            open_excel_file,
            create_new_excel,
            save_excel_file,
            get_excel_content,
            select_pdf_files,
            
            // Gemini chat
            setup_gemini_client,
            chat_with_gemini,
            get_chat_history,
            clear_chat_history,
            
            // Gemini chat PDF
            chat_with_gemini_pdf,
            
            // Excel interaction via automation
            excel_read_cell,
            excel_write_cell,
            excel_read_range,
            excel_set_formula,
            
            // System info
            get_locale_info,
            
            // Add new Google Sheets commands
            open_google_sheets,
            google_sheets_send_prompt,
            google_sheets_send_data,
            google_sheets_interact,
            check_google_sheets_availability
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Application entry point
fn main() {
    run();
}
