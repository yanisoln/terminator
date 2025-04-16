import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  TerminatorTools,
  FindWindowSchema,
  GetElementTextSchema,
  TypeIntoElementSchema,
  LocatorSchema, // For click
  PressKeySchema,
  RunCommandSchema,
  ExploreSchema,
} from "./terminator.js"; // Import the wrapper class and schemas

// --- Configuration ---
// You might want to load this from .env or config files
const TERMINATOR_BASE_URL = process.env.TERMINATOR_URL; // e.g., http://127.0.0.1:9375 or leave undefined for default

// --- Initialize Terminator Tools ---
// Initialization is wrapped in the main function to handle potential errors
let terminatorTools: TerminatorTools;

// --- MCP Server Setup ---
const serverInfo = {
  name: "terminator-mcp-agent",
  version: "0.1.0",
  description: "An MCP server providing desktop automation via Terminator.",
};

const serverInstructions = `
You are an AI assistant capable of controlling a computer desktop using the available tools.
You can interact with UI elements, run commands, and read text.

**Workflow:**
1.  **Identify Target Window:** Use 'findWindow' to locate the main window of the application you need to interact with (e.g., by title). Note its selector (e.g., an ID like '#someId' or role:Name like 'window:"My App"').
2.  **(Optional) Explore:** Use 'explore' with the window's selector chain to understand its structure and find specific child elements (buttons, inputs, etc.) and their selectors.
3.  **Interact:** Use tools like 'clickElement', 'getElementText', 'typeIntoElement', or 'pressKey' with the appropriate **selector chain** (starting from the window selector).
    *   A selector chain is an array, e.g., ['window:"My App"', 'button:"Save"'].
    *   Use selectors suggested by 'explore' or common patterns (role:Name, role:edit, etc.).
4.  **Execute Commands:** Use 'runCommand' for non-UI tasks like running scripts or opening files via the command line.

**Available Tools:**

- **findWindow**: Finds a top-level window by its title. Returns the window element's details, including a potential selector.
- **explore**: Lists child elements within a given element (or the screen). Use its output to find selectors for other tools.
- **clickElement**: Clicks a UI element specified by its selector chain.
- **getElementText**: Reads text content from a UI element specified by its selector chain.
- **typeIntoElement**: Types text into a UI element (like an input field) specified by its selector chain.
- **pressKey**: Sends a key press (like 'Enter', 'Tab', 'Ctrl+C') to a UI element specified by its selector chain.
- **runCommand**: Executes a shell command on the system (specify Windows or Unix command).

**Important:** Always provide the full selector chain when interacting with elements inside a window. Start the chain with the window selector found using 'findWindow' or 'explore'.
`;

const server = new McpServer(serverInfo, { instructions: serverInstructions });


// --- Tool Definitions ---

// findWindow Tool
server.tool(
    "findWindow",
    "Finds a top-level window by title.",
    FindWindowSchema.shape,
    async (args) => {
      const result = await terminatorTools.findWindow(args)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);

// getElementText Tool
server.tool(
    "getElementText",
    "Reads text content from a UI element.",
    GetElementTextSchema.shape,
    async (args) => {
      const result = await terminatorTools.getElementText(args)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);

// typeIntoElement Tool
server.tool(
    "typeIntoElement",
    "Types text into a UI element.",
    TypeIntoElementSchema.shape,
    async (args) => {
      const result = await terminatorTools.typeIntoElement(args)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);

// clickElement Tool
server.tool(
    "clickElement",
    "Clicks a UI element.",
    LocatorSchema.shape,
    async (args) => {
      const result = await terminatorTools.clickElement(args)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);

// pressKey Tool
server.tool(
    "pressKey",
    "Sends a key press to a UI element.",
    PressKeySchema.shape,
    async (args) => {
      const result = await terminatorTools.pressKey(args)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);


// runCommand Tool
server.tool(
    "runCommand",
    "Executes a shell command.",
    RunCommandSchema.shape,
    async (args) => {
      const result = await terminatorTools.runCommand(args)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);

// explore Tool
server.tool(
    "explore",
    "Explores UI elements and their children.",
    ExploreSchema.shape,
    async (args) => {
      const result = await terminatorTools.explore(args)
      console.log(result)
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
      };
    }
);

// --- Start Server ---
async function main() {
  console.log(`Starting ${serverInfo.name} v${serverInfo.version}...`);

  try {
    // Initialize TerminatorTools here so errors are caught before server connection
    terminatorTools = new TerminatorTools(TERMINATOR_BASE_URL);
    console.log("TerminatorTools initialized successfully.");
  } catch (error) {
    console.error("Failed to initialize TerminatorTools:", error);
    process.exit(1); // Exit if tools can't be initialized
  }

  const transport = new StdioServerTransport();
  try {
    // Connect the server to the transport (stdio in this case)
    await server.connect(transport);
    console.log("Server connected and listening on stdio.");
    console.log("Ready to receive MCP requests.");
  } catch (error) {
    console.error("Failed to start or connect server:", error);
    process.exit(1); // Exit if connection fails
  }
}

// Run the main function and handle potential errors
main().catch((error) => {
  console.error("Uncaught error in main execution:", error);
  process.exit(1); // Exit on unhandled errors
});
