import * as http from 'http';
import { TextDecoder } from 'util';

const DEFAULT_BASE_URL = "127.0.0.1:3000";

// --- Interfaces for API Payloads and Responses matching server.rs --- //

// General Responses
export interface BasicResponse {
    message: string;
}

export interface BooleanResponse {
    result: boolean;
}

export interface TextResponse {
    text: string;
}

// Element Related Responses
export interface ElementResponse {
    role: string;
    label?: string | null;
    id?: string | null;
}

export interface ElementsResponse {
    elements: ElementResponse[];
}

export interface ClickResponse {
    method: string;
    coordinates?: [number, number] | null; // Changed from (f64, f64) to tuple
    details: string;
}

export interface AttributesResponse {
    role: string;
    label?: string | null;
    value?: string | null;
    description?: string | null;
    properties: { [key: string]: any | null }; // Using 'any' for simplicity with serde_json::Value
    id?: string | null;
}

export interface BoundsResponse {
    x: number;
    y: number;
    width: number;
    height: number;
}

// Base Request with Selector Chain
interface ChainedRequest {
    selector_chain: string[];
}

// Specific Action Requests
interface TypeTextRequest extends ChainedRequest {
    text: string;
}

interface GetTextRequest extends ChainedRequest {
    max_depth?: number | null;
}

interface PressKeyRequest extends ChainedRequest {
    key: string;
}

// App/URL Requests
interface OpenApplicationRequest {
    app_name: string;
}

interface OpenUrlRequest {
    url: string;
    browser?: string | null;
}

// --- Expectation Requests --- //

// Base request for expect actions, includes timeout
interface ExpectRequest extends ChainedRequest {
    timeout_ms?: number | null; // Optional timeout override in milliseconds
}

// Specific request for expecting text
interface ExpectTextRequest extends ExpectRequest { // Inherits selector_chain and timeout_ms
    expected_text: string;
    max_depth?: number | null; // Optional depth for text comparison
}

// --- Custom Error --- //

export class ApiError extends Error {
    status?: number;
    constructor(message: string, status?: number) {
        super(message);
        this.name = "ApiError";
        this.status = status;
        // Set the prototype explicitly for proper instanceof checks
        Object.setPrototypeOf(this, ApiError.prototype);
    }
}

// --- Client and Locator Classes --- //

export class DesktopUseClient {
    private host: string;
    private port: number;

    constructor(baseUrl: string = DEFAULT_BASE_URL) {
        const [host, portStr] = baseUrl.split(':');
        if (!host || !portStr) {
            throw new Error(`Invalid baseUrl format: ${baseUrl}. Expected format: 'host:port'`);
        }
        this.host = host;
        this.port = parseInt(portStr, 10);
        if (isNaN(this.port)) {
            throw new Error(`Invalid port number in baseUrl: ${portStr}`);
        }
    }

