// examples/client_example.ts
import * as http from 'http';
import * as querystring from 'querystring'; // For simple object stringification if needed (though JSON is primary)
import { TextDecoder } from 'util'; // Needed for decoding response body

const BASE_URL = "127.0.0.1:3000";
const [HOST, PORT_STR] = BASE_URL.split(':');
const PORT = parseInt(PORT_STR, 10);

// --- Interfaces for API Payloads and Responses ---

interface BasicResponse {
    message: string;
}

interface ElementResponse {
    role: string;
    label?: string | null;
    id?: string | null;
    // No handle needed in this version
}

interface ElementsResponse {
    elements: ElementResponse[];
}

interface ClickResponse {
    method: string;
    coordinates?: [number, number] | null;
    details: string;
}

interface TextResponse {
    text: string;
}

interface AttributesResponse {
    role: string;
    label?: string | null;
    value?: string | null;
    description?: string | null;
    properties: { [key: string]: any | null }; // Using 'any' for simplicity with serde_json::Value
    id?: string | null;
}

interface BoundsResponse {
    x: number;
    y: number;
    width: number;
    height: number;
}

interface BooleanResponse {
    result: boolean;
}

// Request Payloads
interface ChainedRequest {
    selector_chain: string[];
}

interface TypeTextRequest {
    selector_chain: string[];
    text: string;
}

interface GetTextRequest {
    selector_chain: string[];
    max_depth?: number | null;
}

interface PressKeyRequest {
    selector_chain: string[];
    key: string;
}

interface OpenApplicationRequest {
    app_name: string;
}

interface OpenUrlRequest {
    url: string;
    browser?: string | null;
}


// --- Custom Error ---

class ApiError extends Error {
    status?: number;
    constructor(message: string, status?: number) {
        super(message);
        this.name = "ApiError";
        this.status = status;
    }
}

// --- Client and Locator Classes ---

class TerminatorClient {
    private host: string;
    private port: number;

    constructor(baseUrl: string = BASE_URL) {
        const [host, portStr] = baseUrl.split(':');
        this.host = host;
        this.port = parseInt(portStr, 10);
    }

    private _makeRequest<T>(endpoint: string, payload: object): Promise<T> {
        return new Promise((resolve, reject) => {
            const jsonPayload = JSON.stringify(payload);
            console.log(`Sending POST to ${endpoint} with payload: ${jsonPayload}`);

            const options: http.RequestOptions = {
                hostname: this.host,
                port: this.port,
                path: endpoint,
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(jsonPayload),
                },
            };

            const req = http.request(options, (res) => {
                let data = '';
                const decoder = new TextDecoder('utf-8');

                res.on('data', (chunk) => {
                    data += decoder.decode(chunk, { stream: true });
                });

                res.on('end', () => {
                    data += decoder.decode(); // Flush any remaining bytes
                    console.log(`Response Status: ${res.statusCode}`);
                    console.log(`Response Data: ${data}`);

                    if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
                        try {
                           if (!data) {
                               // Handle empty successful responses (like open_app/url)
                               resolve({ message: "Operation successful (no content)" } as any);
                           } else {
                               resolve(JSON.parse(data) as T);
                           }
                        } catch (e) {
                           // Handle successful status but non-JSON or empty body for basic messages
                           if (data.includes("opened") || data.includes("successfully")) {
                                resolve({ message: data } as any); // Wrap basic message
                           } else {
                               console.error("Error: Could not decode JSON response despite success status.", e);
                               reject(new ApiError("Invalid JSON response from server", res.statusCode));
                           }
                        }
                    } else {
                        let errorMessage = `Server returned status ${res.statusCode}`;
                        try {
                            const errorData = JSON.parse(data);
                            errorMessage = errorData?.message || errorMessage;
                        } catch (e) {
                            // Ignore JSON parse error for error message
                        }
                        reject(new ApiError(errorMessage, res.statusCode));
                    }
                });
            });

            req.on('error', (e) => {
                console.error(`An error occurred during the request: ${e.message}`);
                if ((e as NodeJS.ErrnoException).code === 'ECONNREFUSED') {
                     reject(new ApiError(`Connection refused to ${this.host}:${this.port}`));
                } else {
                     reject(new ApiError(`Request error: ${e.message}`));
                }
            });

            // Write data to request body
            req.write(jsonPayload);
            req.end();
        });
    }

    locator(selector: string): Locator {
        return new Locator(this, [selector]);
    }

    async openApplication(appName: string): Promise<BasicResponse> {
        const payload: OpenApplicationRequest = { app_name: appName };
        // Type assertion needed because _makeRequest is generic
        return await this._makeRequest<BasicResponse>("/open_application", payload);
    }

    async openUrl(url: string, browser?: string | null): Promise<BasicResponse> {
        const payload: OpenUrlRequest = { url, browser };
        return await this._makeRequest<BasicResponse>("/open_url", payload);
    }

    // Expose _makeRequest for Locator class (alternative: pass client instance)
    // Making it public for simplicity here, could be protected/internal pattern
    public __internalMakeRequest<T>(endpoint: string, payload: object): Promise<T> {
         return this._makeRequest(endpoint, payload);
    }
}

