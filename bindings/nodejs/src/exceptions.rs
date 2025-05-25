use napi::{self, Status};
use napi_derive::napi;
use terminator::errors::AutomationError;

/// Thrown when an element is not found.
#[napi(js_name = "ElementNotFoundError")]
pub struct ElementNotFoundError(pub String);

/// Thrown when an operation times out.
#[napi(js_name = "TimeoutError")]
pub struct TimeoutError(pub String);

/// Thrown when permission is denied.
#[napi(js_name = "PermissionDeniedError")]
pub struct PermissionDeniedError(pub String);

/// Thrown for platform-specific errors.
#[napi(js_name = "PlatformError")]
pub struct PlatformError(pub String);

/// Thrown for unsupported operations.
#[napi(js_name = "UnsupportedOperationError")]
pub struct UnsupportedOperationError(pub String);

/// Thrown for unsupported platforms.
#[napi(js_name = "UnsupportedPlatformError")]
pub struct UnsupportedPlatformError(pub String);

/// Thrown for invalid arguments.
#[napi(js_name = "InvalidArgumentError")]
pub struct InvalidArgumentError(pub String);

/// Thrown for internal errors.
#[napi(js_name = "InternalError")]
pub struct InternalError(pub String);

// Implement Display and Error for all error classes
macro_rules! impl_js_error {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($name), self.0)
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($name), self.0)
            }
        }
        impl std::error::Error for $name {}
    };
}

impl_js_error!(ElementNotFoundError);
impl_js_error!(TimeoutError);
impl_js_error!(PermissionDeniedError);
impl_js_error!(PlatformError);
impl_js_error!(UnsupportedOperationError);
impl_js_error!(UnsupportedPlatformError);
impl_js_error!(InvalidArgumentError);
impl_js_error!(InternalError);

/// Map Terminator errors to NAPI errors
pub fn map_error(err: AutomationError) -> napi::Error {
    match err {
        AutomationError::ElementNotFound(msg) => {
            napi::Error::new(Status::InvalidArg, format!("Element not found: {}", msg))
        }
        AutomationError::Timeout(msg) => {
            napi::Error::new(Status::GenericFailure, format!("Operation timed out: {}", msg))
        }
        AutomationError::PermissionDenied(msg) => {
            napi::Error::new(Status::GenericFailure, format!("Permission denied: {}", msg))
        }
        AutomationError::PlatformError(e) => {
            napi::Error::new(Status::GenericFailure, format!("Platform error: {}", e))
        }
        AutomationError::UnsupportedOperation(msg) => {
            napi::Error::new(Status::InvalidArg, format!("Unsupported operation: {}", msg))
        }
        AutomationError::UnsupportedPlatform(msg) => {
            napi::Error::new(Status::InvalidArg, format!("Unsupported platform: {}", msg))
        }
        AutomationError::InvalidArgument(e) => {
            napi::Error::new(Status::InvalidArg, format!("Invalid argument: {}", e))
        }
        AutomationError::Internal(e) => {
            napi::Error::new(Status::GenericFailure, format!("Internal error: {}", e))
        }
    }
} 