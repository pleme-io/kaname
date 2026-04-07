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
    /// Build a [`CallToolResult`] with a single text content block.
    fn build(text: impl Into<String>, is_error: bool) -> CallToolResult {
        CallToolResult {
            content: vec![Content::text(text.into())],
            structured_content: None,
            is_error: Some(is_error),
            meta: None,
        }
    }

    /// Build a successful JSON result.
    ///
    /// The value is serialised with [`serde_json::to_string`] (compact)
    /// and wrapped in a single text content block.
    #[must_use]
    pub fn success(value: &serde_json::Value) -> CallToolResult {
        let text = serde_json::to_string(value).unwrap_or_else(|e| e.to_string());
        Self::build(text, false)
    }

    /// Build an error result with a plain-text message.
    #[must_use]
    pub fn error(msg: impl AsRef<str>) -> CallToolResult {
        Self::build(msg.as_ref(), true)
    }

    /// Build a successful result containing plain text.
    #[must_use]
    pub fn text(msg: impl AsRef<str>) -> CallToolResult {
        Self::build(msg.as_ref(), false)
    }

    /// Build a successful result by serialising any [`Serialize`](serde::Serialize) value.
    ///
    /// Unlike [`success`](Self::success), which takes a pre-built `serde_json::Value`,
    /// this method accepts any serialisable type directly.
    ///
    /// # Errors
    ///
    /// Returns [`KanameError::Json`](crate::KanameError::Json) if serialisation fails.
    pub fn from_serialize(value: &impl serde::Serialize) -> Result<CallToolResult, crate::KanameError> {
        let v = serde_json::to_value(value)?;
        Ok(Self::success(&v))
    }

    /// Build an error result from any [`Display`](std::fmt::Display) value.
    ///
    /// Convenience shorthand for `ToolResponse::error(&e.to_string())`.
    #[must_use]
    pub fn from_error(error: &impl std::fmt::Display) -> CallToolResult {
        Self::error(error.to_string())
    }

    /// Build a successful result with pretty-printed JSON as text.
    ///
    /// Unlike [`success`](Self::success), which produces compact JSON,
    /// this variant uses [`serde_json::to_string_pretty`] for
    /// human-readable output.
    #[must_use]
    pub fn json_text(value: &serde_json::Value) -> CallToolResult {
        let text = serde_json::to_string_pretty(value).unwrap_or_else(|e| e.to_string());
        Self::build(text, false)
    }
}

/// Build a successful [`CallToolResult`] from any serialisable value.
///
/// Shorthand for `ToolResponse::success(&serde_json::to_value(v)?)` that
/// accepts any `Serialize` implementor directly.
pub fn json_ok(value: &impl serde::Serialize) -> Result<CallToolResult, crate::KanameError> {
    let v = serde_json::to_value(value)?;
    Ok(ToolResponse::success(&v))
}

/// Build an error [`CallToolResult`] from any [`std::fmt::Display`] value.
///
/// Shorthand for `ToolResponse::error(&e.to_string())`.
#[must_use]
pub fn json_err(error: &impl std::fmt::Display) -> CallToolResult {
    ToolResponse::error(error.to_string())
}

