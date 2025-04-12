// examples/ai-explorer/index.ts
import * as dotenv from 'dotenv';
import * as fs from 'fs/promises'; // Use promises for async file operations
import * as path from 'path';
import inquirer from 'inquirer';
// Vercel AI SDK imports
import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { generateText, tool, streamText, CoreMessage } from 'ai';
import { z } from 'zod';
import fetch from 'node-fetch'; // Need to install node-fetch: npm install node-fetch @types/node-fetch

// Terminator/DesktopUseClient import
// Import using the package name, assuming it's correctly linked via package.json
import { DesktopUseClient } from 'desktop-use';

// Suppress Node.js warnings (use with caution)
process.removeAllListeners('warning');

dotenv.config(); // Load .env file

const ENV_PATH = path.resolve(__dirname, '../.env'); // Path to .env in parent dir

async function getApiKey(): Promise<string> {
    let apiKey = process.env.GEMINI_API_KEY;

    if (!apiKey) {
        console.log("Gemini API Key not found in environment variables (.env).");
        const answers = await inquirer.prompt([
            {
                type: 'password', // Use password type to mask input
                name: 'apiKey',
                message: 'Please enter your Gemini API Key:',
                mask: '*',
                validate: (input: string) => !!input || 'API Key cannot be empty.',
            },
        ]);
        apiKey = answers.apiKey;

        // Save the API key to .env
        try {
            await fs.appendFile(ENV_PATH, `\nGEMINI_API_KEY=${apiKey}`);
            console.log("API Key saved to .env file.");
            // Reload dotenv to make the new key available immediately
            dotenv.config({ path: ENV_PATH, override: true });
        } catch (err) {
            console.error("Error saving API key to .env:", err);
            console.warn("Proceeding without saving API key.");
        }
    }

    if (!apiKey) { // Double check if it's still missing (e.g., save failed)
         throw new Error("Failed to get Gemini API Key.");
    }

    return apiKey;
}

