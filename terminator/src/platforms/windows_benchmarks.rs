#[cfg(test)]
mod performance_benchmarks {
    use super::super::windows::WindowsEngine;
    use crate::platforms::AccessibilityEngine;
    use std::time::{Duration, Instant};
    use std::process::Command;
    
    /// Comprehensive benchmark test for window tree building performance
    /// Tests both browser pages and system applications to measure:
    /// - Execution speed
    /// - Memory usage during operation  
    /// - Tree complexity (elements processed)
    /// Goal: Validate high-frequency operation on low-end machines
    #[tokio::test]
    async fn benchmark_tree_building_performance() {
        println!("🚀 Starting Window Tree Building Performance Benchmark");
        println!("============================================================");
        
        let engine = WindowsEngine::new(false, false).expect("Failed to create Windows engine");
        
        // Enhanced test scenarios: browser pages, complex websites, and heavy desktop apps
        let test_scenarios = vec![
            // Light websites for baseline
            ("Browser: Luma Event Page", "https://lu.ma/airstreet", "browser"),
            ("Browser: Dataiku AI Guide", "https://pages.dataiku.com/guide-to-ai-agents", "browser"), 
            
            // Heavy websites with lots of elements
            ("Browser: GitHub Trending", "https://github.com/trending", "browser"),
            ("Browser: Reddit Front Page", "https://reddit.com", "browser"),
            ("Browser: Amazon Homepage", "https://amazon.com", "browser"),
            ("Browser: YouTube Homepage", "https://youtube.com", "browser"),
            ("Browser: Twitter/X Homepage", "https://x.com", "browser"),
            ("Browser: LinkedIn Feed", "https://linkedin.com/feed", "browser"),
            ("Browser: Stack Overflow", "https://stackoverflow.com", "browser"),
            ("Browser: Wikipedia Main Page", "https://en.wikipedia.org/wiki/Main_Page", "browser"),
            
            // Complex web applications
            ("Browser: Figma Community", "https://figma.com/community", "browser"),
            ("Browser: Notion Homepage", "https://notion.so", "browser"),
            ("Browser: Slack Web App", "https://slack.com/signin", "browser"),
            
            // System applications (light)
            ("System: Calculator", "calc", "system"),
            ("System: Notepad", "notepad", "system"),
            ("System: Paint", "mspaint", "system"),
            
            // Heavy desktop applications
            ("System: File Explorer", "explorer", "system"),
            ("System: Task Manager", "taskmgr", "system"),
            ("System: Control Panel", "control", "system"),
            ("System: Windows Settings", "ms-settings:", "system"),
            ("System: Device Manager", "devmgmt.msc", "system"),
            ("System: Registry Editor", "regedit", "system"),
            
            // Microsoft Office suite (if available)
            ("System: Microsoft Word", "winword", "system"),
            ("System: Microsoft Excel", "excel", "system"),
            ("System: Microsoft PowerPoint", "powerpnt", "system"),
            
            // Development tools (if available)
            ("System: Visual Studio Code", "code", "system"),
            ("System: Windows Terminal", "wt", "system"),
        ];
        
        for (name, target, app_type) in test_scenarios {
            println!("\n📊 Testing: {}", name);
            println!("----------------------------------------");
            
            // Launch application/open URL with improved browser handling
            let app_element = match app_type {
                "browser" => {
                    // Try different browsers in order of preference
                    let browsers = vec![
                        ("Chrome", "chrome"),
                        ("Edge", "msedge"), 
                        ("Firefox", "firefox"),
                    ];
                    
                    let mut last_error = None;
                    let mut opened_element = None;
                    
                    for (browser_name, browser_cmd) in browsers {
                        match engine.open_url(target, Some(browser_cmd)) {
                            Ok(element) => {
                                println!("  ✅ Opened in {}", browser_name);
                                opened_element = Some(element);
                                break;
                            },
                            Err(e) => {
                                println!("  ⚠️ {} not available: {}", browser_name, e);
                                last_error = Some(e);
                            }
                        }
                    }
                    
                    match opened_element {
                        Some(element) => {
                            tokio::time::sleep(Duration::from_secs(5)).await; // Longer wait for heavy pages
                            element
                        },
                        None => {
                            println!("❌ Failed to open {} in any browser: {:?}", target, last_error);
                            continue;
                        }
                    }
                },
                "system" => {
                    match engine.open_application(target) {
                        Ok(element) => {
                            tokio::time::sleep(Duration::from_millis(1000)).await; // Wait for app startup
                            element
                        },
                        Err(e) => {
                            println!("❌ Failed to open {}: {} (app may not be installed)", target, e);
                            continue;
                        }
                    }
                },
                _ => continue,
            };
            
            // Get window title for tree building
            let window_title = match app_element.attributes().name {
                Some(title) => title,
                None => {
                    println!("❌ Could not get window title");
                    // Try to close the app before continuing
                    let _ = app_element.close();
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };
            
            // Get memory usage before operation
            let memory_before = get_process_memory_mb();
            
            // Benchmark tree building with multiple runs for accuracy
            let mut durations = Vec::new();
            let mut elements_processed = Vec::new();
            
            const BENCHMARK_RUNS: usize = 3;
            
            for run in 1..=BENCHMARK_RUNS {
                print!("  Run {}/{}: ", run, BENCHMARK_RUNS);
                
                let start_time = Instant::now();
                
                match engine.get_window_tree_by_title(&window_title) {
                    Ok(tree) => {
                        let duration = start_time.elapsed();
                        let element_count = count_tree_elements(&tree);
                        
                        durations.push(duration);
                        elements_processed.push(element_count);
                        
                        println!("✅ {}ms ({} elements)", duration.as_millis(), element_count);
                    },
                    Err(e) => {
                        println!("❌ Failed: {}", e);
                        continue;
                    }
                }
                
                // Small delay between runs to reduce system load
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            
            // Get memory usage after operation
            let memory_after = get_process_memory_mb();
            let memory_delta = memory_after.saturating_sub(memory_before);
            
            // Calculate and display metrics
            if !durations.is_empty() {
                let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
                let min_duration = durations.iter().min().unwrap();
                let max_duration = durations.iter().max().unwrap();
                let avg_elements = elements_processed.iter().sum::<usize>() / elements_processed.len();
                
                println!("\n  📈 Performance Metrics:");
                println!("     Average Time: {}ms", avg_duration.as_millis());
                println!("     Range: {}ms - {}ms", min_duration.as_millis(), max_duration.as_millis());
                println!("     Elements Processed: {} (avg)", avg_elements);
                println!("     Memory Delta: {}MB", memory_delta);
                println!("     Throughput: {:.1} elements/ms", avg_elements as f64 / avg_duration.as_millis() as f64);
                
                // Enhanced performance assessment for different complexity levels
                let performance_rating = assess_performance_enhanced(avg_duration, avg_elements, memory_delta, app_type);
                println!("     Performance Rating: {}", performance_rating);
                
                // Add complexity assessment
                let complexity_rating = assess_complexity(avg_elements);
                println!("     Complexity Level: {}", complexity_rating);
            }
            
            // Enhanced close handling with retry logic
            println!("  🔄 Closing application...");
            let close_result = close_application_with_retry(&app_element, app_type).await;
            match close_result {
                Ok(method) => println!("  ✅ Closed successfully using: {}", method),
                Err(e) => println!("  ⚠️ Close failed: {}", e),
            }
            
            tokio::time::sleep(Duration::from_millis(1000)).await; // Longer cleanup wait
        }
        
        println!("\n============================================================");
        println!("🎯 Benchmark completed!");
        println!("📊 Performance ratings help assess suitability for high-frequency automation");
        println!("🔍 Complexity levels indicate UI tree depth and element density");
    }
    
    /// Enhanced close handling with retry logic for different application types
    async fn close_application_with_retry(app_element: &crate::UIElement, app_type: &str) -> Result<String, String> {
        // Try the built-in close method first
        match app_element.close() {
            Ok(_) => return Ok("UIElement.close()".to_string()),
            Err(e) => println!("    ⚠️ UIElement.close() failed: {}", e),
        }
        
        // For browsers, try specific close methods
        if app_type == "browser" {
            // Try Ctrl+W (close tab) first
            if let Ok(_) = app_element.press_key("ctrl+w") {
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok("Ctrl+W".to_string());
            }
            
            // Try Alt+F4 (close window)
            if let Ok(_) = app_element.press_key("alt+f4") {
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok("Alt+F4".to_string());
            }
        }
        
        // For system apps, try Alt+F4
        if app_type == "system" {
            if let Ok(_) = app_element.press_key("alt+f4") {
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok("Alt+F4".to_string());
            }
            
            // Try Escape for some system dialogs
            if let Ok(_) = app_element.press_key("escape") {
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok("Escape".to_string());
            }
        }
        
        // Last resort: try to kill the process (be careful with this)
        Err("All close methods failed".to_string())
    }
    
    /// Count total elements in a UI tree recursively
    fn count_tree_elements(node: &crate::UINode) -> usize {
        1 + node.children.iter().map(count_tree_elements).sum::<usize>()
    }
    
    /// Get current process memory usage in MB
    fn get_process_memory_mb() -> u64 {
        let output = Command::new("powershell")
            .args(["-Command", "Get-Process -Id $PID | Select-Object -ExpandProperty WorkingSet64"])
            .output();
            
        match output {
            Ok(output) => {
                let memory_str = String::from_utf8_lossy(&output.stdout);
                memory_str.trim().parse::<u64>().unwrap_or(0) / 1024 / 1024
            },
            Err(_) => 0,
        }
    }
    
    /// Enhanced performance assessment with different criteria for different app types
    fn assess_performance_enhanced(duration: Duration, elements: usize, memory_mb: u64, app_type: &str) -> &'static str {
        let ms = duration.as_millis();
        
        // Different thresholds based on application type and complexity
        match app_type {
            "browser" => {
                // Browsers are expected to be slower due to complex web content
                match (ms, elements, memory_mb) {
                    (0..=100, 0..=100, 0..=15) => "🟢 Excellent - Fast even for web content",
                    (101..=300, 0..=300, 0..=25) => "🟡 Good - Acceptable for web automation", 
                    (301..=600, 0..=500, 0..=50) => "🟠 Fair - May struggle with complex sites",
                    (601..=1200, 0..=1000, 0..=100) => "🔴 Poor - Too slow for frequent web automation",
                    _ => "🔴 Very Poor - Unsuitable for web automation",
                }
            },
            "system" => {
                // System apps should be faster
                match (ms, elements, memory_mb) {
                    (0..=50, 0..=50, 0..=10) => "🟢 Excellent - Perfect for high-frequency system automation",
                    (51..=150, 0..=150, 0..=20) => "🟡 Good - Suitable for system automation", 
                    (151..=300, 0..=300, 0..=35) => "🟠 Fair - May struggle with frequent system calls",
                    (301..=600, 0..=600, 0..=75) => "🔴 Poor - Too slow for system automation",
                    _ => "🔴 Very Poor - Unsuitable for system automation",
                }
            },
            _ => {
                // Generic assessment
                match (ms, memory_mb) {
                    (0..=100, 0..=15) => "🟢 Excellent",
                    (101..=250, 0..=30) => "🟡 Good", 
                    (251..=500, 0..=60) => "🟠 Fair",
                    (501..=1000, 0..=120) => "🔴 Poor",
                    _ => "🔴 Very Poor",
                }
            }
        }
    }
    
    /// Assess UI complexity based on element count
    fn assess_complexity(elements: usize) -> &'static str {
        match elements {
            0..=25 => "🟢 Simple - Basic UI with few elements",
            26..=100 => "🟡 Moderate - Standard application complexity",
            101..=300 => "🟠 Complex - Rich interface with many elements", 
            301..=600 => "🔴 Heavy - Dense UI requiring careful optimization",
            _ => "🟣 Extreme - Very complex interface, high processing cost",
        }
    }
} 