class Locator {
    // Use public readonly properties for simplicity
    public readonly _client: TerminatorClient;
    public readonly _selector_chain: string[];

    constructor(client: TerminatorClient, selectorChain: string[]) {
        this._client = client;
        this._selector_chain = selectorChain;
    }

    locator(selector: string): Locator {
        const newChain = [...this._selector_chain, selector];
        return new Locator(this._client, newChain);
    }

    // --- Action Methods ---

    async findElement(): Promise<ElementResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client.__internalMakeRequest<ElementResponse>("/find_element", payload);
    }

    async findElements(): Promise<ElementsResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client.__internalMakeRequest<ElementsResponse>("/find_elements", payload);
    }

    async click(): Promise<ClickResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client.__internalMakeRequest<ClickResponse>("/click", payload);
    }

    async typeText(text: string): Promise<BasicResponse> {
        const payload: TypeTextRequest = { selector_chain: this._selector_chain, text };
        return await this._client.__internalMakeRequest<BasicResponse>("/type_text", payload);
    }

    async getText(maxDepth?: number | null): Promise<TextResponse> {
        const payload: GetTextRequest = { selector_chain: this._selector_chain, max_depth: maxDepth };
        return await this._client.__internalMakeRequest<TextResponse>("/get_text", payload);
    }

    async getAttributes(): Promise<AttributesResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client.__internalMakeRequest<AttributesResponse>("/get_attributes", payload);
    }

    async getBounds(): Promise<BoundsResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client.__internalMakeRequest<BoundsResponse>("/get_bounds", payload);
    }

    async isVisible(): Promise<boolean> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        const response = await this._client.__internalMakeRequest<BooleanResponse>("/is_visible", payload);
        return response?.result ?? false; // Safely access result
    }

    async pressKey(key: string): Promise<BasicResponse> {
        const payload: PressKeyRequest = { selector_chain: this._selector_chain, key };
        return await this._client.__internalMakeRequest<BasicResponse>("/press_key", payload);
    }
}

// --- Utility Functions ---
function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}


// --- Example Usage using the SDK ---

// Use an async IIFE (Immediately Invoked Function Expression) to allow top-level await
(async () => {
    const client = new TerminatorClient();

    try {
        // 1. Open Calculator
        console.log("\n--- 1. Opening Application ---");
        // Adjust app name if necessary (e.g., 'Calculator' on Windows, maybe different on macOS/Linux)
        await client.openApplication("Calc");
        await sleep(2000); // Allow app to open

        // 2. Create locators using chaining
        console.log("\n--- 2. Defining Locators ---");
        const calculatorWindow = client.locator("window:Calc"); // Adjust selector if needed
        // Locators relative to the calculator window
        // IMPORTANT: Selectors might differ significantly on non-Windows platforms or even Win versions
        const displayElement = calculatorWindow.locator("Name:Display is 0");
        const button1 = calculatorWindow.locator("Name:One");
        const buttonPlus = calculatorWindow.locator("Name:Plus");
        const button2 = calculatorWindow.locator("Name:Two");
        const buttonEquals = calculatorWindow.locator("Name:Equals");
        // Locators for result verification
        const finalDisplay = calculatorWindow.locator("Name:Display is 3");
        const fallbackDisplay = calculatorWindow.locator("Id:CalculatorResults"); // Often more stable ID

        // 3. Get initial text
        console.log("\n--- 3. Getting Initial Text ---");
        const initialTextResponse = await displayElement.getText();
        console.log(`Initial display text: ${initialTextResponse?.text}`);

        // 4. Perform clicks (1 + 2 =)
        console.log("\n--- 4. Performing Clicks ---");
        await button1.click();
        await sleep(500);
        await buttonPlus.click();
        await sleep(500);
        await button2.click();
        await sleep(500);
        await buttonEquals.click();
        await sleep(1000); // Wait for calculation

        // 5. Get final text
        console.log("\n--- 5. Getting Final Text ---");
        try {
             // Try the locator that expects the result "3"
             const finalTextResponse = await finalDisplay.getText();
             console.log(`Final display text: ${finalTextResponse?.text}`);
        } catch (e) {
            if (e instanceof ApiError) {
                 // If the above failed (e.g., element name didn't become "Display is 3")
                 console.log(`Could not find element by expected final name (${finalDisplay._selector_chain}): ${e.message}. Trying fallback...`);
                 // Fallback: try getting text using the more stable AutomationId
                 const fallbackTextResponse = await fallbackDisplay.getText();
                 console.log(`Final display text (fallback by ID): ${fallbackTextResponse?.text}`);
            } else {
                 // Re-throw unexpected errors
                 throw e;
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
        // await calculatorWindow.pressKey("%{F4}"); // Alt+F4 on Windows - Ensure key codes match server expectation

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

    console.log("\n--- Example Finished ---");

})(); // End of async IIFE
