import { DesktopUseClient } from "desktop-use"; // Assuming ts-sdk is correctly resolved

// Initialize with the default URL or allow configuration via environment variables
const sdkBaseUrl = process.env.NEXT_PUBLIC_SDK_BASE_URL || "http://127.0.0.1:9375";

let clientInstance: DesktopUseClient | null = null;

// Function to get the client instance (singleton pattern for client-side)
export const getSdkClient = (): DesktopUseClient => {
  if (typeof window === "undefined") {
    // Return a dummy or throw error during SSR/build time if needed
    // For simplicity, let's allow it but expect it to be called client-side
    console.warn("Attempted to get SDK client on the server side.");
    // Return a mock or throw if strictness is required
    return new DesktopUseClient(sdkBaseUrl); // Or throw new Error("SDK client only available client-side");
  }
  if (!clientInstance) {
    console.log(`Initializing SDK client with base URL: ${sdkBaseUrl}`);
    clientInstance = new DesktopUseClient(sdkBaseUrl);
  }
  return clientInstance;
};