    /**
     * Internal method to make requests to the Terminator server.
     * Exposed for use by the Locator class, but intended for internal use.
     * @internal
     */
    public _makeRequest<T>(endpoint: string, payload: object): Promise<T> {
        return new Promise((resolve, reject) => {
            const jsonPayload = JSON.stringify(payload);
            console.log(`[TerminatorClient] Sending POST to ${endpoint} with payload: ${jsonPayload}`);

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
                    console.log(`[TerminatorClient] Response Status: ${res.statusCode}`);
                    console.log(`[TerminatorClient] Response Data: ${data}`);

                    if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
                        try {
                           if (!data || res.statusCode === 204) { // Handle No Content
                               // Resolve with a generic success message if no data
                               resolve({ message: "Operation successful" } as T);
                           } else {
                               resolve(JSON.parse(data) as T);
                           }
                        } catch (e) {
                            console.error("[TerminatorClient] Error: Could not decode JSON response despite success status.", e);
                            reject(new ApiError("Invalid JSON response from server", res.statusCode));
                        }
                    } else {
                        let errorMessage = `Server returned status ${res.statusCode}`;
                        let errorDetail = data; // Default detail is the raw response body
                        try {
                            // Attempt to parse the error response as JSON for a structured message
                            const errorData = JSON.parse(data);
                            errorMessage = errorData?.message || errorMessage;
                            // Include the original detail if parsing failed or message was missing
                            if (!errorData?.message) errorDetail = data;
                        } catch (e) {
                            // Ignore JSON parse error, use the raw data as detail
                        }
                        console.error(`[TerminatorClient] Error: ${errorMessage}, Detail: ${errorDetail}`);
                        reject(new ApiError(errorMessage, res.statusCode));
                    }
                });
            });

            req.on('error', (e) => {
                console.error(`[TerminatorClient] Request error: ${e.message}`);
                const nodeError = e as NodeJS.ErrnoException;
                if (nodeError.code === 'ECONNREFUSED') {
                     reject(new ApiError(`Connection refused to ${this.host}:${this.port}. Is the Terminator server running?`, nodeError.errno));
                } else {
                     reject(new ApiError(`Request error: ${nodeError.message}`, nodeError.errno));
                }
            });

            // Write data to request body and end request
            req.write(jsonPayload);
            req.end();
        });
    }

    /**
     * Creates a new Locator instance starting from the root of the application hierarchy.
     * @param selector - The initial selector string (e.g., 'window:My App', 'Id:someId', 'Name:Click Me').
     * @returns A new Locator instance.
     */
    locator(selector: string): Locator {
        if (!selector || typeof selector !== 'string') {
            throw new Error('Initial selector must be a non-empty string.');
        }
        return new Locator(this, [selector]);
    }

    /**
     * Opens an application by its name or path.
     * @param appName - The name or path of the application to open.
     * @returns A promise resolving to a basic response on success.
     */
    async openApplication(appName: string): Promise<BasicResponse> {
        const payload: OpenApplicationRequest = { app_name: appName };
        return await this._makeRequest<BasicResponse>("/open_application", payload);
    }

    /**
     * Opens a URL, optionally specifying the browser.
     * @param url - The URL to open.
     * @param browser - Optional name or path of the browser to use. Uses system default if null/undefined.
     * @returns A promise resolving to a basic response on success.
     */
    async openUrl(url: string, browser?: string | null): Promise<BasicResponse> {
        const payload: OpenUrlRequest = { url, browser };
        return await this._makeRequest<BasicResponse>("/open_url", payload);
    }
}

export class Locator {
    // Marked internal, though technically public for access
    /** @internal */
    public readonly _client: DesktopUseClient;
    /** @internal */
    public readonly _selector_chain: string[];

    /** @internal */
    constructor(client: DesktopUseClient, selectorChain: string[]) {
        this._client = client;
        this._selector_chain = selectorChain;
    }

    /**
     * Creates a new Locator instance scoped to the current locator.
     * @param selector - The selector string to append to the current chain.
     * @returns A new Locator instance representing the nested element.
     */
    locator(selector: string): Locator {
        if (!selector || typeof selector !== 'string') {
            throw new Error('Nested selector must be a non-empty string.');
        }
        const newChain = [...this._selector_chain, selector];
        return new Locator(this._client, newChain);
    }

    // --- Action Methods based on server.rs endpoints --- //

