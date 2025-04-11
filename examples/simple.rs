use std::{thread, time::Duration};
use terminator::{platforms, AutomationError}; // Ensure terminator is in scope

fn main() -> Result<(), AutomationError> {
    println!("starting simple automation...");

    // initialize the engine (false, false disables debug/accessibility mode)
    let engine = platforms::create_engine(false, false)?;
    println!("engine created.");

    // --- target application ---
    #[cfg(target_os = "macos")]
    let app_name = "TextEdit";
    #[cfg(target_os = "windows")]
    let app_name = "Notepad";

    println!("targeting application: {}", app_name);

    // try to get or open the application
    let app = match engine.get_application_by_name(app_name) {
        Ok(app) => {
            println!("{} already running, focusing.", app_name);
            app.focus()?;
            app
        }
        Err(_) => {
            println!("{} not running, attempting to open.", app_name);
            engine.open_application(app_name)?
        }
    };

    println!("{} should be open/focused.", app_name);
    thread::sleep(Duration::from_secs(2)); // wait for app


    app.type_text("Hello")?;
    println!("text typed.");
    thread::sleep(Duration::from_millis(500));

    println!("automation example finished.");
    Ok(())
}