"use client"; // Required for useState, useEffect, and client-side SDK interaction

import React, { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import SdkFeatureCard from "@/components/SdkFeatureCard"; // Corrected path
import LogConsole from "@/components/LogConsole"; // Corrected path
import { getSdkClient, ApiError } from "@/lib/sdk"; // Corrected path
import { sleep } from "@/lib/utils"; // We'll need a sleep function

interface LogMessage {
  id: number;
  timestamp: string;
  type: "info" | "error" | "success" | "sdk-out" | "sdk-err";
  message: string;
}

// Define the structure for our feature showcases
interface SdkFeature {
  id: string;
  title: string;
  description: string;
  codeSnippet: string;
  action: (addLog: (type: LogMessage['type'], message: string) => void) => Promise<void>;
}

export default function HomePage() {
  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [isLoading, setIsLoading] = useState<Record<string, boolean>>({}); // Track loading state per feature

  const addLog = useCallback((type: LogMessage['type'], message: string) => {
    setLogs((prevLogs) => [
      ...prevLogs,
      {
        id: Date.now() + Math.random(), // Simple unique ID
        timestamp: new Date().toLocaleTimeString(),
        type,
        message,
      },
    ]);
  }, []);

  const handleAction = useCallback(async (featureId: string, actionFn: SdkFeature['action']) => {
    setIsLoading((prev) => ({ ...prev, [featureId]: true }));
    addLog("info", `Executing: ${featureId}...`);
    try {
      const client = getSdkClient(); // Get client instance
      await actionFn(addLog);
      // Success logging is handled within the actionFn for specific results
    } catch (error) {
      console.error(`Error executing ${featureId}:`, error);
      addLog("error", `Failed ${featureId}: ${error}`);
    } finally {
      setIsLoading((prev) => ({ ...prev, [featureId]: false }));
    }
  }, [addLog]);

  // --- Define SDK Features to Showcase ---
  const features: SdkFeature[] = [
    {
      id: "typeInNotepad",
      title: "Type into Notepad",
      description: "Opens Notepad, waits, locates editor, types text. (Windows Specific)",
      codeSnippet: `const client = getSdkClient();\nawait client.openApplication('notepad.exe');\nawait sleep(1000);\n// Selector might need adjustment!\nconst locator = client.locator('window:*Notepad').locator('Document');\nawait locator.typeText('Hello from SDK!');`,
      action: async (addLog) => {
          const isWindows = navigator.userAgent.toLowerCase().includes("win");
          if (!isWindows) {
              addLog("error", "This demo is Windows-specific (Notepad).");
              return;
          }
          const client = getSdkClient();
          addLog("info", "Opening Notepad...");
          const response = await client.openApplication('notepad.exe');
          addLog("success", `Opened Notepad: ${response.message}`);
          addLog("info", "Waiting 1 second for app to open...");
          await sleep(1000); // Use imported sleep

          addLog("info", "Locating Notepad window and document editor...");
          // Selectors are fragile and depend on OS, language, and app version.
          // 'Document' or 'ControlType.Document' or specific class names might work.
          // Requires the Terminator backend and accessibility APIs to find the element.
          // Using wildcard title '*' might be necessary if title changes (e.g., 'Untitled - Notepad')
          const locator = client.locator('window:Notepad'); // Common selector for text area

          addLog("info", "Attempting to type text...");
          try {
              const response = await locator.typeText('Hello from Terminator SDK!');
              addLog("success", `Text typed: ${response.message}`);
          } catch (e) {
              addLog("error", `Failed to type text. Could not find element or other error: ${e instanceof Error ? e.message : String(e)}`);
              addLog("info", "Make sure Notepad is open and has focus. The selector 'window:*Notepad' -> 'Document' might need adjustment for your system/language.");
          }
      }
    },
    {
      id: "openNotepad",
      title: "Open Application",
      description: "Opens Notepad (Windows specific example).",
      codeSnippet: `const client = getSdkClient();\nawait client.openApplication('notepad.exe');`,
      action: async (addLog) => {
        const isWindows = navigator.userAgent.toLowerCase().includes("win");
        const appName = isWindows ? "notepad.exe" : "TextEdit"; // Example for Mac
        addLog("info", `Attempting to open: ${appName}`);
        const client = getSdkClient();
        const response = await client.openApplication(appName);
        addLog("success", `Opened ${appName}: ${response.message}`);
      },
    },
    {
      id: "activateNotepad",
      title: "Activate Application",
      description: "Activates/Focuses Notepad if it's already open.",
      codeSnippet: `const client = getSdkClient();\nawait client.activateApplication('notepad.exe');`,
      action: async (addLog) => {
        const isWindows = navigator.userAgent.toLowerCase().includes("win");
        const appName = isWindows ? "notepad.exe" : "TextEdit";
        addLog("info", `Attempting to activate: ${appName}`);
        const client = getSdkClient();
        try {
            const response = await client.activateApplication(appName);
            addLog("success", `Activate requested for ${appName}: ${response.message}`);
        } catch(e) {
            addLog("error", `Could not activate ${appName}. Is it running? Error: ${e instanceof Error ? e.message : String(e)}`);
        }
      },
    },
    {
      id: "runCommand",
      title: "Run Command",
      description: "Executes a simple 'echo' command.",
      codeSnippet: `const client = getSdkClient();\nawait client.runCommand({\n  windowsCommand: 'echo "Hello from SDK!"',\n  unixCommand: 'echo "Hello from SDK!"'\n});`,
      action: async (addLog) => {
        const client = getSdkClient();
        const response = await client.runCommand({
            windowsCommand: 'echo "Hello from SDK!"',
            unixCommand: 'echo "Hello from SDK!"' // Provide both for cross-platform
        });
        addLog("success", `Command executed (Exit Code: ${response.exit_code ?? 'N/A'})`);
        if (response.stdout) addLog("sdk-out", `Stdout: ${response.stdout}`);
        if (response.stderr) addLog("sdk-err", `Stderr: ${response.stderr}`);
      },
    },
     {
      id: "openUrl",
      title: "Open URL",
      description: "Opens a URL in the default browser.",
      codeSnippet: `const client = getSdkClient();\nawait client.openUrl('https://google.com');`,
      action: async (addLog) => {
        const url = 'https://google.com';
        addLog("info", `Attempting to open URL: ${url}`);
        const client = getSdkClient();
        const response = await client.openUrl(url);
        addLog("success", `Open URL request sent: ${response.message}`);
      },
    },
    {
        id: "openFile",
        title: "Open File",
        description: "Attempts to open a file (e.g., 'C:\\test.txt'). Requires server access.",
        codeSnippet: `const client = getSdkClient();\n// Ensure the server has access to this path!\nawait client.openFile('C:\\\\test.txt');`,
        action: async (addLog) => {
            const isWindows = navigator.userAgent.toLowerCase().includes("win");
            // Use a plausible path based on OS for demo, but emphasize server access need
            const filePath = isWindows ? 'C:\\windows\\system.ini' : '/etc/hosts';
            addLog("info", `Attempting to open file: ${filePath} (Server must have access)`);
            const client = getSdkClient();
            try {
                const response = await client.openFile(filePath);
                addLog("success", `Open file request sent for ${filePath}: ${response.message}`);
            } catch(e) {
                addLog("error", `Failed to open ${filePath}. Server access issue or file not found? Error: ${e instanceof Error ? e.message : String(e)}`);
            }
        },
    },
     {
       id: "captureScreen",
       title: "Capture Screen",
       description: "Captures a screenshot of the primary monitor.",
       codeSnippet: `const client = getSdkClient();\nconst screenshot = await client.captureScreen();\nconsole.log(screenshot.width, screenshot.height);`,
       action: async (addLog) => {
         addLog("info", "Capturing screen...");
         const client = getSdkClient();
         const response = await client.captureScreen();
         addLog("success", `Screen captured: ${response.width}x${response.height}. Base64 data omitted.`);
         // You could store response.image_base64 if needed for OCR etc.
       },
     },
    // {
    //     id: "ocrScreen",
    //     title: "OCR Screen",
    //     description: "Captures the screen and performs OCR on it.",
    //     codeSnippet: `const client = getSdkClient();\nconst screen = await client.captureScreen();\nconst ocrResult = await client.ocrScreenshot({\n imageBase64: screen.image_base64,\n width: screen.width,\n height: screen.height\n});\nconsole.log(ocrResult.text);`,
    //     action: async (addLog) => {
    //         addLog("info", "Capturing screen for OCR...");
    //         const client = getSdkClient();
    //         const screen = await client.captureScreen();
    //         addLog("info", `Screen captured (${screen.width}x${screen.height}). Performing OCR...`);
    //         const ocrResult = await client.ocrScreenshot({
    //             imageBase64: screen.image_base64,
    //             width: screen.width,
    //             height: screen.height
    //         });
    //         addLog("success", `OCR completed.`);
    //         addLog("sdk-out", `OCR Text (truncated): ${ocrResult.text.substring(0, 100)}...`);
    //     },
    // },
    //   {
    //    id: "activateBrowser",
    //    title: "Activate Browser Window",
    //    description: "Activates browser window by title (e.g., 'Google'). Requires a matching window.",
    //    codeSnippet: `const client = getSdkClient();\nawait client.activateBrowserWindowByTitle('Google');`,
    //    action: async (addLog) => {
    //      const title = 'Google'; // Example title
    //      addLog("info", `Attempting to activate browser window with title containing: "${title}"`);
    //      const client = getSdkClient();
    //      const response = await client.activateBrowserWindowByTitle(title);
    //      addLog("success", `Activate browser request sent: ${response.message}`);
    //    },
    //  },
     // Example: Type into Notepad (more complex, requires locator)

  ];

  return (
    <main className="container mx-auto px-4 py-8">
      <div className="mb-8 flex items-center justify-between">
         <h1 className="text-2xl font-semibold tracking-tight">
            Terminator SDK Showcase
         </h1>
         {/* Add any global controls if needed */}
      </div>

      <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
        {features.map((feature) => (
          <SdkFeatureCard
            key={feature.id}
            title={feature.title}
            description={feature.description}
            codeSnippet={feature.codeSnippet}
          >
            <Button
              onClick={() => handleAction(feature.id, feature.action)}
              disabled={isLoading[feature.id]}
              size="sm"
            >
              {isLoading[feature.id] ? "Executing..." : "Run Action"}
            </Button>
          </SdkFeatureCard>
        ))}
      </div>

      <LogConsole logs={logs} />
    </main>
  );
}