    /**
     * Finds the first element matching the locator chain.
     * Waits for the element to appear if it's not immediately available (handled server-side).
     * @returns A promise resolving to the element's basic details.
     */
    async first(): Promise<ElementResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client._makeRequest<ElementResponse>("/first", payload);
    }

    /**
     * Finds all elements matching the last selector in the chain, within the context
     * established by the preceding selectors.
     * @returns A promise resolving to an array of element details.
     */
    async all(): Promise<ElementsResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client._makeRequest<ElementsResponse>("/all", payload);
    }

    /**
     * Clicks the element identified by the locator chain.
     * Waits for the element to be actionable (handled server-side).
     * @returns A promise resolving to details about the click action.
     */
    async click(): Promise<ClickResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client._makeRequest<ClickResponse>("/click", payload);
    }

    /**
     * Types the given text into the element identified by the locator chain.
     * Waits for the element to be actionable (handled server-side).
     * @param text - The text to type.
     * @returns A promise resolving to a basic response on success.
     */
    async typeText(text: string): Promise<BasicResponse> {
        const payload: TypeTextRequest = { selector_chain: this._selector_chain, text };
        return await this._client._makeRequest<BasicResponse>("/type_text", payload);
    }

    /**
     * Retrieves the text content of the element identified by the locator chain.
     * @param maxDepth - Optional maximum depth to search for text within child elements (defaults to server-side default, e.g., 5).
     * @returns A promise resolving to the element's text content.
     */
    async getText(maxDepth?: number | null): Promise<TextResponse> {
        const payload: GetTextRequest = { selector_chain: this._selector_chain, max_depth: maxDepth };
        return await this._client._makeRequest<TextResponse>("/get_text", payload);
    }

    /**
     * Retrieves the attributes of the element identified by the locator chain.
     * @returns A promise resolving to the element's attributes.
     */
    async getAttributes(): Promise<AttributesResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client._makeRequest<AttributesResponse>("/get_attributes", payload);
    }

    /**
     * Retrieves the bounding rectangle (position and size) of the element identified by the locator chain.
     * @returns A promise resolving to the element's bounds.
     */
    async getBounds(): Promise<BoundsResponse> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        return await this._client._makeRequest<BoundsResponse>("/get_bounds", payload);
    }

    /**
     * Checks if the element identified by the locator chain is currently visible.
     * Note: Visibility determination is platform-dependent.
     * @returns A promise resolving to true if the element is visible, false otherwise.
     */
    async isVisible(): Promise<boolean> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        // The server returns BooleanResponse, we extract the boolean result here
        const response = await this._client._makeRequest<BooleanResponse>("/is_visible", payload);
        return response?.result ?? false; // Safely access result, default to false if missing
    }

    /**
     * Sends keyboard key presses to the element identified by the locator chain.
     * Use syntax expected by the target platform (e.g., "Enter", "Ctrl+A", "%fx" for Alt+Fx).
     * @param key - The key or key combination to press.
     * @returns A promise resolving to a basic response on success.
     */
    async pressKey(key: string): Promise<BasicResponse> {
        const payload: PressKeyRequest = { selector_chain: this._selector_chain, key };
        return await this._client._makeRequest<BasicResponse>("/press_key", payload);
    }

    /**
     * Activates the application window associated with the element identified by the locator chain.
     * This typically brings the window to the foreground.
     * Waits for the element first (handled server-side).
     * @returns The current Locator instance to allow for method chaining.
     * @throws {ApiError} If the server fails to activate the application window.
     */
    async activateApp(): Promise<this> {
        const payload: ChainedRequest = { selector_chain: this._selector_chain };
        await this._client._makeRequest<BasicResponse>("/activate_app", payload);
        return this;
    }

    // --- Expectation Methods --- //

    /**
     * Waits for the element identified by the locator chain to be visible.
     * Throws an error if the element is not visible within the specified timeout.
     * @param timeout - Optional timeout in milliseconds to override the default.
     * @returns A promise resolving to the element's details if it becomes visible.
     */
    async expectVisible(timeout?: number | null): Promise<ElementResponse> {
        const payload: ExpectRequest = {
            selector_chain: this._selector_chain,
            timeout_ms: timeout,
        };
        return await this._client._makeRequest<ElementResponse>("/expect_visible", payload);
    }

    /**
     * Waits for the element identified by the locator chain to be enabled.
     * Throws an error if the element is not enabled within the specified timeout.
     * @param timeout - Optional timeout in milliseconds to override the default.
     * @returns A promise resolving to the element's details if it becomes enabled.
     */
    async expectEnabled(timeout?: number | null): Promise<ElementResponse> {
        const payload: ExpectRequest = {
            selector_chain: this._selector_chain,
            timeout_ms: timeout,
        };
        return await this._client._makeRequest<ElementResponse>("/expect_enabled", payload);
    }

    /**
     * Waits for the text content of the element identified by the locator chain
     * to equal the expected text.
     * Throws an error if the text does not match within the specified timeout.
     * @param expectedText - The exact text string to wait for.
     * @param options - Optional parameters including maxDepth for text retrieval and timeout.
     * @returns A promise resolving to the element's details if the text matches.
     */
    async expectTextEquals(expectedText: string, options?: { maxDepth?: number | null, timeout?: number | null }): Promise<ElementResponse> {
        const payload: ExpectTextRequest = {
            selector_chain: this._selector_chain,
            expected_text: expectedText,
            max_depth: options?.maxDepth,
            timeout_ms: options?.timeout,
        };
        return await this._client._makeRequest<ElementResponse>("/expect_text_equals", payload);
    }
}

// Utility function (optional, could be part of a separate utils file)
export function sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
} 