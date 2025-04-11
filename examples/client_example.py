import http.client
import json
import sys
import time

BASE_URL = "127.0.0.1:3000"

class ApiError(Exception):
    """Custom exception for API errors."""
    def __init__(self, message, status=None):
        super().__init__(message)
        self.status = status

class TerminatorClient:
    """Client for interacting with the Terminator API server."""
    def __init__(self, base_url=BASE_URL):
        self.base_url = base_url
        self._conn_host = base_url.split(':')[0]
        self._conn_port = int(base_url.split(':')[1])

    def _make_request(self, endpoint, payload):
        """Internal helper to make POST requests."""
        try:
            conn = http.client.HTTPConnection(self._conn_host, self._conn_port)
            headers = {'Content-type': 'application/json'}
            json_payload = json.dumps(payload)
            
            print(f"Sending POST to {endpoint} with payload: {json_payload}")
            conn.request("POST", endpoint, json_payload, headers)
            
            response = conn.getresponse()
            status = response.status
            data = response.read().decode()
            conn.close()
            
            print(f"Response Status: {status}")
            print(f"Response Data: {data}")

            if 200 <= status < 300:
                try:
                    return json.loads(data)
                except json.JSONDecodeError:
                    # Some successful responses might have no body (e.g., simple message)
                    # Check if data is empty or just contains the success message
                    if not data or "opened" in data or "successfully" in data:
                         return {"message": data} # Return a dict for consistency
                    print("Error: Could not decode JSON response.")
                    raise ApiError("Invalid JSON response from server", status=status)
            else:
                error_message = f"Server returned status {status}"
                try:
                    error_data = json.loads(data)
                    error_message = error_data.get('message', error_message)
                except json.JSONDecodeError:
                    pass # Use the generic error message
                raise ApiError(error_message, status=status)
                
        except ConnectionRefusedError:
            print(f"Error: Connection refused. Is the server running at {self.base_url}?")
            raise ApiError(f"Connection refused to {self.base_url}")
        except ApiError: # Re-raise ApiErrors
             raise
        except Exception as e:
            print(f"An unexpected error occurred during the request: {e}")
            raise ApiError(f"Unexpected request error: {e}")

    def locator(self, selector: str):
        """Creates a new Locator instance starting a chain."""
        return Locator(self, [selector])

    def open_application(self, app_name: str):
        """Opens an application."""
        payload = {"app_name": app_name}
        return self._make_request("/open_application", payload)

    def open_url(self, url: str, browser: str = None):
        """Opens a URL."""
        payload = {"url": url, "browser": browser}
        return self._make_request("/open_url", payload)

class Locator:
    """Represents a locator chain, similar to Playwright's Locator."""
    def __init__(self, client: TerminatorClient, selector_chain: list):
        self._client = client
        self._selector_chain = selector_chain

    def locator(self, selector: str):
        """Creates a new locator by appending a selector to the current chain."""
        new_chain = self._selector_chain + [selector]
        return Locator(self._client, new_chain)

    # --- Action Methods --- 
    # These methods send the current selector_chain to the server

    def find_element(self):
        """Finds the first element matching the chain."""
        payload = {"selector_chain": self._selector_chain}
        return self._client._make_request("/find_element", payload)

    def find_elements(self):
        """Finds all elements matching the chain."""
        payload = {"selector_chain": self._selector_chain}
        return self._client._make_request("/find_elements", payload)
        
    def click(self):
        """Clicks the element matching the chain."""
        payload = {"selector_chain": self._selector_chain}
        return self._client._make_request("/click", payload)

    def type_text(self, text: str):
        """Types text into the element matching the chain."""
        payload = {"selector_chain": self._selector_chain, "text": text}
        return self._client._make_request("/type_text", payload)

    def get_text(self, max_depth: int = None):
        """Gets text from the element matching the chain."""
        payload = {"selector_chain": self._selector_chain, "max_depth": max_depth}
        return self._client._make_request("/get_text", payload)
        
    def get_attributes(self):
        """Gets attributes of the element matching the chain."""
        payload = {"selector_chain": self._selector_chain}
        return self._client._make_request("/get_attributes", payload)

    def get_bounds(self):
        """Gets bounds of the element matching the chain."""
        payload = {"selector_chain": self._selector_chain}
        return self._client._make_request("/get_bounds", payload)

    def is_visible(self):
        """Checks if the element matching the chain is visible."""
        payload = {"selector_chain": self._selector_chain}
        response = self._client._make_request("/is_visible", payload)
        return response.get('result', False) # Return boolean directly
        
    def press_key(self, key: str):
        """Presses a key on the element matching the chain."""
        payload = {"selector_chain": self._selector_chain, "key": key}
        return self._client._make_request("/press_key", payload)

# --- Example Usage using the SDK ---

if __name__ == "__main__":
    client = TerminatorClient()

    try:
        # 1. Open Calculator
        print("\n--- 1. Opening Application ---")
        client.open_application("Calc")
        time.sleep(2)

        # 2. Create locators using chaining
        print("\n--- 2. Defining Locators ---")
        calculator_window = client.locator("window:Calc")
        display_element = calculator_window.locator("Name:Display is 0")
        button_1 = calculator_window.locator("Name:One")
        button_plus = calculator_window.locator("Name:Plus")
        button_2 = calculator_window.locator("Name:Two")
        button_equals = calculator_window.locator("Name:Equals")
        final_display = calculator_window.locator("Name:Display is 3")
        fallback_display = calculator_window.locator("Id:CalculatorResults")

        # 3. Get initial text
        print("\n--- 3. Getting Initial Text ---")
        initial_text_response = display_element.get_text()
        print(f"Initial display text: {initial_text_response.get('text')}")

        # 4. Perform clicks (1 + 2 =)
        print("\n--- 4. Performing Clicks ---")
        button_1.click()
        time.sleep(0.5)
        button_plus.click()
        time.sleep(0.5)
        button_2.click()
        time.sleep(0.5)
        button_equals.click()
        time.sleep(1)

        # 5. Get final text
        print("\n--- 5. Getting Final Text ---")
        try:
             final_text_response = final_display.get_text() # Try selector for expected result
             print(f"Final display text: {final_text_response.get('text')}")
        except ApiError as e:
            print(f"Could not find element by expected final name ({final_display._selector_chain}): {e}. Trying fallback...")
            # Fallback: try getting text using the AutomationId 
            fallback_text_response = fallback_display.get_text()
            print(f"Final display text (fallback by ID): {fallback_text_response.get('text')}")
            
        # Example: Get attributes of the equals button
        print("\n--- Example: Get Attributes of '=' button ---")
        attrs = button_equals.get_attributes()
        print(f"Equals button attributes: {attrs}")

        # Example: Check visibility of the equals button
        print("\n--- Example: Check Visibility of '=' button ---")
        visible = button_equals.is_visible()
        print(f"Is Equals button visible? {visible}")

        # Optional: Close the calculator
        # print("\n--- Optional: Closing Calculator ---")
        # calculator_window.press_key("%{F4}") # Alt+F4 on Windows

    except ApiError as e:
        print(f"\nAPI Error occurred: {e} (Status: {e.status})")
        sys.exit(1)
    except Exception as e:
        print(f"\nAn unexpected error occurred: {e}")
        sys.exit(1)
        
    print("\n--- Example Finished ---") 