import {
  DesktopUseClient,
  ApiError,
  BasicResponse,
  CommandOutputResponse,
  ElementResponse,
  ExploreResponse,
  ClickResponse,
  OcrResponse,
} from "desktop-use"; // Assuming SDK is installed as a package or adjust path: e.g., "../ts-sdk/src/index.js"
import { z } from "zod";

// --- Zod Schemas for Tool Parameters (used in index.ts) ---

export const FindWindowSchema = z.object({
  titleContains: z
    .string()
    .describe(
      "A substring of the window title to search for (case-insensitive)."
    ),
  timeoutMs: z
    .number()
    .optional()
    .describe("Optional timeout in milliseconds."),
});

export const LocatorSchema = z.object({
  selectorChain: z
    .array(z.string())
    .describe(
      "An array of selector strings to locate the element (e.g., ['window:My App', 'button:OK'])."
    ),
    timeoutMs: z
    .number()
    .optional()
    .describe("Optional timeout in milliseconds for the action.")
});

export const GetElementTextSchema = LocatorSchema.extend({
  maxDepth: z
    .number()
    .optional()
    .describe("Maximum depth to search for text within child elements."),
});

export const TypeIntoElementSchema = LocatorSchema.extend({
  textToType: z.string().describe("The text to type into the element."),
});

export const PressKeySchema = LocatorSchema.extend({
    key: z.string().describe("The key or key combination to press (e.g., 'Enter', 'Ctrl+A').")
});

export const RunCommandSchema = z.object({
  windowsCommand: z
    .string()
    .optional()
    .describe("The command to run on Windows."),
  unixCommand: z
    .string()
    .optional()
    .describe("The command to run on Linux/macOS."),
});

export const ExploreSchema = z.object({
    selectorChain: z
    .array(z.string())
    .optional()
    .describe("Optional selector chain to explore from a specific element. Explores screen if omitted.")
})

// New schema for captureScreen (takes no arguments)
export const CaptureScreenSchema = z.object({});

// --- Terminator Tools Class ---

export class TerminatorTools {
  private client: DesktopUseClient;

  constructor(baseUrl?: string) {
    try {
      this.client = new DesktopUseClient(baseUrl); // Uses default if undefined
      console.log(
        `[TerminatorTools] Initialized DesktopUseClient targeting ${
          baseUrl || "default URL"
        }.`
      );
    } catch (error: any) {
      console.error(
        `[TerminatorTools] Failed to initialize DesktopUseClient: ${error.message}`
      );
      throw error; // Re-throw to prevent server from starting incorrectly
    }
  }

  /** Helper to handle API errors */
  private handleApiError(error: unknown, operation: string): { error: string } {
    let errorMessage = `Terminator API Error during ${operation}: Unknown error`;
    if (error instanceof ApiError) {
      errorMessage = `Terminator API Error during ${operation}: ${error.message} (Status: ${error.status})`;
    } else if (error instanceof Error) {
      errorMessage = `Error during ${operation}: ${error.message}`;
    }
    console.error(`[TerminatorTools] ${errorMessage}`);
    return { error: errorMessage };
  }

  /** Finds a window and returns its details */
  async findWindow(
    args: z.infer<typeof FindWindowSchema>
  ): Promise<{ windowElement: ElementResponse } | { error: string }> {
    try {
      console.log(`[TerminatorTools] Finding window: titleContains="${args.titleContains}"`);
      const windowLocator = await this.client.findWindow(
        { titleContains: args.titleContains },
        { timeout: args.timeoutMs }
      );
      // We need the element details, not just the locator
      const windowElement = await windowLocator.first();
      console.log(`[TerminatorTools] Found window: Role=${windowElement.role}, Name=${windowElement.label}`);
      return { windowElement };
    } catch (error) {
      return this.handleApiError(error, `findWindow(titleContains=${args.titleContains})`);
    }
  }

   /** Creates a locator with optional timeout */
   private getLocator(selectorChain: string[], timeoutMs?: number | null) {
        let locator = this.client.locator(selectorChain[0]); // Start with the first selector
        for (let i = 1; i < selectorChain.length; i++) {
            locator = locator.locator(selectorChain[i]); // Chain subsequent selectors
        }
        if (timeoutMs != null) {
            locator = locator.timeout(timeoutMs); // Apply timeout if provided
        }
        return locator;
    }


  /** Gets text from an element */
  async getElementText(
    args: z.infer<typeof GetElementTextSchema>
  ): Promise<{ text: string } | { error: string }> {
    try {
       console.log(`[TerminatorTools] Getting text from: ${JSON.stringify(args.selectorChain)}`);
       const locator = this.getLocator(args.selectorChain, args.timeoutMs);
       const result = await locator.getText(args.maxDepth);
       console.log(`[TerminatorTools] Got text snippet: "${result.text.substring(0, 100)}..."`);
       return { text: result.text };
    } catch (error) {
         return this.handleApiError(error, `getElementText(${JSON.stringify(args.selectorChain)})`);
    }
  }

