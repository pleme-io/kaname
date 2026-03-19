//! Shared MCP tool input types and registration helpers for app config management.
//!
//! Eliminates the `ConfigGet`/`ConfigSet` boilerplate duplicated across GPU apps.
//!
//! # Usage
//!
//! ```rust
//! use kaname::{ToolRegistry, register_config_tools, ConfigGetInput, ConfigSetInput};
//!
//! let mut registry = ToolRegistry::new();
//! register_config_tools(&mut registry);
//! assert_eq!(registry.len(), 3); // config_get, config_set, status
//! ```

use serde::Deserialize;

/// Input for the `config_get` MCP tool.
///
/// Shared across all pleme-io apps that expose configuration via MCP.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct ConfigGetInput {
    /// Config key to retrieve (dot-separated path, e.g. `appearance.font_size`).
    /// Omit to get all configuration.
    #[schemars(description = "Config key to retrieve (dot-separated path). Omit for all config.")]
    pub key: Option<String>,
}

/// Input for the `config_set` MCP tool.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct ConfigSetInput {
    /// Config key to set (dot-separated path).
    #[schemars(description = "Config key to set (dot-separated path)")]
    pub key: String,
    /// New value as a JSON string.
    #[schemars(description = "New value as JSON string")]
    pub value: String,
}

/// Input for the `status` MCP tool (common across apps).
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct StatusInput {
    /// Optional section to query (e.g. `cpu`, `memory`, `connections`).
    #[schemars(description = "Optional status section to query")]
    pub section: Option<String>,
}

/// Register standard config management tools in a [`ToolRegistry`](crate::ToolRegistry).
///
/// Registers:
/// - `config_get` — retrieve configuration values
/// - `config_set` — update configuration values
/// - `status` — query application status
///
/// Apps call this to get all three tools, then add app-specific tools on top.
pub fn register_config_tools(registry: &mut crate::ToolRegistry) {
    registry.register(
        "config_get",
        "Get configuration value(s). Omit key to retrieve all config.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Config key (dot-separated path). Omit for all config."
                }
            }
        }),
    );

    registry.register(
        "config_set",
        "Set a configuration value.",
        serde_json::json!({
            "type": "object",
            "required": ["key", "value"],
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Config key to set (dot-separated path)"
                },
                "value": {
                    "type": "string",
                    "description": "New value as JSON string"
                }
            }
        }),
    );

    registry.register(
        "status",
        "Get application status.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "section": {
                    "type": "string",
                    "description": "Optional status section to query"
                }
            }
        }),
    );
}

/// Trait for apps to implement config get/set logic.
///
/// Enables mockable, testable config handling. Each GPU app implements this
/// with its own configuration type (typically backed by shikumi).
pub trait ConfigHandler: Send + Sync {
    /// Get a config value by key, or all config as JSON if key is `None`.
    fn get(&self, key: Option<&str>) -> Result<serde_json::Value, String>;
    /// Set a config value by key.
    fn set(&mut self, key: &str, value: &str) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ToolRegistry;

    // ---- ConfigGetInput deserialization ----

    #[test]
    fn config_get_input_deserializes_with_key() {
        let input: ConfigGetInput =
            serde_json::from_value(json!({"key": "appearance.font_size"})).unwrap();
        assert_eq!(input.key.as_deref(), Some("appearance.font_size"));
    }

    #[test]
    fn config_get_input_deserializes_without_key() {
        let input: ConfigGetInput = serde_json::from_value(json!({})).unwrap();
        assert!(input.key.is_none());
    }

    #[test]
    fn config_get_input_deserializes_with_null_key() {
        let input: ConfigGetInput = serde_json::from_value(json!({"key": null})).unwrap();
        assert!(input.key.is_none());
    }

    #[test]
    fn config_get_input_deserializes_with_empty_key() {
        let input: ConfigGetInput = serde_json::from_value(json!({"key": ""})).unwrap();
        assert_eq!(input.key.as_deref(), Some(""));
    }

    #[test]
    fn config_get_input_deserializes_with_nested_dot_path() {
        let input: ConfigGetInput =
            serde_json::from_value(json!({"key": "a.b.c.d.e"})).unwrap();
        assert_eq!(input.key.as_deref(), Some("a.b.c.d.e"));
    }

    // ---- ConfigSetInput deserialization ----

    #[test]
    fn config_set_input_deserializes_with_required_fields() {
        let input: ConfigSetInput =
            serde_json::from_value(json!({"key": "font_size", "value": "14"})).unwrap();
        assert_eq!(input.key, "font_size");
        assert_eq!(input.value, "14");
    }

