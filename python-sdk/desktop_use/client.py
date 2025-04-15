"""Main client and locator logic for the Terminator SDK."""

import requests
import json
import logging
import time
from typing import List, Optional, Dict, Any, Type, TypeVar
from dataclasses import asdict, dataclass

from .exceptions import ApiError, ConnectionError
from .models import (
    BasicResponse, BooleanResponse, TextResponse, ElementResponse, ElementsResponse,
    ClickResponse, AttributesResponse, BoundsResponse, ChainedRequest,
    TypeTextRequest, GetTextRequest, PressKeyRequest, OpenApplicationRequest,
    OpenUrlRequest, ExpectRequest, ExpectTextRequest,
    OpenFileRequest, RunCommandRequest, CaptureMonitorRequest, OcrImagePathRequest,
    CommandOutputResponse, ScreenshotResponse, OcrResponse,
    OcrScreenshotRequest
)

logger = logging.getLogger(__name__)

DEFAULT_BASE_URL = "http://127.0.0.1:9375"

# Generic type variable for response models
T = TypeVar('T')

# Add this dataclass
@dataclass
class ActivateBrowserWindowRequest:
    title: str

class DesktopUseClient:
    """Client for interacting with the Terminator desktop automation server."""

    def __init__(self, base_url: str = DEFAULT_BASE_URL):
        """
        Initializes the client.

        Args:
            base_url: The base URL of the Terminator server.
        """
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.session.headers.update({"Content-Type": "application/json"})
        logger.info(f"DesktopUseClient initialized for server at {self.base_url}")

    def _make_request(self, endpoint: str, payload: Any, response_model: Type[T]) -> T:
        """Internal helper to make POST requests and handle responses."""
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        # Convert dataclass payload to dict, handle non-dataclass payloads
        if hasattr(payload, '__dataclass_fields__'):
             payload_dict = asdict(payload, dict_factory=lambda x: {k: v for (k, v) in x if v is not None})
        elif isinstance(payload, dict):
            payload_dict = payload
        else:
            raise TypeError("Payload must be a dataclass instance or a dictionary")

        json_payload = json.dumps(payload_dict)
        logger.debug(f"Sending POST to {url} with payload: {json_payload}")

        try:
            response = self.session.post(url, data=json_payload)
            logger.debug(f"Response Status: {response.status_code}")
            logger.debug(f"Response Data: {response.text[:500]}...") # Log truncated response

            if 200 <= response.status_code < 300:
                try:
                    if not response.content or response.status_code == 204:
                        # Handle No Content or empty successful response
                        # Return default instance if possible, or a basic message
                        if response_model is BasicResponse:
                            return BasicResponse(message="Operation successful") # type: ignore
                        try:
                            # Try creating a default instance (works for some dataclasses)
                             return response_model()
                        except TypeError:
                             # Fallback if default construction fails
                             logger.warning(f"Received empty success response for {response_model.__name__}, returning default BasicResponse")
                             return BasicResponse(message="Operation successful (empty response)") # type: ignore

                    data = response.json()
                    # Use dictionary unpacking to create the dataclass instance
                    return response_model(**data)
                except (json.JSONDecodeError, TypeError) as e:
                    logger.error(f"Failed to decode JSON or create response model {response_model.__name__}: {e}", exc_info=True)
                    raise ApiError(f"Invalid JSON response or model mismatch: {e}", response.status_code)
            else:
                # Attempt to parse error message from server
                try:
                    error_data = response.json()
                    error_message = error_data.get('message', response.text)
                except json.JSONDecodeError:
                    error_message = response.text
                logger.error(f"API Error ({response.status_code}): {error_message}")
                raise ApiError(error_message, response.status_code)

        except requests.exceptions.ConnectionError as e:
            logger.error(f"Connection Error connecting to {url}: {e}", exc_info=True)
            raise ConnectionError(f"Could not connect to Terminator server at {url}. Is it running? Details: {e}")
        except requests.exceptions.RequestException as e:
            logger.error(f"Request failed: {e}", exc_info=True)
            raise ApiError(f"An unexpected request error occurred: {e}")

    def locator(self, selector: str) -> 'Locator':
        """
        Creates a new Locator instance starting from the root.

        Args:
            selector: The initial selector string.

        Returns:
            A new Locator instance.
        """
        if not isinstance(selector, str) or not selector:
            raise ValueError("Initial selector must be a non-empty string.")
        return Locator(self, [selector])

    def open_application(self, app_name: str) -> BasicResponse:
        """
        Opens an application by its name or path.

        Args:
            app_name: The name or path of the application.

        Returns:
            BasicResponse indicating success.
        """
        payload = OpenApplicationRequest(app_name=app_name)
        return self._make_request("/open_application", payload, BasicResponse)

    def activate_application(self, app_name: str) -> BasicResponse:
        """
        Activates an application by its name or path, bringing its window to the foreground.

        Args:
            app_name: The name or path of the application to activate.

        Returns:
            BasicResponse indicating success.
        """
        payload = ActivateApplicationRequest(app_name=app_name)
        return self._make_request("/activate_application", payload, BasicResponse)

    def open_url(self, url: str, browser: Optional[str] = None) -> BasicResponse:
        """
        Opens a URL, optionally in a specific browser.

        Args:
            url: The URL to open.
            browser: Optional browser name/path.

        Returns:
            BasicResponse indicating success.
        """
        payload = OpenUrlRequest(url=url, browser=browser)
        return self._make_request("/open_url", payload, BasicResponse)

    # --- New Top-Level Methods --- #

    def open_file(self, file_path: str) -> BasicResponse:
        """
        Opens a file using its default application.

        Args:
            file_path: The path to the file.

        Returns:
            BasicResponse indicating success.
        """
        payload = OpenFileRequest(file_path=file_path)
        return self._make_request("/open_file", payload, BasicResponse)

    def run_command(self, windows_command: Optional[str] = None, unix_command: Optional[str] = None) -> CommandOutputResponse:
        """
        Executes a command, choosing the appropriate one based on the server's OS.

        Provide at least one of windows_command or unix_command.

        Args:
            windows_command: The command to run on Windows.
            unix_command: The command to run on Unix-like systems (Linux, macOS).

        Returns:
            CommandOutputResponse containing stdout, stderr, and exit code.
        """
        if not windows_command and not unix_command:
            raise ValueError("At least one of windows_command or unix_command must be provided.")
        payload = RunCommandRequest(windows_command=windows_command, unix_command=unix_command)
        return self._make_request("/run_command", payload, CommandOutputResponse)

    def capture_screen(self) -> ScreenshotResponse:
        """
        Captures a screenshot of the primary monitor.

        Returns:
            ScreenshotResponse containing base64 encoded image data, width, and height.
        """
        # No payload needed for capture_screen
        return self._make_request("/capture_screen", {}, ScreenshotResponse)

    def capture_monitor_by_name(self, monitor_name: str) -> ScreenshotResponse:
        """
        Captures a screenshot of a specific monitor by its name.

        Args:
            monitor_name: The name of the monitor (platform-specific).

        Returns:
            ScreenshotResponse containing base64 encoded image data, width, and height.
        """
        payload = CaptureMonitorRequest(monitor_name=monitor_name)
        return self._make_request("/capture_monitor", payload, ScreenshotResponse)

    def ocr_image_path(self, image_path: str) -> OcrResponse:
        """
        Performs OCR on an image file located at the given path.
        The server needs access to this path.

        Args:
            image_path: The path to the image file.

        Returns:
            OcrResponse containing the extracted text.
        """
        payload = OcrImagePathRequest(image_path=image_path)
        return self._make_request("/ocr_image_path", payload, OcrResponse)

    def ocr_screenshot(self, image_base64: str, width: int, height: int) -> OcrResponse:
        """
        Performs OCR directly on raw image data (e.g., from a previous screenshot).

        Args:
            image_base64: The base64 encoded string of the image data.
            width: The width of the image in pixels.
            height: The height of the image in pixels.

        Returns:
            OcrResponse containing the extracted text.
        """
        payload = OcrScreenshotRequest(
            image_base64=image_base64,
            width=width,
            height=height
        )
        return self._make_request("/ocr_screenshot", payload, OcrResponse)

    def activate_browser_window_by_title(self, title: str) -> BasicResponse:
        """
        Activates a browser window if it contains an element (like a tab) with the specified title.

        This brings the browser window to the foreground.
        Note: This does not guarantee the specific tab will be active within the window.

        Args:
            title: The title (or partial title) of the tab/element to search for within browser windows.

        Returns:
            BasicResponse indicating success.
        """
        payload = ActivateBrowserWindowRequest(title=title)
        return self._make_request("/activate_browser_window", payload, BasicResponse)


