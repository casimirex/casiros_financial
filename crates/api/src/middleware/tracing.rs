//! Request-id generation, for correlating a client's request with its logs.

use uuid::Uuid;

/// The HTTP header used to propagate a request's id to the client.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Generates a new, random request id.
#[must_use]
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}
