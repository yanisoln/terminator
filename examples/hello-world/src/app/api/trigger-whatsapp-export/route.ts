import { NextRequest, NextResponse } from "next/server";
import { DesktopUseClient, ApiError, sleep } from "desktop-use"; // Use the installed package name

// nodejs instead of edge runtime
export const runtime = "nodejs";

// --- Configuration --- //
// Adjust the WhatsApp application name if necessary (e.g., 'WhatsApp' on macOS, might differ on Windows)
const WHATSAPP_APP_NAME = "WhatsApp";

// --- Placeholder Locators --- //
// !! These are examples and WILL likely need adjustment for your OS and WhatsApp version !!
// Use accessibility inspection tools to find the correct locators.
const CHAT_WINDOW_LOCATOR = `window:${WHATSAPP_APP_NAME}`; // Might need a more specific chat window title
const MENU_BUTTON_LOCATOR = "button"; // Example: Find a button named 'Menu' or similar (e.g., three dots)
const MORE_MENU_ITEM_LOCATOR = "menuitem:Name:More"; // Example: Menu item named 'More'
const EXPORT_CHAT_MENU_ITEM_LOCATOR = "menuitem:Name:Export Chat"; // Example: Menu item named 'Export Chat'
const ATTACH_MEDIA_BUTTON_LOCATOR = "button:Name:Attach Media"; // Button to confirm export *with* media (for ZIP)
const SAVE_DIALOG_CONFIRM_KEY = "Enter"; // Key press to confirm the default save dialog

export async function POST(request: NextRequest) {
  console.log("api route /api/trigger-whatsapp-export called");
  const client = new DesktopUseClient();

  try {
    // 1. Ensure WhatsApp is open (or open it)
    // This part is optional if you assume it's already open.
    await client.openApplication(WHATSAPP_APP_NAME);
    await sleep(3000); // Wait for app to potentially open/focus

    // 2. Define the main chat window locator
    // const chatWindow = client.locator(CHAT_WINDOW_LOCATOR);
    // It's good practice to wait for the window to be visible/stable
    // await chatWindow.expectVisible(5000); // Wait up to 5 seconds

    // 3. Navigate the export menu
    //    (Sequence depends heavily on WhatsApp's current UI layout)
    console.log("attempting to click menu button...");
    const buttons = await client.locator(MENU_BUTTON_LOCATOR).all();
    console.log("buttons:", buttons);
    await sleep(500); // Wait for menu to appear

    // console.log("attempting to click more menu item...");
    // await chatWindow.locator(MORE_MENU_ITEM_LOCATOR).click();
    // await sleep(500); // Wait for submenu

    // console.log("attempting to click export chat menu item...");
    // await chatWindow.locator(EXPORT_CHAT_MENU_ITEM_LOCATOR).click();
    // await sleep(1000); // Wait for export options dialog

    // // 4. Choose 'Attach Media' for ZIP export
    // console.log("attempting to click attach media button...");
    // // This locator might be relative to a different dialog window that appeared
    // // If the above fails, try locating the button from the root client:
    // // await client.locator(ATTACH_MEDIA_BUTTON_LOCATOR).click();
    // await chatWindow.locator(ATTACH_MEDIA_BUTTON_LOCATOR).click();
    // await sleep(2000); // Wait for save dialog to appear

    // // 5. Confirm the Save Dialog (Simplistic Approach)
    // console.log("attempting to confirm save dialog...");
    // // This assumes the main chat window still has focus, which might be incorrect.
    // // The save dialog is a *new* window/element.
    // // A more robust way involves locating the save dialog and pressing Enter within it.
    // // Example (needs specific locator for the dialog):
    // // const saveDialog = client.locator('window:Name:Export Chat'); // Find save dialog window
    // // await saveDialog.pressKey(SAVE_DIALOG_CONFIRM_KEY);
    // await chatWindow.pressKey(SAVE_DIALOG_CONFIRM_KEY); // Simplistic: Send Enter to assumed active window

    console.log("export process likely initiated.");

    return NextResponse.json({
      message:
        "whatsapp export automation sequence initiated. check whatsapp and your default save location.",
    });
  } catch (error: any) {
    console.error("terminator automation error:", error);
    let errorMessage = "an unknown error occurred during automation.";
    if (error instanceof ApiError) {
      errorMessage = `terminator api error: ${error.message} (Status: ${
        error.status ?? "N/A"
      })`;
    } else if (error instanceof Error) {
      errorMessage = `automation script error: ${error.message}`;
    }
    return NextResponse.json({ message: errorMessage }, { status: 500 });
  }
}
