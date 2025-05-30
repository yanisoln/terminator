// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// This lib.rs is auto-generated by Tauri for library builds
// We redirect to the main module where our actual application logic is

mod excel;
mod excel_interaction;
mod gemini;
mod commands;

use commands::{
    open_excel_file, create_new_excel, save_excel_file, get_excel_content,
    setup_gemini_client, chat_with_gemini, chat_with_gemini_pdf, get_chat_history, clear_chat_history,
    excel_read_cell, excel_write_cell, excel_read_range, excel_set_formula,
    select_pdf_files,
    AppStateStruct
};

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppStateStruct::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_excel_file,
            create_new_excel,
            save_excel_file,
            get_excel_content,
            select_pdf_files,
            
            setup_gemini_client,
            chat_with_gemini,
            get_chat_history,
            clear_chat_history,
            
            chat_with_gemini_pdf,
            
            excel_read_cell,
            excel_write_cell,
            excel_read_range,
            excel_set_formula
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