    #[test]
    fn config_set_input_fails_without_key() {
        let result = serde_json::from_value::<ConfigSetInput>(json!({"value": "14"}));
        assert!(result.is_err());
    }

    #[test]
    fn config_set_input_fails_without_value() {
        let result = serde_json::from_value::<ConfigSetInput>(json!({"key": "font_size"}));
        assert!(result.is_err());
    }

    #[test]
    fn config_set_input_accepts_json_object_as_value_string() {
        let input: ConfigSetInput = serde_json::from_value(
            json!({"key": "appearance", "value": r#"{"theme":"dark","font_size":14}"#}),
        )
        .unwrap();
        assert_eq!(input.key, "appearance");
        assert!(input.value.contains("dark"));
    }

    #[test]
    fn config_set_input_with_empty_strings() {
        let input: ConfigSetInput =
            serde_json::from_value(json!({"key": "", "value": ""})).unwrap();
        assert_eq!(input.key, "");
        assert_eq!(input.value, "");
    }

    // ---- StatusInput deserialization ----

    #[test]
    fn status_input_deserializes_with_section() {
        let input: StatusInput =
            serde_json::from_value(json!({"section": "memory"})).unwrap();
        assert_eq!(input.section.as_deref(), Some("memory"));
    }

    #[test]
    fn status_input_deserializes_without_section() {
        let input: StatusInput = serde_json::from_value(json!({})).unwrap();
        assert!(input.section.is_none());
    }

    #[test]
    fn status_input_deserializes_with_null_section() {
        let input: StatusInput = serde_json::from_value(json!({"section": null})).unwrap();
        assert!(input.section.is_none());
    }

    // ---- register_config_tools ----

    #[test]
    fn register_config_tools_registers_exactly_three() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn register_config_tools_has_config_get() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        assert!(registry.get("config_get").is_some());
    }

    #[test]
    fn register_config_tools_has_config_set() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        assert!(registry.get("config_set").is_some());
    }

