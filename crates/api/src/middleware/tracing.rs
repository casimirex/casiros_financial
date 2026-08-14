//! Request-id generation, for correlating a client's request with its logs.

use uuid::Uuid;

/// The HTTP header used to propagate a request's id to the client.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Generates a new, random request id.
#[must_use]
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::new_request_id;

    #[test]
    fn generates_unique_ids() {
        let a = new_request_id();
        let b = new_request_id();
        assert_ne!(a, b);
    }

    #[test]
    fn generates_a_parseable_uuid() {
        let id = new_request_id();
        assert!(uuid::Uuid::parse_str(&id).is_ok());
    }
}