/// Convert a `Result<T, E>` into a [`CallToolResult`].
///
/// On `Ok(v)`, serialises `v` as compact JSON (like [`json_ok`]).
/// On `Err(e)`, formats the error as a plain-text error result (like [`json_err`]).
pub fn json_result<T: serde::Serialize, E: std::fmt::Display>(
    result: Result<T, E>,
) -> Result<CallToolResult, crate::KanameError> {
    match result {
        Ok(v) => json_ok(&v),
        Err(e) => Ok(json_err(&e)),
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

    // --- ToolResponse::success edge cases ---

    #[test]
    fn success_with_null_value() {
        let result = ToolResponse::success(&json!(null));
        assert_eq!(first_text(&result), "null");
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn success_with_boolean_value() {
        let result = ToolResponse::success(&json!(true));
        assert_eq!(first_text(&result), "true");
    }

    #[test]
    fn success_with_string_value() {
        let result = ToolResponse::success(&json!("hello world"));
        assert_eq!(first_text(&result), r#""hello world""#);
    }

    #[test]
    fn success_with_number_value() {
        let result = ToolResponse::success(&json!(3.14));
        assert_eq!(first_text(&result), "3.14");
    }

    #[test]
    fn success_with_empty_object() {
        let result = ToolResponse::success(&json!({}));
        assert_eq!(first_text(&result), "{}");
    }

    #[test]
    fn success_with_empty_array() {
        let result = ToolResponse::success(&json!([]));
        assert_eq!(first_text(&result), "[]");
    }

    #[test]
    fn success_with_deeply_nested_object() {
        let value = json!({"a": {"b": {"c": {"d": 42}}}});
        let result = ToolResponse::success(&value);
        // Compact serialisation -- no whitespace between structural chars.
        let text = first_text(&result);
        assert!(!text.contains(' '));
        assert!(text.contains("42"));
    }

    // --- structural invariants ---

    #[test]
    fn success_has_exactly_one_content_block() {
        let result = ToolResponse::success(&json!({"k": "v"}));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn error_has_exactly_one_content_block() {
        let result = ToolResponse::error("oops");
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn text_has_exactly_one_content_block() {
        let result = ToolResponse::text("msg");
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn json_text_has_exactly_one_content_block() {
        let result = ToolResponse::json_text(&json!({"k": 1}));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn success_has_no_structured_content() {
        let result = ToolResponse::success(&json!({}));
        assert!(result.structured_content.is_none());
    }

    #[test]
    fn error_has_no_structured_content() {
        let result = ToolResponse::error("err");
        assert!(result.structured_content.is_none());
    }

    #[test]
    fn text_has_no_structured_content() {
        let result = ToolResponse::text("t");
        assert!(result.structured_content.is_none());
    }

    #[test]
    fn success_has_no_meta() {
        let result = ToolResponse::success(&json!({}));
        assert!(result.meta.is_none());
    }

    #[test]
    fn error_has_no_meta() {
        let result = ToolResponse::error("err");
        assert!(result.meta.is_none());
    }

    // --- ToolResponse::error edge cases ---

    #[test]
    fn error_with_empty_string() {
        let result = ToolResponse::error("");
        assert_eq!(first_text(&result), "");
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn error_with_multiline_message() {
        let result = ToolResponse::error("line1\nline2\nline3");
        assert_eq!(first_text(&result), "line1\nline2\nline3");
    }

    #[test]
    fn error_with_unicode_message() {
        let result = ToolResponse::error("fehler: ungueltige eingabe \u{00e4}\u{00f6}\u{00fc}");
        assert!(first_text(&result).contains('\u{00e4}'));
        assert_eq!(result.is_error, Some(true));
    }

    // --- ToolResponse::text edge cases ---

    #[test]
    fn text_with_empty_string() {
        let result = ToolResponse::text("");
        assert_eq!(first_text(&result), "");
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn text_preserves_whitespace() {
        let result = ToolResponse::text("  leading and trailing  ");
        assert_eq!(first_text(&result), "  leading and trailing  ");
    }

    // --- ToolResponse::json_text edge cases ---

    #[test]
    fn json_text_with_array() {
        let value = json!([1, "two", null, true]);
        let result = ToolResponse::json_text(&value);
        let expected = serde_json::to_string_pretty(&value).unwrap();
        assert_eq!(first_text(&result), expected);
    }

    #[test]
    fn json_text_with_scalar() {
        // Even scalars should round-trip through pretty-print.
        let result = ToolResponse::json_text(&json!(42));
        assert_eq!(first_text(&result), "42");
        assert_eq!(result.is_error, Some(false));
    }

    // --- ToolResponse::from_error ---

    #[test]
    fn from_error_with_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let result = ToolResponse::from_error(&io_err);
        assert_eq!(result.is_error, Some(true));
        assert!(first_text(&result).contains("not found"));
    }

    #[test]
    fn from_error_with_string() {
        let result = ToolResponse::from_error(&"plain error");
        assert_eq!(first_text(&result), "plain error");
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn from_error_matches_json_err() {
        let msg = "something";
        let from_method = ToolResponse::from_error(&msg);
        let from_fn = json_err(&msg);
        assert_eq!(first_text(&from_method), first_text(&from_fn));
    }

    // --- ToolResponse::from_serialize ---

    #[test]
    fn from_serialize_with_struct() {
        #[derive(serde::Serialize)]
        struct Info { name: String }
        let result = ToolResponse::from_serialize(&Info { name: "test".into() }).unwrap();
        assert_eq!(first_text(&result), r#"{"name":"test"}"#);
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn from_serialize_with_vec() {
        let result = ToolResponse::from_serialize(&vec![1, 2, 3]).unwrap();
        assert_eq!(first_text(&result), "[1,2,3]");
    }

    #[test]
    fn from_serialize_matches_json_ok() {
        let v = json!({"a": 1});
        let from_serialize = ToolResponse::from_serialize(&v).unwrap();
        let from_ok = json_ok(&v).unwrap();
        assert_eq!(first_text(&from_serialize), first_text(&from_ok));
    }

    // --- AsRef<str> acceptance ---

    #[test]
    fn error_accepts_owned_string() {
        let msg = String::from("owned error");
        let result = ToolResponse::error(msg);
        assert_eq!(first_text(&result), "owned error");
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn text_accepts_owned_string() {
        let msg = String::from("owned text");
        let result = ToolResponse::text(msg);
        assert_eq!(first_text(&result), "owned text");
        assert_eq!(result.is_error, Some(false));
    }

    // --- cross-method consistency ---

    #[test]
    fn success_and_text_differ_for_json_looking_string() {
        // success() wraps a JSON value; text() wraps a literal string.
        let json_str = r#"{"key":"value"}"#;
        let success_result = ToolResponse::success(&json!({"key": "value"}));
        let text_result = ToolResponse::text(json_str);

        // Both should produce the same text content.
        assert_eq!(first_text(&success_result), first_text(&text_result));
        // Both are non-error.
        assert_eq!(success_result.is_error, text_result.is_error);
    }

    #[test]
    fn error_and_text_share_same_text_but_differ_on_is_error() {
        let msg = "some message";
        let err = ToolResponse::error(msg);
        let txt = ToolResponse::text(msg);

        assert_eq!(first_text(&err), first_text(&txt));
        assert_eq!(err.is_error, Some(true));
        assert_eq!(txt.is_error, Some(false));
    }

    // --- json_ok ---

    #[test]
    fn json_ok_with_object() {
        let result = json_ok(&json!({"status": "ok"})).unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(first_text(&result), r#"{"status":"ok"}"#);
    }

    #[test]
    fn json_ok_with_typed_struct() {
        #[derive(serde::Serialize)]
        struct Greeting {
            message: String,
        }
        let g = Greeting {
            message: "hi".to_string(),
        };
        let result = json_ok(&g).unwrap();
        assert_eq!(first_text(&result), r#"{"message":"hi"}"#);
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn json_ok_with_scalar() {
        let result = json_ok(&42_i32).unwrap();
        assert_eq!(first_text(&result), "42");
    }

    #[test]
    fn json_ok_with_vec() {
        let result = json_ok(&vec![1, 2, 3]).unwrap();
        assert_eq!(first_text(&result), "[1,2,3]");
    }

    #[test]
    fn json_ok_matches_success_for_json_values() {
        let v = json!({"a": 1});
        let from_ok = json_ok(&v).unwrap();
        let from_success = ToolResponse::success(&v);
        assert_eq!(first_text(&from_ok), first_text(&from_success));
        assert_eq!(from_ok.is_error, from_success.is_error);
    }

    // --- json_err ---

    #[test]
    fn json_err_marks_as_error() {
        let result = json_err(&"bad input");
        assert_eq!(result.is_error, Some(true));
        assert_eq!(first_text(&result), "bad input");
    }

    #[test]
    fn json_err_with_error_type() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let result = json_err(&io_err);
        assert_eq!(result.is_error, Some(true));
        assert!(first_text(&result).contains("not found"));
    }

    #[test]
    fn json_err_with_kaname_error() {
        let err = crate::KanameError::ConfigKeyNotFound {
            key: "missing".to_string(),
        };
        let result = json_err(&err);
        assert_eq!(result.is_error, Some(true));
        assert!(first_text(&result).contains("missing"));
    }

    #[test]
    fn json_err_matches_error_for_strings() {
        let msg = "something broke";
        let from_err_fn = json_err(&msg);
        let from_method = ToolResponse::error(msg);
        assert_eq!(first_text(&from_err_fn), first_text(&from_method));
        assert_eq!(from_err_fn.is_error, from_method.is_error);
    }

    // --- json_result ---

    #[test]
    fn json_result_ok_path() {
        let r: Result<_, String> = Ok(json!({"x": 1}));
        let result = json_result(r).unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(first_text(&result), r#"{"x":1}"#);
    }

    #[test]
    fn json_result_err_path() {
        let r: Result<serde_json::Value, _> = Err("boom");
        let result = json_result(r).unwrap();
        assert_eq!(result.is_error, Some(true));
        assert_eq!(first_text(&result), "boom");
    }

    #[test]
    fn json_result_with_typed_ok() {
        #[derive(serde::Serialize)]
        struct Count {
            n: u32,
        }
        let r: Result<Count, String> = Ok(Count { n: 5 });
        let result = json_result(r).unwrap();
        assert_eq!(first_text(&result), r#"{"n":5}"#);
    }

    #[test]
    fn json_result_with_io_error() {
        let r: Result<i32, std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "disk full"));
        let result = json_result(r).unwrap();
        assert_eq!(result.is_error, Some(true));
        assert!(first_text(&result).contains("disk full"));
    }
}
