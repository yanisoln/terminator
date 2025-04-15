// examples/client_example.ts
// Import necessary components from the installed SDK
import { DesktopUseClient, ApiError, sleep } from '../ts-sdk/src/index'; // Adjust if using package name after build/install

// --- Example Usage using the SDK ---

// Use an async IIFE (Immediately Invoked Function Expression) to allow top-level await
(async () => {
    // Ensure the Terminator server (e.g., examples/server.rs) is running!
    const client = new DesktopUseClient();

    try {
        // 1. Open Calculator
        console.log("\n--- 1. Opening Application ---");
        // Adjust app name if necessary (e.g., 'Calculator' or 'calc' on Windows)
        await client.openApplication("Calc");
        await sleep(2000); // Allow app to open

        // 2. Create locators using chaining from the SDK
        console.log("\n--- 2. Defining Locators ---");
        const calculatorWindow = client.locator("window:Calculator"); // Adjust selector if window title is different
        // Locators relative to the calculator window
        // IMPORTANT: Selectors might differ significantly on non-Windows platforms or even Win versions
        const displayElement = calculatorWindow.locator("Id:CalculatorResults"); // Using AutomationId is often more stable
        const button1 = calculatorWindow.locator("Name:One");
        const buttonPlus = calculatorWindow.locator("Name:Plus");
        const button2 = calculatorWindow.locator("Name:Two");
        const buttonEquals = calculatorWindow.locator("Name:Equals");

        // 3. Get initial text
        console.log("\n--- 3. Getting Initial Text ---");
        try {
            // Wait for the display to be visible before getting text
            await displayElement.expectVisible(3000);
            const initialTextResponse = await displayElement.getText();
            console.log(`Initial display text: ${initialTextResponse?.text}`);
        } catch (e) {
            console.warn(`Could not get initial display text: ${e instanceof Error ? e.message : e}`);
        }

        // 4. Perform clicks (1 + 2 =)
        console.log("\n--- 4. Performing Clicks --- (1 + 2 =)");
        await button1.click();
        await sleep(500);
        await buttonPlus.click();
        await sleep(500);
        await button2.click();
        await sleep(500);
        await buttonEquals.click();
        await sleep(1000); // Wait for calculation

        // 5. Get final text & Verify using expect
        console.log("\n--- 5. Verifying Final Text --- (Expecting 3)");
        try {
             // Use expectTextEquals to wait for the result to be '3'
             // Note: Calculator display might show 'Display is 3' or just '3'. Adapt if needed.
             await displayElement.expectTextEquals("3", { timeout: 5000, maxDepth: 1 });
             console.log(`Final display text is verified to be '3'`);

             // Optionally, get the text again after verification
             const finalTextResponse = await displayElement.getText();
             console.log(`Final display text (raw): ${finalTextResponse?.text}`);

        } catch (e) {
            console.error(`Verification failed or could not get final text: ${e instanceof Error ? e.message : e}`);
            // Try getting raw text anyway on failure for debugging
            try {
                 const rawText = await displayElement.getText();
                 console.error(`Raw text on failure: ${rawText?.text}`);
            } catch (innerErr) {
                 console.error(`Could not even get raw text after verification failure: ${innerErr instanceof Error ? innerErr.message : innerErr}`);
            }
        }

        // Example: Get attributes of the equals button
        console.log("\n--- Example: Get Attributes of '=' button ---");
        const attrs = await buttonEquals.getAttributes();
        console.log(`Equals button attributes: ${JSON.stringify(attrs, null, 2)}`);

        // Example: Check visibility of the equals button
        console.log("\n--- Example: Check Visibility of '=' button ---");
        const visible = await buttonEquals.isVisible();
        console.log(`Is Equals button visible? ${visible}`);

        // Optional: Close the calculator
        // console.log("\n--- Optional: Closing Calculator ---");
        // try {
        //     await calculatorWindow.pressKey("%{F4}"); // Alt+F4 on Windows - Ensure key codes match server expectation
        //     console.log("Sent close command.");
        // } catch (e) {
        //      console.warn(`Could not send close command: ${e instanceof Error ? e.message : e}`);
        // }

    } catch (e) {
        if (e instanceof ApiError) {
             console.error(`\nAPI Error occurred: ${e.message} (Status: ${e.status ?? 'N/A'})`);
        } else if (e instanceof Error) {
            console.error(`\nAn unexpected error occurred: ${e.message}`);
            console.error(e.stack);
        } else {
             console.error(`\nAn unknown error occurred: ${e}`);
        }
        process.exit(1); // Exit with error code
    }

    console.log("\n--- Example Finished --- (Press Ctrl+C if server doesn't exit)");

})(); // End of async IIFE