async function main() {
    console.log("\n✨ Welcome to the AI Explorer for Terminator! ✨");

    const apiKey = await getApiKey();
    console.log("🔑 Gemini API Key loaded.");

    const google = createGoogleGenerativeAI({
        apiKey: apiKey,
    });
    const model = google('models/gemini-2.0-flash'); // Using 1.5 Pro for better tool use potentially
    console.log(`🤖 Initialized Gemini Model: ${model.modelId}`);

    let desktopClient: DesktopUseClient | null = null;
    try {
        desktopClient = new DesktopUseClient();
        console.log("🖥️ Connected to Terminator server.");
        // Quick check
        // const check = await desktopClient.locator("window:*").findElements();
        // console.log(`   Root element check successful.`);
    } catch (error) {
        console.error("❌ Failed to connect to Terminator server. Is it running?");
        console.error(error instanceof Error ? error.message : error);
        process.exit(1);
    }
    // Ensure desktopClient is not null for tool execution
    if (!desktopClient) {
        console.error("❌ Desktop client initialization failed unexpectedly.");
        process.exit(1);
    }

    const { automationGoal } = await inquirer.prompt([
        {
            type: 'input',
            name: 'automationGoal',
            message: 'What UI automation task would you like to generate code for?'
        }
    ]);

    console.log(`🎯 Your goal: ${automationGoal}`);
    console.log("\n🧠 Thinking and generating code...\n");

    // --- Define Tools ---
    const tools = {
        readDesktopUseDocs: tool({
            description: "Reads the plain text content from the Desktop-Use documentation. Useful for fetching documentation or web page content.",
            parameters: z.object({}),
            execute: async () => {
                try {
                    console.log(`\n🔧 [Tool Call] Reading content from: https://docs.screenpi.pe/terminator/js-sdk-reference`);
                    const response = await fetch("https://docs.screenpi.pe/terminator/js-sdk-reference");
                    if (!response.ok) {
                        throw new Error(`HTTP error! status: ${response.status}`);
                    }
                    const text = await response.text();
                    console.log(`\n✅ [Tool Result] Read content snippet.`);
                    return { success: true, content: text };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to read URL: ${error.message}`);
                    return { success: false, error: error.message };
                }
            },
        }),
        askUserTool: tool({
            description: "Asks the user a question and waits for their text response. Use this to clarify instructions, confirm actions, or get more details.",
            parameters: z.object({
                question: z.string().describe('The question to ask the user')
            }),
            execute: async ({ question }) => {
                console.log(`\n❓ [Tool Call] Asking user: ${question}`);
                const answers = await inquirer.prompt([
                    {
                        type: 'input',
                        name: 'userResponse',
                        message: question,
                    },
                ]);
                 console.log(`\n✅ [Tool Result] User responded.`);
                return { userResponse: answers.userResponse };
            },
        }),
        // --- Desktop Interaction Tools ---
        findElementTool: tool({
            description: "Finds a UI element using a selector string (e.g., 'window:My App', 'Name:OK Button', 'Id:some-id'). Returns element details if found, otherwise an error.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element (e.g., 'window:My App', 'Name:OK Button')")
            }),
            execute: async ({ selector }) => {
                try {
                    console.log(`\n🔧 [Tool Call] Finding element: "${selector}"`);
                    // Use the non-null asserted desktopClient
                    const element = await desktopClient!.locator(selector).first();
                     console.log(`\n✅ [Tool Result] Found element: ${JSON.stringify(element)}`);
                    return { success: true, element };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to find element "${selector}": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
        clickElementTool: tool({
            description: "Clicks the UI element identified by the selector string.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element to click")
            }),
            execute: async ({ selector }) => {
                try {
                    console.log(`\n🔧 [Tool Call] Clicking element: "${selector}"`);
                  
                    const element = await desktopClient!.locator(selector).activateApp();
                    const result = await element.click();
                    console.log(`\n✅ [Tool Result] Clicked element: ${JSON.stringify(result)}`);
                    return { success: true, details: result };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to click element "${selector}": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
        typeTextTool: tool({
            description: "Types the given text into the UI element identified by the selector string.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element to type into"),
                text: z.string().describe("The text to type")
            }),
            execute: async ({ selector, text }) => {
                 try {
                    console.log(`\n🔧 [Tool Call] Typing text "${text}" into element: "${selector}"`);
                    const element = await desktopClient!.locator(selector).activateApp();
                    const result = await element.typeText(text);
                    console.log(`\n✅ [Tool Result] Typed text.`);
                    return { success: true, details: result };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to type into element "${selector}": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
         getTextTool: tool({
            description: "Gets the text content from the UI element identified by the selector string.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element to get text from")
            }),
            execute: async ({ selector }) => {
                 try {
                    console.log(`\n🔧 [Tool Call] Getting text from element: "${selector}"`);
                    const element = await desktopClient!.locator(selector).activateApp();
                    const result = await element.getText();
                    console.log(`\n✅ [Tool Result] Got text: "${result.text}"`);
                    return { success: true, text: result.text };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to get text from element "${selector}": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
        openApplicationTool: tool({
            description: "Opens a desktop application by its name or path.",
            parameters: z.object({
                appName: z.string().describe("The name or path of the application to open")
            }),
             execute: async ({ appName }) => {
                 try {
                    console.log(`\n🔧 [Tool Call] Opening application: "${appName}"`);
                    const result = await desktopClient!.openApplication(appName);
                    console.log(`\n✅ [Tool Result] Opened application.`);
                    return { success: true, details: result };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to open application "${appName}": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
         openUrlTool: tool({
            description: "Opens a URL in the default browser or a specified browser.",
            parameters: z.object({
                url: z.string().describe("The URL to open"),
                browser: z.string().optional().describe("Optional: Specify browser name/path")
            }),
             execute: async ({ url, browser }) => {
                 try {
                    console.log(`\n🔧 [Tool Call] Opening URL: "${url}" ${browser ? `with browser: ${browser}` : ''}`);
                    const result = await desktopClient!.openUrl(url, browser);
                    console.log(`\n✅ [Tool Result] Opened URL.`);
                    return { success: true, details: result };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to open URL "${url}": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
        // --- Expectation Tools ---
        expectVisibleTool: tool({
            description: "Waits for the element specified by the selector to become visible within a timeout. Use this to ensure an element has appeared after an action.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element to check"),
                timeout: z.number().optional().describe("Optional timeout in milliseconds")
            }),
            execute: async ({ selector, timeout }) => {
                try {
                    console.log(`\n🔧 [Tool Call] Expecting element visible: \"${selector}\"` + (timeout ? ` (Timeout: ${timeout}ms)`: ''));
                    const element = await desktopClient!.locator(selector).expectVisible(timeout);
                    console.log(`\n✅ [Tool Result] Element \"${selector}\" is visible.`);
                    return { success: true, element }; // Return element details on success
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to expect element visible \"${selector}\": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
        expectEnabledTool: tool({
            description: "Waits for the element specified by the selector to become enabled within a timeout. Use this to ensure an element is interactive.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element to check"),
                timeout: z.number().optional().describe("Optional timeout in milliseconds")
            }),
            execute: async ({ selector, timeout }) => {
                try {
                    console.log(`\n🔧 [Tool Call] Expecting element enabled: \"${selector}\"` + (timeout ? ` (Timeout: ${timeout}ms)`: ''));
                    const element = await desktopClient!.locator(selector).expectEnabled(timeout);
                    console.log(`\n✅ [Tool Result] Element \"${selector}\" is enabled.`);
                    return { success: true, element };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to expect element enabled \"${selector}\": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
        expectTextEqualsTool: tool({
            description: "Waits for the text content of the element specified by the selector to exactly match the expected text within a timeout.",
            parameters: z.object({
                selector: z.string().describe("The selector string for the UI element whose text to check"),
                expectedText: z.string().describe("The exact text the element should contain"),
                maxDepth: z.number().optional().describe("Optional max depth for text search"),
                timeout: z.number().optional().describe("Optional timeout in milliseconds")
            }),
            execute: async ({ selector, expectedText, maxDepth, timeout }) => {
                try {
                    console.log(`\n🔧 [Tool Call] Expecting text in \"${selector}\" to equal \"${expectedText}\"` + (timeout ? ` (Timeout: ${timeout}ms)`: ''));
                    const element = await desktopClient!.locator(selector).expectTextEquals(expectedText, { maxDepth, timeout });
                    console.log(`\n✅ [Tool Result] Element \"${selector}\" text equals \"${expectedText}\".`);
                    return { success: true, element };
                } catch (error: any) {
                    console.error(`\n❌ [Tool Error] Failed to expect text equals for \"${selector}\": ${error.message}`);
                    return { success: false, error: error.message };
                }
            }
        }),
    };

    // --- Construct Prompt ---
    const messages: CoreMessage[] = [
        {
            role: 'system',
            content: `You are an expert AI assistant specializing in generating UI automation scripts using the 'desktop-use' TypeScript SDK.
            Your **primary goal** is to **produce a high-quality, executable TypeScript code script** using the 'desktop-use' SDK that fulfills the user's automation request. This script is the main deliverable for the user to integrate into their projects.
            You have access to tools to interact with the user's desktop environment, ask the user questions, and read documentation. Tool usage is a **means to generate better code**, not the end goal itself.

            **Desktop-Use SDK Context:**
            - **Client:** The main entry point is \`DesktopUseClient\`. Assume an instance named \`desktopClient\` is already initialized.
            - **Locator:** Use \`desktopClient.locator('selector')\` to create a \`Locator\` for an element or group of elements. Selectors target elements based on properties like window title, name, ID, or role.
            - **Chaining:** Locators can be chained to narrow down the search context, e.g., \`desktopClient.locator('window:My App').locator('Name:File')\`.
            - **Actions:** Locators have action methods like \`.click()\`, \`.typeText('some text')\`, \`.getText()\`, \`.pressKey('Enter')\`, \`.getAttributes()\`, \`.getBounds()\`, \`.findElement()\` (gets first), \`.findElements()\` (gets all).
            - **Expectations:** Locators have expectation methods (prefixed with \`expect\`) that wait for a condition, e.g., \`.expectVisible()\`, \`.expectEnabled()\`, \`.expectTextEquals('text')\`. These methods have optional timeout parameters.
            - **App/URL Control:** Use \`desktopClient.openApplication('appName')\` and \`desktopClient.openUrl('url')\` for top-level actions.
            - **Error Handling:** Actions might throw an \`ApiError\` if an element is not found or an action fails.
            - **Async:** All SDK methods interacting with the UI are asynchronous and should be awaited.

            **Core Behavior:**
            - **Code Generation is Paramount:** Your main focus is translating the user's request into a valid 'desktop-use' TypeScript script. Use the SDK context provided. Assume \`desktopClient\` is initialized in the generated code.
            - **Tools Support Code Generation:** Use inspection tools ('findElementTool', 'getTextTool') and expectation tools ('expectVisibleTool', etc.) primarily to **inform your code generation**. Verify selectors, understand UI state, and ensure the logic you write into the script will be robust. Use 'readUrlContentTool' for documentation if needed.
            - **Avoid Direct Execution (Default):** Do not use action tools ('clickElementTool', 'typeTextTool', etc.) **unless** the user explicitly asks you to perform a direct interaction *as part of the current request analysis* (e.g., "Click this button now and tell me what happens"). Your default is to generate code that performs the action, not perform the action yourself.
            - **Confirm Ambiguities & Final Code Plan:** If the request is ambiguous, or before generating the final, complete code script, use 'askUserTool' to confirm your understanding and the overall plan for the script. Avoid excessive confirmation for simple steps during analysis.
            - **Clarify When Needed:** Use 'askUserTool' if selectors are unclear or more information is needed to write the code.
            - **Final Output is Code:** The final output of your process should always be the complete TypeScript code block, ready for the user. Start it with necessary imports (if any beyond the client).

            **Guidelines:**
            - **Always Verify State Changes:** After performing an action that should open a new window/dialog or enable/reveal an element (e.g., clicking 'File' -> 'Save As...'), **you must** use an appropriate \`expect\` tool (like \`expectVisibleTool\`, \`expectEnabledTool\`) on an element within the *new* state *before* proceeding with further interactions in that new state. This confirms the action succeeded. Otherwise your clicks etc. might run but not do anything.
            - **Selector Strategy:** Use specific selectors (like 'window:Window Title', 'Name:Button Name', 'Id:some-id', 'role:button'). Be precise. Use inspection tools if unsure. If a selector fails (especially for dynamic content like results displays), try alternative selectors. Check for a specific Automation ID (\`Id:...\`) using inspection tools (like Accessibility Insights) as these are often more reliable.
            - **Structure:** Structure the generated code logically using async/await.
            - **Tip for User:** If you need help finding element details (like Name, Id, Role), consider using Accessibility Insights for Windows, available at https://accessibilityinsights.io/downloads/. It can inspect UI elements.

            Example Selector Syntax:
            - 'window:Calc' (Finds a window with title "Calc")
            - 'name:Open' (Finds an element with accessible name "Open")
            - 'id:username-input' (Finds an element by its automation ID)
            - 'role:button' (Finds elements with the role/type "button")
            - 'name:File; role:menu item' (Chained selector: find "File" menu item)

            Example Generation Flow (Conceptual):
            - User asks: "Generate code to click the OK button in the 'Save As' dialog."
            - You (optional): Use 'findElementTool' with selector 'window:Save As; Name:OK' to verify and get details.
            - Tool Result: Success/Failure/Details...
            - You: Generate final code based on findings: \`\`\`typescript
              // Example assuming element was found
              await desktopClient.locator('window:Save').click();
              \`\`\`
            `,
        },
        {
            role: 'user',
            content: `Generate the desktop-use TypeScript code for this task: ${automationGoal}`
        }
    ];

    // --- Generate and Stream Text ---
    try {
        const { textStream } = streamText({
            model: model,
            tools: tools,
            messages: messages,
            toolChoice: 'required',
            // maxToolSteps: 10, // Allow multiple tool calls if needed
             // You might want to increase maxSteps if complex interactions are needed
             maxSteps: 30,
            //  temperature: 0.3, // Lower temperature for more deterministic code generation
             onError: (error) => {
                console.error("Error:", error);
             },
            //  onFinish: (text) => {
            //     console.log("Finished", text);
            //  },
            //  onChunk: (chunk) => {
            //     console.log("Chunk:", chunk);
            //  }
        });

        // Stream the output to the console
        let fullResponse = "";
        // process.stdout.write("\n```typescript\n"); // Start markdown code block
        for await (const textPart of textStream) {
            process.stdout.write(textPart);
            fullResponse += textPart;
        }
        //  process.stdout.write("\n```\n"); // End markdown code block

        // Optional: You could further process the fullResponse here if needed
        console.log("\n\n✅ Generation complete.");

    } catch (error) {
        console.error("\n❌ An error occurred during text generation:");
        console.error(error);
    } finally {
         // Close the client if it was created - though maybe keep it open for iterative use?
         // For this script, let's assume we might want to run again, so we don't close yet.
         // await desktopClient?.close(); // If you had a close method
         console.log("\n👋 AI Explorer session finished.");
    }
}

main().catch(console.error); 