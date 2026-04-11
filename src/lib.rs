//! Kaname (要) --- MCP server scaffold.
//!
//! Extracts the common boilerplate from karakuri and hikyaku's MCP servers:
//! tool registration, response formatting, and rmcp type re-exports.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use kaname::{ToolRegistry, ToolResponse, McpServerInfo};
//!
//! let info = McpServerInfo::new("my-server", "0.1.0", "Does things");
//!
//! let mut registry = ToolRegistry::new();
//! registry.register("greet", "Say hello", serde_json::json!({
//!     "type": "object",
//!     "properties": { "name": { "type": "string" } }
//! }));
//!
//! // In a tool handler:
//! let response = ToolResponse::success(&serde_json::json!({"greeting": "hello"}));
//! let error = ToolResponse::error("something broke");
//! let text = ToolResponse::text("plain output");
//! ```

pub mod config_tools;
mod error;
mod response;
mod server;

pub use config_tools::{register_config_tools, ConfigGetInput, ConfigHandler, ConfigSetInput, StatusInput};
pub use error::KanameError;
pub use response::{ToolResponse, json_err, json_ok, json_result};
pub use server::{McpServerInfo, McpTool, ToolRegistry};

pub use rmcp;
pub use schemars;

#[cfg(test)]
mod integration_tests {
    use super::*;

    // ── ConfigHandler as trait object ────────────────────────────────

    struct InMemoryConfig {
        data: serde_json::Map<String, serde_json::Value>,
    }

    impl InMemoryConfig {
        fn new() -> Self {
            Self {
                data: serde_json::Map::new(),
            }
        }
    }

    impl ConfigHandler for InMemoryConfig {
        fn get(&self, key: Option<&str>) -> Result<serde_json::Value, KanameError> {
            match key {
                None => Ok(serde_json::Value::Object(self.data.clone())),
                Some(k) => self.data.get(k).cloned().ok_or_else(|| {
                    KanameError::ConfigKeyNotFound {
                        key: k.to_string(),
                    }
                }),
            }
        }

        fn set(&mut self, key: &str, value: &str) -> Result<(), KanameError> {
            let parsed: serde_json::Value = serde_json::from_str(value)?;
            self.data.insert(key.to_string(), parsed);
            Ok(())
        }
    }