  /** Types text into an element */
  async typeIntoElement(
    args: z.infer<typeof TypeIntoElementSchema>
  ): Promise<BasicResponse | { error: string }> {
    try {
       console.log(`[TerminatorTools] Typing into: ${JSON.stringify(args.selectorChain)}`);
       const locator = this.getLocator(args.selectorChain, args.timeoutMs);
       const result = await locator.typeText(args.textToType);
       console.log(`[TerminatorTools] Typed text successfully.`);
       return result;
    } catch (error) {
        return this.handleApiError(error, `typeIntoElement(${JSON.stringify(args.selectorChain)})`);
    }
  }

  /** Clicks an element */
  async clickElement(
    args: z.infer<typeof LocatorSchema>
  ): Promise<ClickResponse | { error: string }> {
    try {
      console.log(`[TerminatorTools] Clicking element: ${JSON.stringify(args.selectorChain)}`);
      const locator = this.getLocator(args.selectorChain, args.timeoutMs);
      const result = await locator.click();
      console.log(`[TerminatorTools] Click successful: ${result.details}`);
      return result;
    } catch (error) {
       return this.handleApiError(error, `clickElement(${JSON.stringify(args.selectorChain)})`);
    }
  }

   /** Presses a key or key combination on an element */
    async pressKey(
        args: z.infer<typeof PressKeySchema>
    ): Promise<BasicResponse | { error: string }> {
        try {
            console.log(`[TerminatorTools] Pressing key "${args.key}" on: ${JSON.stringify(args.selectorChain)}`);
            const locator = this.getLocator(args.selectorChain, args.timeoutMs);
            const result = await locator.pressKey(args.key);
            console.log(`[TerminatorTools] Key press successful.`);
            return result;
        } catch (error) {
            return this.handleApiError(error, `pressKey(${JSON.stringify(args.selectorChain)}, key=${args.key})`);
        }
    }


  /** Runs a command */
  async runCommand(
    args: z.infer<typeof RunCommandSchema>
  ): Promise<CommandOutputResponse | { error: string }> {
    if (!args.windowsCommand && !args.unixCommand) {
      return { error: "At least one of windowsCommand or unixCommand must be provided." };
    }
    try {
      const commandDesc = args.windowsCommand ? `Win: ${args.windowsCommand}` : `Unix: ${args.unixCommand}`;
      console.log(`[TerminatorTools] Running command: ${commandDesc}`);
      const result = await this.client.runCommand({
        windowsCommand: args.windowsCommand,
        unixCommand: args.unixCommand,
      });
       console.log(`[TerminatorTools] Command finished. Exit code: ${result.exit_code}`);
      return result;
    } catch (error) {
       return this.handleApiError(error, `runCommand`);
    }
  }

   /** Explores UI elements */
    async explore(
        args: z.infer<typeof ExploreSchema>
    ): Promise<ExploreResponse | { error: string }> {
        try {
            const targetDesc = args.selectorChain ? `from ${JSON.stringify(args.selectorChain)}` : "screen";
            console.log(`[TerminatorTools] Exploring ${targetDesc}`);
            let result: ExploreResponse;
            if (args.selectorChain && args.selectorChain.length > 0) {
                const locator = this.getLocator(args.selectorChain); // Timeout handled within explore if needed
                 result = await locator.explore();
            } else {
                 result = await this.client.exploreScreen();
            }

            console.log(`[TerminatorTools] Exploration found ${result.children.length} children.`);
            // Limit the size of the response sent back to the model potentially
            // For now, return the full response
             return result;
        } catch (error) {
             const targetDesc = args.selectorChain ? `from ${JSON.stringify(args.selectorChain)}` : "screen";
             return this.handleApiError(error, `explore(${targetDesc})`);
        }
    }

    /** Activates the window associated with the element */
    async activateApp(
        args: z.infer<typeof LocatorSchema> // Use LocatorSchema (chain + timeout)
    ): Promise<BasicResponse | { error: string }> {
        try {
            console.log(`[TerminatorTools] Activating element/window: ${JSON.stringify(args.selectorChain)}`);
            // Construct the payload for the backend endpoint
            const payload = { 
                selector_chain: args.selectorChain, 
                timeout_ms: args.timeoutMs 
            };
            // Directly call the backend endpoint using the client's internal method
            const result = await this.client._makeRequest<BasicResponse>("/activate_app", payload);
            console.log(`[TerminatorTools] Activation successful: ${result.message}`);
            return result; // Return the BasicResponse from the backend
        } catch (error) {
            return this.handleApiError(error, `activateApp(${JSON.stringify(args.selectorChain)})`);
        }
    }

    /** Captures a screenshot of the primary monitor and performs OCR */
    async captureScreen(
        // No arguments needed based on CaptureScreenSchema
    ): Promise<{ text: string } | { error: string }> { // Expect OcrResponse format
        try {
            console.log(`[TerminatorTools] Capturing screen and performing OCR...`);
            // Directly call the updated /capture_screen endpoint which now does OCR
            const result: OcrResponse = await this.client._makeRequest<OcrResponse>("/capture_screen", {});
            console.log(`[TerminatorTools] Capture and OCR successful.`);
            return { text: result.text }; // Return the OCR text
        } catch (error) {
            return this.handleApiError(error, `captureScreen (with internal OCR)`);
        }
    }
}
