# terminator 🤖

high level demo:

https://github.com/user-attachments/assets/024c06fa-19f2-4fc9-b52d-329768ee52d0

dev demo 1:

https://github.com/user-attachments/assets/890d6842-782c-4b2b-8920-224bd63c4545

dev demo 2:

https://github.com/user-attachments/assets/c9f472f7-79ed-49c6-a4d0-93608fa1ce55

**terminator** is an AI-first cross-platform ui automation library for rust, designed to interact with native gui applications on windows and macos using a Playwright-like API.

it provides a unified api to find and control ui elements like buttons, text fields, windows, and more, enabling the creation of automation scripts, testing tools, and assistive technologies.

because it's using OS level APIs, it is 100x faster and more reliable for AI computer use than OpenAI Operator, Anthropic Computer Use, and acting at a lower level than a browser it can easily drop-in replace BrowserUse, BrowserBase, Playwright, etc.

> **Note:** While we support MacOS and Windows, we are focusing right now on Windows so if you are on MacOS you'll have to look into the code and figure out yourself.

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

this example opens calc on windows, types buttons, and get the result.

```bash
cargo run --example server -- --debug
```

in another terminal:

```bash
python3 .\examples\client_example.py
```

make sure to have python installed

## next steps & potential experiments

if you want to understand better how your desktop data is structured on windows, [check out this app](https://accessibilityinsights.io/downloads/)

now that you have a grasp of the basics, here are some exciting avenues to explore with **terminator**:

*   **ai-driven automation loop:**
    *   imagine an ai agent observing an application's state using terminator's element discovery features.
    *   the agent could then generate python (or eventually javascript) code on the fly using the `terminator` sdk to interact with the ui.
    *   this code gets executed, the agent observes the result, and the loop continues, enabling complex, adaptive automation. this could automate tedious data entry, testing, or even user support tasks.

*   **pdf to legacy form filler:**
    *   extract data from a pdf document.
    *   leverage **terminator** to navigate and fill this data into a legacy windows application (or any gui app) that lacks a modern api. this bridges the gap between old and new systems.

*   **javascript/typescript integration (future):**
    *   once the js sdk is ready, explore integrating **terminator** into web-based tools or electron apps. imagine browser extensions that can interact with native desktop applications or node.js scripts automating cross-application workflows.

we encourage you to experiment! how can **terminator** automate *your* specific workflows? what unique integrations can you build? share your ideas and contributions!

## todos

- [ ] JS SDK
- [x] Python SDK
- [ ] switch to `cidre` on macos
- [ ] optional support for [screenshots](https://github.com/nashaofu/xcap), [OCR](https://github.com/mediar-ai/uniOCR), & example vision usage on top of low level usage
- [ ] More to come...

## contributing

contributions are welcome! please feel free to submit issues and pull requests.

many stuff are experimental, we'd love any help in fixing stuff!

we use this on windows: https://github.com/leexgone/uiautomation-rs  
and we'd like to switch to this on macos: https://github.com/yury/cidre/blob/main/cidre/examples/ax-tree/main.rs

*(add contribution guidelines if you have specific requirements)*
