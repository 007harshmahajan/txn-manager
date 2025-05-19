use serde::{Deserialize, Serialize};

/// Standard API response structure for consistent response formats
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Status of the response (usually "success" or "error")
    pub status: String,
    /// Human-readable message about the response
    pub message: String,
    /// Optional data payload - only included when there is data to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Creates a success response with data
    ///
    /// # Arguments
    /// * `message` - A message describing the successful operation
    /// * `data` - The data to include in the response
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            status: "success".to_string(),
            message: message.into(),
            data: Some(data),
        }
    }

    /// Creates a success response without any data
    ///
    /// This is useful for operations that don't return data, such as deletions
    ///
    /// # Arguments
    /// * `message` - A message describing the successful operation
    pub fn success_no_data(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            status: "success".to_string(),
            message: message.into(),
            data: None,
        }
    }
}
