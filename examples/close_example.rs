use terminator::Desktop;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Demonstrating the close() functionality");
    
    // Create desktop instance
    let desktop = Desktop::new(false, false)?;
    
    // Open Calculator for demonstration
    println!("📱 Opening Calculator...");
    let calculator = desktop.open_application("calc")?;
    
    // Wait a moment for it to open
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    println!("✅ Calculator opened successfully!");
    println!("🔧 Window title: {}", calculator.window_title());
    
    // Demonstrate that close() can be called on any element
    println!("🧪 Testing close() on different element types:");
    
    // Try to find some elements and test close on them
    if let Ok(children) = calculator.children() {
        for (i, child) in children.iter().take(3).enumerate() {
            println!("  📦 Element {}: {} ({})", i + 1, child.role(), child.name_or("unnamed"));
            
            // Try closing - should do nothing for buttons/text but work for windows
            match child.close() {
                Ok(_) => println!("    ✅ close() succeeded (probably did nothing for non-closable element)"),
                Err(e) => println!("    ❌ close() failed: {}", e),
            }
        }
    }
    
    // Now close the calculator window itself
    println!("🗑️  Closing Calculator window...");
    match calculator.close() {
        Ok(_) => println!("✅ Calculator closed successfully!"),
        Err(e) => println!("❌ Failed to close Calculator: {}", e),
    }
    
    // Wait a moment to see the effect
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    println!("🎯 Demo completed! The close() method:");
    println!("   - Closes windows and applications when called on them");
    println!("   - Does nothing safely when called on buttons, text, etc.");
    println!("   - Uses native Windows patterns (WindowPattern, Alt+F4) as fallbacks");
    
    Ok(())
} 