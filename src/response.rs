//! Convenience helpers for building [`rmcp::model::CallToolResult`] values.
//!
//! These wrappers eliminate the repetitive boilerplate of constructing
//! `Content` vectors and setting the `is_error` flag for every tool handler.

use rmcp::model::{CallToolResult, Content};

/// Convenience constructors for MCP tool call results.
///
/// Each method returns a fully-formed [`CallToolResult`] ready to be
/// returned from a tool handler.
pub struct ToolResponse;

impl ToolResponse {
    /// Build a successful JSON result.
    ///
    /// The value is serialised with [`serde_json::to_string`] (compact)
    /// and wrapped in a single text content block.
    #[must_use]
    pub fn success(value: &serde_json::Value) -> CallToolResult {
        let text = serde_json::to_string(value).unwrap_or_else(|e| e.to_string());
        CallToolResult {
            content: vec![Content::text(text)],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        }
    }

    /// Build an error result with a plain-text message.
    #[must_use]
    pub fn error(msg: &str) -> CallToolResult {
        CallToolResult {
            content: vec![Content::text(msg)],
            structured_content: None,
            is_error: Some(true),
            meta: None,
        }
    }

    /// Build a successful result containing plain text.
    #[must_use]
    pub fn text(msg: &str) -> CallToolResult {
        CallToolResult {
            content: vec![Content::text(msg)],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        }
    }

    /// Build a successful result with pretty-printed JSON as text.
    ///
    /// Unlike [`success`](Self::success), which produces compact JSON,
    /// this variant uses [`serde_json::to_string_pretty`] for
    /// human-readable output.
    #[must_use]
    pub fn json_text(value: &serde_json::Value) -> CallToolResult {
        let text = serde_json::to_string_pretty(value).unwrap_or_else(|e| e.to_string());
        CallToolResult {
            content: vec![Content::text(text)],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // Helper to extract the text string from the first content block.
    fn first_text(result: &CallToolResult) -> &str {
        result.content[0]
            .raw
            .as_text()
            .expect("expected text content")
            .text
            .as_str()
    }

    // --- ToolResponse::success ---

    #[test]
    fn success_creates_non_error_result() {
        let result = ToolResponse::success(&json!({"key": "value"}));
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn success_serialises_json_compactly() {
        let value = json!({"count": 42});
        let result = ToolResponse::success(&value);
        assert_eq!(result.content.len(), 1);
        assert_eq!(first_text(&result), r#"{"count":42}"#);
    }

    #[test]
    fn success_with_array_value() {
        let value = json!([1, 2, 3]);
        let result = ToolResponse::success(&value);
        assert_eq!(first_text(&result), "[1,2,3]");
    }

    // --- ToolResponse::error ---

    #[test]
    fn error_creates_error_result() {
        let result = ToolResponse::error("something broke");
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn error_contains_message() {
        let result = ToolResponse::error("bad input");
        assert_eq!(first_text(&result), "bad input");
    }

    // --- ToolResponse::text ---

    #[test]
    fn text_creates_non_error_result() {
        let result = ToolResponse::text("hello");
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn text_contains_plain_message() {
        let result = ToolResponse::text("plain output");
        assert_eq!(first_text(&result), "plain output");
    }

    // --- ToolResponse::json_text ---

    #[test]
    fn json_text_creates_non_error_result() {
        let result = ToolResponse::json_text(&json!({"x": 1}));
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn json_text_pretty_prints() {
        let value = json!({"a": 1});
        let result = ToolResponse::json_text(&value);
        let expected = serde_json::to_string_pretty(&value).unwrap();
        assert_eq!(first_text(&result), expected);
    }

    #[test]
    fn json_text_differs_from_success() {
        let value = json!({"nested": {"key": "value"}});
        let compact = ToolResponse::success(&value);
        let pretty = ToolResponse::json_text(&value);

        let compact_text = first_text(&compact);
        let pretty_text = first_text(&pretty);

        // Pretty output should contain newlines; compact should not.
        assert!(!compact_text.contains('\n'));
        assert!(pretty_text.contains('\n'));
    }
}