    /// ConfigHandler works through a trait object (dyn dispatch).
    #[test]
    fn config_handler_as_trait_object() {
        let mut config: Box<dyn ConfigHandler> = Box::new(InMemoryConfig::new());
        config.set("theme", r#""dark""#).unwrap();
        let val = config.get(Some("theme")).unwrap();
        assert_eq!(val, serde_json::json!("dark"));
    }

    /// ConfigHandler get(None) returns all keys after multiple sets.
    #[test]
    fn config_handler_get_all_after_multiple_sets() {
        let mut config = InMemoryConfig::new();
        config.set("font_size", "14").unwrap();
        config.set("theme", r#""dark""#).unwrap();
        config.set("enabled", "true").unwrap();

        let all = config.get(None).unwrap();
        let obj = all.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert_eq!(obj["font_size"], serde_json::json!(14));
        assert_eq!(obj["theme"], serde_json::json!("dark"));
        assert_eq!(obj["enabled"], serde_json::json!(true));
    }

    /// ConfigHandler set with various JSON types.
    #[test]
    fn config_handler_set_various_json_types() {
        let mut config = InMemoryConfig::new();

        // Null
        config.set("nothing", "null").unwrap();
        assert_eq!(config.get(Some("nothing")).unwrap(), serde_json::json!(null));

        // Boolean
        config.set("flag", "false").unwrap();
        assert_eq!(config.get(Some("flag")).unwrap(), serde_json::json!(false));

        // Array
        config.set("list", "[1, 2, 3]").unwrap();
        assert_eq!(config.get(Some("list")).unwrap(), serde_json::json!([1, 2, 3]));

        // Nested object
        config.set("nested", r#"{"a": {"b": true}}"#).unwrap();
        assert_eq!(
            config.get(Some("nested")).unwrap(),
            serde_json::json!({"a": {"b": true}})
        );
    }

    // ── KanameError source trait ────────────────────────────────────

    /// KanameError::Json preserves the serde_json error source chain.
    #[test]
    fn kaname_error_json_has_source() {
        use std::error::Error;

        let bad: Result<serde_json::Value, _> = serde_json::from_str("not json");
        let serde_err = bad.unwrap_err();
        let err = KanameError::from(serde_err);
        // The source() should return Some for the wrapped serde_json error.
        assert!(err.source().is_some());
    }

    /// KanameError::Io preserves the I/O error source chain.
    #[test]
    fn kaname_error_io_has_source() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        let err = KanameError::from(io_err);
        assert!(err.source().is_some());
    }

    /// KanameError::ConfigKeyNotFound has no source (leaf error).
    #[test]
    fn kaname_error_config_key_no_source() {
        use std::error::Error;

        let err = KanameError::ConfigKeyNotFound {
            key: "missing".to_string(),
        };
        assert!(err.source().is_none());
    }

    // ── ToolRegistry interleaving register and register_tool ────────

    /// Mixing register() and register_tool() in the same registry.
    #[test]
    fn registry_mixed_registration_methods() {
        let mut registry = ToolRegistry::new();

        registry.register("via_register", "From register()", serde_json::json!({}));
        registry.register_tool(McpTool::new(
            "via_register_tool",
            "From register_tool()",
            serde_json::json!({"type": "object"}),
        ));
        registry.register("third", "Third tool", serde_json::json!({}));

        assert_eq!(registry.len(), 3);
        let names: Vec<&str> = registry.names().collect();
        assert_eq!(names, vec!["via_register", "via_register_tool", "third"]);
    }

    // ── McpTool Display with special characters ─────────────────────

    /// McpTool Display with unicode and special characters.
    #[test]
    fn mcp_tool_display_special_characters() {
        let tool = McpTool::new(
            "search_\u{691c}\u{7d22}",
            "Search: find items (advanced)",
            serde_json::json!({}),
        );
        let display = format!("{tool}");
        assert!(display.contains("\u{691c}\u{7d22}"));
        assert!(display.contains("Search: find items (advanced)"));
    }

    /// McpServerInfo Display with special characters.
    #[test]
    fn mcp_server_info_display_special_characters() {
        let info = McpServerInfo::new("server-\u{30b5}", "0.1.0-beta", "desc");
        let display = format!("{info}");
        assert_eq!(display, "server-\u{30b5} v0.1.0-beta");
    }

    // ── register_config_tools schema validity ───────────────────────

    /// All config tool schemas have "type": "object" at the top level.
    #[test]
    fn config_tool_schemas_are_all_objects() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);

        for tool in registry.iter() {
            assert_eq!(
                tool.schema["type"],
                serde_json::json!("object"),
                "Tool {} schema should have type=object",
                tool.name
            );
        }
    }