class Locator:
    """Represents a UI element locator, allowing chained selections and actions."""

    def __init__(self, client: DesktopUseClient, selector_chain: List[str]):
        """Internal constructor. Use client.locator() to create instances."""
        self._client = client
        self._selector_chain = selector_chain

    def locator(self, selector: str) -> 'Locator':
        """
        Creates a new Locator scoped to the current one.

        Args:
            selector: The selector to append to the chain.

        Returns:
            A new Locator instance.
        """
        if not isinstance(selector, str) or not selector:
            raise ValueError("Nested selector must be a non-empty string.")
        new_chain = self._selector_chain + [selector]
        return Locator(self._client, new_chain)

    # --- Action Methods --- #

    def first(self) -> ElementResponse:
        """Finds the first element matching the locator chain."""
        payload = ChainedRequest(selector_chain=self._selector_chain)
        return self._client._make_request("/first", payload, ElementResponse)

    def all(self) -> ElementsResponse:
        """Finds all elements matching the last selector in the chain."""
        payload = ChainedRequest(selector_chain=self._selector_chain)
        return self._client._make_request("/all", payload, ElementsResponse)

    def click(self) -> ClickResponse:
        """Clicks the element."""
        payload = ChainedRequest(selector_chain=self._selector_chain)
        return self._client._make_request("/click", payload, ClickResponse)

    def type_text(self, text: str) -> BasicResponse:
        """Types text into the element."""
        payload = TypeTextRequest(selector_chain=self._selector_chain, text=text)
        return self._client._make_request("/type_text", payload, BasicResponse)

    def get_text(self, max_depth: Optional[int] = None) -> TextResponse:
        """Gets the text content of the element."""
        payload = GetTextRequest(selector_chain=self._selector_chain, max_depth=max_depth)
        return self._client._make_request("/get_text", payload, TextResponse)

    def get_attributes(self) -> AttributesResponse:
        """Gets the attributes of the element."""
        payload = ChainedRequest(selector_chain=self._selector_chain)
        return self._client._make_request("/get_attributes", payload, AttributesResponse)

    def get_bounds(self) -> BoundsResponse:
        """Gets the bounding rectangle of the element."""
        payload = ChainedRequest(selector_chain=self._selector_chain)
        return self._client._make_request("/get_bounds", payload, BoundsResponse)

    def is_visible(self) -> bool:
        """Checks if the element is currently visible."""
        payload = ChainedRequest(selector_chain=self._selector_chain)
        response = self._client._make_request("/is_visible", payload, BooleanResponse)
        return response.result

    def press_key(self, key: str) -> BasicResponse:
        """Sends key presses to the element."""
        payload = PressKeyRequest(selector_chain=self._selector_chain, key=key)
        return self._client._make_request("/press_key", payload, BasicResponse)

    def activate_app(self) -> 'Locator':
        """
        Activates the application window associated with the element.

        This typically brings the window to the foreground.
        Waits for the element first (handled server-side).

        Returns:
            The current Locator instance to allow for method chaining.
        Raises:
            ApiError: If the server fails to activate the application window.
        """
        payload = ChainedRequest(selector_chain=self._selector_chain)
        # Calls the new /activate_app endpoint
        self._client._make_request("/activate_app", payload, BasicResponse)
        return self # Return self for chaining

    # --- Expectation Methods --- #

    def expect_visible(self, timeout: Optional[int] = None) -> ElementResponse:
        """
        Waits for the element to be visible.

        Args:
            timeout: Optional timeout in milliseconds.

        Returns:
            ElementResponse if visible within timeout.
        Raises:
            ApiError: If timeout occurs or another error happens.
        """
        payload = ExpectRequest(selector_chain=self._selector_chain, timeout_ms=timeout)
        return self._client._make_request("/expect_visible", payload, ElementResponse)

    def expect_enabled(self, timeout: Optional[int] = None) -> ElementResponse:
        """
        Waits for the element to be enabled.

        Args:
            timeout: Optional timeout in milliseconds.

        Returns:
            ElementResponse if enabled within timeout.
        Raises:
            ApiError: If timeout occurs or another error happens.
        """
        payload = ExpectRequest(selector_chain=self._selector_chain, timeout_ms=timeout)
        return self._client._make_request("/expect_enabled", payload, ElementResponse)

    def expect_text_equals(
        self,
        expected_text: str,
        max_depth: Optional[int] = None,
        timeout: Optional[int] = None
    ) -> ElementResponse:
        """
        Waits for the element's text to equal the expected value.

        Args:
            expected_text: The exact text to match.
            max_depth: Optional depth for text retrieval.
            timeout: Optional timeout in milliseconds.

        Returns:
            ElementResponse if text matches within timeout.
        Raises:
            ApiError: If timeout occurs, text doesn't match, or another error happens.
        """
        payload = ExpectTextRequest(
            selector_chain=self._selector_chain,
            expected_text=expected_text,
            max_depth=max_depth,
            timeout_ms=timeout
        )
        return self._client._make_request("/expect_text_equals", payload, ElementResponse)

# Helper function (can be part of the SDK or used externally)
def sleep(seconds: float) -> None:
    """Pauses execution for a specified number of seconds."""
    time.sleep(seconds) 