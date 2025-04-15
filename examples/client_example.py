"""Example usage of the Terminator Python SDK from the root examples folder."""

import logging
import sys
import os
import time

# Add the python-sdk directory to the path to find the terminator_sdk module
SDK_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'python-sdk'))
if SDK_PATH not in sys.path:
    sys.path.insert(0, SDK_PATH)

# Now we can import the SDK
from desktop_use import DesktopUseClient, ApiError, ConnectionError, sleep

# --- Configuration --- #
# Optional: Configure logging for more detailed output
# logging.basicConfig(level=logging.DEBUG,
#                     format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logging.basicConfig(level=logging.INFO,
                    format='%(levelname)s: %(message)s')

# Ensure the Terminator server (e.g., `cargo run --example server`) is running!

def run_example():
    """Runs the calculator automation example."""
    # Ensure the package is installed or path is set correctly
    try:
        from desktop_use import DesktopUseClient, ApiError, ConnectionError, sleep
    except ImportError:
        print("Error: Could not import desktop_use.", file=sys.stderr)
        print(f"Ensure the SDK is installed (pip install -e ../python-sdk) or SDK path is correct ({SDK_PATH}).", file=sys.stderr)
        sys.exit(1)

    client = DesktopUseClient()

    try:
        # 1. Open Calculator
        print("\n--- 1. Opening Application ---")
        # Adjust app name if necessary (e.g., 'Calculator' or 'calc' on Windows)
        client.open_application("Calc")
        sleep(2.0) # Allow app to open

        # 2. Create locators using chaining
        print("\n--- 2. Defining Locators ---")
        # Adjust window selector if title is different (e.g., "Calculator" vs "Calc")
        calculator_window = client.locator("window:Calculator")
        # Locators relative to the calculator window
        # IMPORTANT: Selectors might differ on non-Windows platforms or even Win versions
        display_element = calculator_window.locator("Id:CalculatorResults") # Using AutomationId is often more stable
        button_1 = calculator_window.locator("Name:One")
        button_plus = calculator_window.locator("Name:Plus")
        button_2 = calculator_window.locator("Name:Two")
        button_equals = calculator_window.locator("Name:Equals")

        # 3. Get initial text (with expect_visible)
        print("\n--- 3. Getting Initial Text ---")
        try:
            display_element.expect_visible(timeout=3000) # Wait up to 3 seconds
            initial_text = display_element.get_text()
            print(f"Initial display text: {initial_text.text}")
        except ApiError as e:
            print(f"Warning: Could not get initial display text: {e}", file=sys.stderr)

        # 4. Perform clicks (1 + 2 =)
        print("\n--- 4. Performing Clicks --- (1 + 2 =)")
        button_1.click()
        sleep(0.5)
        button_plus.click()
        sleep(0.5)
        button_2.click()
        sleep(0.5)
        button_equals.click()
        sleep(1.0) # Wait for calculation

        # 5. Verify final text using expect
        print("\n--- 5. Verifying Final Text --- (Expecting 3)")
        try:
            # Note: Calculator display might show 'Display is 3' or just '3'.
            # We'll expect the exact text content '3'.
            display_element.expect_text_equals("3", timeout=5000, max_depth=1)
            print("Final display text is verified to be '3'")

            # Optionally get text again after verification
            final_text = display_element.get_text()
            print(f"Final display text (raw): {final_text.text}")

        except ApiError as e:
            print(f"Verification failed or could not get final text: {e}", file=sys.stderr)
            # Try getting raw text anyway on failure for debugging
            try:
                raw_text = display_element.get_text()
                print(f"Raw text on failure: {raw_text.text}", file=sys.stderr)
            except ApiError as inner_e:
                print(f"Could not get raw text after verification failure: {inner_e}", file=sys.stderr)

        # Example: Get attributes
        print("\n--- Example: Get Attributes of '=' button ---")
        attrs = button_equals.get_attributes()
        print(f"Equals button attributes: {attrs}") # Dataclasses have a nice __repr__

        # Example: Check visibility
        print("\n--- Example: Check Visibility of '=' button ---")
        is_visible = button_equals.is_visible()
        print(f"Is Equals button visible? {is_visible}")

        # Optional: Close the calculator
        # print("\n--- Optional: Closing Calculator ---")
        # try:
        #     calculator_window.press_key("%{F4}") # Alt+F4 on Windows
        #     print("Sent close command.")
        # except ApiError as e:
        #     print(f"Warning: Could not send close command: {e}", file=sys.stderr)

    except ConnectionError as e:
        print(f"\n{e}", file=sys.stderr)
        print("Please ensure the Terminator server (`cargo run --example server`) is running.", file=sys.stderr)
        sys.exit(1)
    except ApiError as e:
        print(f"\nAPI Error occurred: {e}", file=sys.stderr)
        sys.exit(1)
    except ImportError as e:
        print(f"\nImport Error: {e}", file=sys.stderr)
        print(f"Ensure the SDK path is correct ({SDK_PATH}) and dependencies are installed.", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\nAn unexpected error occurred: {e}", file=sys.stderr)
        logging.exception("Unexpected error details:") # Log stack trace for unexpected errors
        sys.exit(1)

    print("\n--- Example Finished ---")

if __name__ == "__main__":
    run_example() 