    #[test]
    fn register_config_tools_has_status() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        assert!(registry.get("status").is_some());
    }

    #[test]
    fn register_config_tools_preserves_insertion_order() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tools = registry.tools();
        assert_eq!(tools[0].name, "config_get");
        assert_eq!(tools[1].name, "config_set");
        assert_eq!(tools[2].name, "status");
    }

    #[test]
    fn config_get_schema_has_key_property() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("config_get").unwrap();
        assert_eq!(tool.schema["properties"]["key"]["type"], "string");
    }

    #[test]
    fn config_set_schema_requires_key_and_value() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("config_set").unwrap();
        let required = tool.schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("key")));
        assert!(required.contains(&json!("value")));
    }

    #[test]
    fn config_set_schema_has_key_and_value_properties() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("config_set").unwrap();
        assert_eq!(tool.schema["properties"]["key"]["type"], "string");
        assert_eq!(tool.schema["properties"]["value"]["type"], "string");
    }

    #[test]
    fn status_schema_has_section_property() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("status").unwrap();
        assert_eq!(tool.schema["properties"]["section"]["type"], "string");
    }

    #[test]
    fn config_get_has_correct_description() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("config_get").unwrap();
        assert!(tool.description.contains("config"));
    }

    #[test]
    fn config_set_has_correct_description() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("config_set").unwrap();
        assert!(tool.description.contains("Set"));
    }

    #[test]
    fn status_has_correct_description() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let tool = registry.get("status").unwrap();
        assert!(tool.description.contains("status"));
    }

    // ---- register_config_tools coexists with app-specific tools ----

    #[test]
    fn config_tools_coexist_with_app_specific_tools() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        registry.register(
            "play_track",
            "Play a music track",
            json!({"type": "object", "properties": {"track_id": {"type": "string"}}}),
        );
        assert_eq!(registry.len(), 4);
        assert!(registry.get("config_get").is_some());
        assert!(registry.get("play_track").is_some());
    }

    #[test]
    fn app_tools_registered_before_config_tools_preserved() {
        let mut registry = ToolRegistry::new();
        registry.register("custom_first", "Custom", json!({}));
        register_config_tools(&mut registry);
        let tools = registry.tools();
        assert_eq!(tools[0].name, "custom_first");
        assert_eq!(tools[1].name, "config_get");
        assert_eq!(registry.len(), 4);
    }

    // ---- to_tool_list integration ----

    #[test]
    fn config_tools_convert_to_tool_list() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        let list = registry.to_tool_list();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].name.as_ref(), "config_get");
        assert_eq!(list[1].name.as_ref(), "config_set");
        assert_eq!(list[2].name.as_ref(), "status");
    }

    // ---- ConfigHandler trait ----

    struct MockConfig {
        data: serde_json::Map<String, serde_json::Value>,
    }

    impl MockConfig {
        fn new() -> Self {
            Self {
                data: serde_json::Map::new(),
            }
        }
    }

    impl ConfigHandler for MockConfig {
        fn get(&self, key: Option<&str>) -> Result<serde_json::Value, String> {
            match key {
                None => Ok(serde_json::Value::Object(self.data.clone())),
                Some(k) => self
                    .data
                    .get(k)
                    .cloned()
                    .ok_or_else(|| format!("key not found: {k}")),
            }
        }

        fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
            let parsed: serde_json::Value =
                serde_json::from_str(value).map_err(|e| e.to_string())?;
            self.data.insert(key.to_string(), parsed);
            Ok(())
        }
    }

    #[test]
    fn config_handler_get_all_returns_empty_object() {
        let config = MockConfig::new();
        let result = config.get(None).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn config_handler_set_and_get_roundtrip() {
        let mut config = MockConfig::new();
        config.set("font_size", "14").unwrap();
        let value = config.get(Some("font_size")).unwrap();
        assert_eq!(value, json!(14));
    }

    #[test]
    fn config_handler_get_missing_key_returns_error() {
        let config = MockConfig::new();
        let result = config.get(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn config_handler_set_invalid_json_returns_error() {
        let mut config = MockConfig::new();
        let result = config.set("key", "not valid json {{{");
        assert!(result.is_err());
    }

    #[test]
    fn config_handler_set_complex_value() {
        let mut config = MockConfig::new();
        config
            .set("appearance", r#"{"theme":"dark","font_size":14}"#)
            .unwrap();
        let value = config.get(Some("appearance")).unwrap();
        assert_eq!(value["theme"], "dark");
        assert_eq!(value["font_size"], 14);
    }

    #[test]
    fn config_handler_overwrite_value() {
        let mut config = MockConfig::new();
        config.set("size", "10").unwrap();
        config.set("size", "20").unwrap();
        let value = config.get(Some("size")).unwrap();
        assert_eq!(value, json!(20));
    }

    // ---- Clone / Debug trait tests ----

    #[test]
    fn config_get_input_is_cloneable() {
        let input: ConfigGetInput =
            serde_json::from_value(json!({"key": "a.b"})).unwrap();
        let cloned = input.clone();
        assert_eq!(cloned.key, input.key);
    }

    #[test]
    fn config_set_input_is_cloneable() {
        let input: ConfigSetInput =
            serde_json::from_value(json!({"key": "a", "value": "1"})).unwrap();
        let cloned = input.clone();
        assert_eq!(cloned.key, input.key);
        assert_eq!(cloned.value, input.value);
    }

    #[test]
    fn status_input_is_cloneable() {
        let input: StatusInput =
            serde_json::from_value(json!({"section": "cpu"})).unwrap();
        let cloned = input.clone();
        assert_eq!(cloned.section, input.section);
    }

    #[test]
    fn config_get_input_debug_contains_key() {
        let input: ConfigGetInput =
            serde_json::from_value(json!({"key": "test_key"})).unwrap();
        let debug = format!("{input:?}");
        assert!(debug.contains("test_key"));
    }

    #[test]
    fn config_set_input_debug_contains_fields() {
        let input: ConfigSetInput =
            serde_json::from_value(json!({"key": "k", "value": "v"})).unwrap();
        let debug = format!("{input:?}");
        assert!(debug.contains("k"));
        assert!(debug.contains("v"));
    }

    // ---- JSON Schema generation ----

    #[test]
    fn config_get_input_generates_json_schema() {
        let schema = schemars::schema_for!(ConfigGetInput);
        let value = serde_json::to_value(&schema).unwrap();
        assert!(value["properties"]["key"].is_object());
    }

    #[test]
    fn config_set_input_generates_json_schema_with_required() {
        let schema = schemars::schema_for!(ConfigSetInput);
        let value = serde_json::to_value(&schema).unwrap();
        let required = value["required"].as_array().unwrap();
        assert!(required.contains(&json!("key")));
        assert!(required.contains(&json!("value")));
    }

    #[test]
    fn status_input_generates_json_schema() {
        let schema = schemars::schema_for!(StatusInput);
        let value = serde_json::to_value(&schema).unwrap();
        assert!(value["properties"]["section"].is_object());
    }

    // ---- double registration is idempotent ----

    #[test]
    fn double_registration_overwrites_cleanly() {
        let mut registry = ToolRegistry::new();
        register_config_tools(&mut registry);
        register_config_tools(&mut registry);
        // Should still be 3 tools (overwritten, not duplicated).
        assert_eq!(registry.len(), 3);
    }
}
