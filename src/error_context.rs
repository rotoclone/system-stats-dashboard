//! Context for the error template.

use serde::Serialize;

/// Context for the error template.
#[derive(Serialize)]
pub struct ErrorContext {
    /// The title of the page.
    pub title: String,
    /// The error message.
    pub message: String,
}
