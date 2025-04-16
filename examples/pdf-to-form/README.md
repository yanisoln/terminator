# Example: PDF to Form Automation



https://github.com/user-attachments/assets/0539bdf7-4253-47a8-8c1d-0b5da6c2fefd



This example demonstrates how to use the `terminator` library and the Vercel AI SDK to automate the process of extracting data from a PDF file and filling it into a form (web or desktop native).

The script uses AI to:
1.  Identify the PDF viewer window and the form window.
2.  Read the content from the PDF document.
3.  Identify and fill the corresponding input fields in the form using the extracted data.

## Prerequisites

1.  **Terminator Server:** Ensure the `terminator` server is running. Follow the setup instructions in the main [project README](../../README.md#quick-start).
2.  **Node.js and npm:** Make sure you have Node.js and npm installed.
3.  **Gemini API Key:** You need a Google Gemini API key. The script will prompt you to enter it if it's not found in a `.env` file within this example directory.

## Getting Started

1.  **Navigate to Example Directory:**
    ```bash
    cd examples/pdf-to-form
    ```
2.  **Install Dependencies:**
    ```bash
    npm install
    ```
3.  **(Optional) Create `.env` file:** Create a file named `.env` in this directory (`examples/pdf-to-form`) and add your Gemini API key:
    ```env
    GEMINI_API_KEY=YOUR_GEMINI_API_KEY_HERE
    ```
    If you skip this, the script will ask for the key when you run it.
4.  **Run the Script:**
    ```bash
    npm run dev
    ```
5.  **Manual Setup:** The script will pause and ask you to manually:
    *   Open the `data.pdf` file located in this directory in Microsoft Edge (or adjust the path in `src/index.ts` if needed).
    *   Open the target web application (`https://v0-pharmaceutical-form-design-5aeik3.vercel.app/`) in another Microsoft Edge window.
    *   Arrange the PDF window on the left side of your screen and the web app window on the right side.
    *   Confirm in the terminal when the setup is complete.
6.  **Automation:** Once confirmed, the AI will take over, identify the windows, read the PDF, and attempt to fill the web form. Watch the terminal for progress and tool calls.

## How it Works

The script (`src/index.ts`) connects to the running `terminator` server using the `desktop-use` client library. It defines tools for the AI (powered by the Vercel AI SDK and Google Gemini) to interact with the desktop UI:
*   `findWindow`: Locates the PDF and web app windows.
*   `readElementText`: Reads text content from UI elements (specifically the PDF document).
*   `typeIntoElement`: Types text into input fields on the web form.
*   `finishTask`: A placeholder tool called when the AI believes the task is complete.

The AI follows a system prompt guiding it through the steps of finding windows, reading the PDF content *within the correct window*, and then filling the form fields *within the web app window*.
