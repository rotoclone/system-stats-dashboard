use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorContext {
    pub title: String,
    pub message: String,
}
