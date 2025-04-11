# terminator 🤖

https://github.com/user-attachments/assets/024c06fa-19f2-4fc9-b52d-329768ee52d0

https://github.com/user-attachments/assets/890d6842-782c-4b2b-8920-224bd63c4545

**terminator** is an AI-first cross-platform ui automation library for rust, designed to interact with native gui applications on windows and macos using a Playwright-like API.

it provides a unified api to find and control ui elements like buttons, text fields, windows, and more, enabling the creation of automation scripts, testing tools, and assistive technologies.

because it's using OS level APIs, it is 100x faster and more reliable for AI computer use than OpenAI Operator, Anthropic Computer Use, and acting at a lower level than a browser it can easily drop-in replace BrowserUse, BrowserBase, Playwright, etc.

## features

*   **cross-platform:** experimentally supports **windows** and **macos** with a consistent api.
*   **application control:** find running applications (by name, pid), open new applications, open urls.
*   **element discovery:** locate ui elements using various strategies (`Selector` enum):
    *   `Role` (e.g., "button", "textfield", "axbutton")
    *   `Name` (accessibility label/title)
    *   `Value`
    *   `Description`
    *   `AutomationId` (windows)
    *   `Id` (internal unique id)
    *   Text content (partial matches)
*   **element interaction:**
    *   click, double-click, right-click
    *   type text, set value, press keys (including modifiers like {enter}, {cmd}, {alt})
    *   focus, hover
    *   scroll (up, down, left, right)
    *   retrieve text content, attributes (value, role, description, bounds, properties)
    *   check state (enabled, visible, focused)
*   **element hierarchy:** traverse the accessibility tree (children, parent).
*   **debug support:** optional verbose logging and accessibility mode for easier debugging.

## platform support

*   ✅ windows (using ui automation)
*   ✅ macos (using accessibility api)
*   🐧 linux (planned/experimental - *adjust as needed*)

## installation

```
# download the repo
git clone https://github.com/mediar-ai/terminator
# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## quick start

this example opens notepad on macos (or notepad on windows), types text, and retrieves it.

```rust
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
```

*(remember to handle potential errors more robustly in real applications)*

## examples

more detailed examples can be found in the [`examples/`](./examples) directory:

*   `win_automation.rs`: demonstrates various windows-specific interactions.
*   `windows_pdf_to_legacy.rs`: simulates filling a legacy windows form.

run an example using:

```bash
cargo run --example <example_name> # e.g., cargo run --example mac_automation
```

## tests

unit and integration tests are located in `src/tests.rs`. run them with:

```bash
cargo test
```

*(note: some tests interact with live ui elements and might require specific applications to be running or be ignored (`#[ignore]`) by default).*

## todos

- [ ] JS SDK
- [ ] Python SDK
- [ ] switch to `cidre` on macos
- [ ] optional support for [screenshots](https://github.com/nashaofu/xcap), [OCR](https://github.com/mediar-ai/uniOCR), & example vision usage on top of low level usage
- [ ] Linux support?
- [ ] More to come...

## contributing

contributions are welcome! please feel free to submit issues and pull requests.

many stuff are experimental, we'd love any help in fixing stuff!

we use this on windows: https://github.com/leexgone/uiautomation-rs  
and we'd like to switch to this on macos: https://github.com/yury/cidre/blob/main/cidre/examples/ax-tree/main.rs

*(add contribution guidelines if you have specific requirements)*