    /// All config tool schemas have a "properties" field.
    #[test]
    fn config_tool_schemas_have_properties() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);

        for tool in registry.iter() {
            assert!(
                tool.schema["properties"].is_object(),
                "Tool {} schema should have properties object",
                tool.name
            );
        }
    }

    // ── ConfigGetInput with extra fields ─────────────────────────────

    /// Deserializing ConfigGetInput with extra/unknown fields succeeds
    /// (serde default behavior is to ignore unknown fields).
    #[test]
    fn config_get_input_ignores_extra_fields() {
        let input: ConfigGetInput = serde_json::from_value(serde_json::json!({
            "key": "font_size",
            "unknown_field": "should be ignored",
            "another": 42
        }))
        .unwrap();
        assert_eq!(input.key.as_deref(), Some("font_size"));
    }

    /// Deserializing ConfigSetInput with extra fields succeeds.
    #[test]
    fn config_set_input_ignores_extra_fields() {
        let input: ConfigSetInput = serde_json::from_value(serde_json::json!({
            "key": "theme",
            "value": r#""dark""#,
            "extra": true
        }))
        .unwrap();
        assert_eq!(input.key, "theme");
    }

    // ── McpServerInfo granular inequality ────────────────────────────

    /// McpServerInfo inequality on each field independently.
    #[test]
    fn server_info_inequality_per_field() {
        let base = McpServerInfo::new("server", "1.0", "desc");

        let diff_name = McpServerInfo::new("other", "1.0", "desc");
        assert_ne!(base, diff_name);

        let diff_version = McpServerInfo::new("server", "2.0", "desc");
        assert_ne!(base, diff_version);

        let diff_desc = McpServerInfo::new("server", "1.0", "different");
        assert_ne!(base, diff_desc);
    }

    // ── ToolResponse::from_serialize with trivial types ───────────────

    /// from_serialize with a simple bool produces correct output.
    #[test]
    fn from_serialize_with_bool() {
        let result = ToolResponse::from_serialize(&true).unwrap();
        assert_eq!(result.is_error, Some(false));
        let text = result.content[0]
            .raw
            .as_text()
            .unwrap()
            .text
            .as_str();
        assert_eq!(text, "true");
    }

    /// from_serialize with Option::None produces "null".
    #[test]
    fn from_serialize_with_none() {
        let val: Option<i32> = None;
        let result = ToolResponse::from_serialize(&val).unwrap();
        let text = result.content[0]
            .raw
            .as_text()
            .unwrap()
            .text
            .as_str();
        assert_eq!(text, "null");
    }

    // ── json_result with complex types ──────────────────────────────

    /// json_result works with nested structures on the Ok path.
    #[test]
    fn json_result_nested_struct() {
        #[derive(serde::Serialize)]
        struct Outer {
            inner: Inner,
        }
        #[derive(serde::Serialize)]
        struct Inner {
            values: Vec<i32>,
        }

        let r: Result<Outer, String> = Ok(Outer {
            inner: Inner { values: vec![1, 2] },
        });
        let result = json_result(r).unwrap();
        assert_eq!(result.is_error, Some(false));

        let text = result.content[0]
            .raw
            .as_text()
            .unwrap()
            .text
            .as_str();
        assert!(text.contains("values"));
        assert!(text.contains("[1,2]"));
    }

    // ── Full workflow: register tools, convert, validate ─────────────

    /// Full workflow: register config tools plus app-specific tools,
    /// convert to rmcp tool list, verify all tools are present.
    #[test]
    fn full_workflow_config_plus_app_tools() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);

        // Add app-specific tools.
        registry.register(
            "play",
            "Play a track",
            serde_json::json!({
                "type": "object",
                "required": ["track_id"],
                "properties": {
                    "track_id": { "type": "string" }
                }
            }),
        );
        registry.register(
            "stop",
            "Stop playback",
            serde_json::json!({"type": "object", "properties": {}}),
        );

        // Verify counts.
        assert_eq!(registry.len(), 5);
        assert!(registry.contains("config_get"));
        assert!(registry.contains("config_set"));
        assert!(registry.contains("status"));
        assert!(registry.contains("play"));
        assert!(registry.contains("stop"));

        // Convert to rmcp list.
        let list = registry.to_tool_list();
        assert_eq!(list.len(), 5);

        // Verify insertion order preserved through to_tool_list.
        let names: Vec<&str> = list.iter().map(|t| t.name.as_ref()).collect();
        assert_eq!(
            names,
            vec!["config_get", "config_set", "status", "play", "stop"]
        );

        // Verify play tool schema has required field.
        let play_schema = list[3].schema_as_json_value();
        assert!(play_schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("track_id")));
    }
}

/// Convenience re-exports for common usage.
///
/// ```rust,ignore
/// use kaname::prelude::*;
/// ```
pub mod prelude {
    pub use crate::config_tools::{
        ConfigGetInput, ConfigHandler, ConfigSetInput, StatusInput, register_config_tools,
    };
    pub use crate::error::KanameError;
    pub use crate::response::{ToolResponse, json_err, json_ok, json_result};
    pub use crate::server::{McpServerInfo, McpTool, ToolRegistry};
}
