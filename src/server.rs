use std::collections::HashMap;
use std::sync::Arc;

/// Metadata describing an MCP server.
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

impl McpServerInfo {
    /// Create a new server info descriptor.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
        }
    }
}

/// A single MCP tool definition with its JSON Schema.
#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
}

/// A registry that collects MCP tool definitions.
///
/// Use this to declare all tools a server supports, then retrieve
/// them for advertisement during the MCP `initialize` handshake.
#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, McpTool>,
    insertion_order: Vec<String>,
}

impl ToolRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool with a name, description, and JSON Schema for its input.
    ///
    /// If a tool with the same name already exists, it is replaced
    /// (the insertion-order position is preserved).
    pub fn register(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        schema: serde_json::Value,
    ) {
        let name = name.into();
        if !self.tools.contains_key(&name) {
            self.insertion_order.push(name.clone());
        }
        self.tools.insert(
            name.clone(),
            McpTool {
                name,
                description: description.into(),
                schema,
            },
        );
    }

    /// Get all registered tools in insertion order.
    #[must_use]
    pub fn tools(&self) -> Vec<&McpTool> {
        self.insertion_order
            .iter()
            .filter_map(|name| self.tools.get(name))
            .collect()
    }

    /// Look up a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&McpTool> {
        self.tools.get(name)
    }

    /// Return the number of registered tools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Return `true` if no tools are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Convert every registered tool into the rmcp [`Tool`](rmcp::model::Tool)
    /// type, preserving insertion order.
    ///
    /// The resulting vector is ready to be placed inside a
    /// `ListToolsResult` during the MCP handshake.
    #[must_use]
    pub fn to_tool_list(&self) -> Vec<rmcp::model::Tool> {
        self.tools()
            .into_iter()
            .map(|t| {
                let input_schema = match &t.schema {
                    serde_json::Value::Object(map) => Arc::new(map.clone()),
                    other => {
                        let mut map = serde_json::Map::new();
                        map.insert("type".to_string(), serde_json::json!("object"));
                        map.insert("properties".to_string(), other.clone());
                        Arc::new(map)
                    }
                };
                rmcp::model::Tool::new(t.name.clone(), t.description.clone(), input_schema)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // ---- McpServerInfo ----

    #[test]
    fn server_info_stores_fields() {
        let info = McpServerInfo::new("test-server", "1.0.0", "A test server");
        assert_eq!(info.name, "test-server");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "A test server");
    }

    #[test]
    fn server_info_accepts_string_types() {
        let info = McpServerInfo::new(
            String::from("owned"),
            String::from("2.0.0"),
            String::from("owned desc"),
        );
        assert_eq!(info.name, "owned");
    }

    // ---- ToolRegistry basics ----

    #[test]
    fn new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.tools().is_empty());
    }

    #[test]
    fn register_and_get_roundtrip() {
        let mut registry = ToolRegistry::new();
        registry.register(
            "greet",
            "Say hello",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            }),
        );

        let tool = registry.get("greet").expect("tool should exist");
        assert_eq!(tool.name, "greet");
        assert_eq!(tool.description, "Say hello");
    }

    #[test]
    fn get_missing_returns_none() {
        let registry = ToolRegistry::new();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn tools_returns_all_in_insertion_order() {
        let mut registry = ToolRegistry::new();
        registry.register("alpha", "First tool", json!({}));
        registry.register("beta", "Second tool", json!({}));
        registry.register("gamma", "Third tool", json!({}));

        let tools = registry.tools();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "alpha");
        assert_eq!(tools[1].name, "beta");
        assert_eq!(tools[2].name, "gamma");
    }

    #[test]
    fn len_tracks_registrations() {
        let mut registry = ToolRegistry::new();
        assert_eq!(registry.len(), 0);
        registry.register("a", "A", json!({}));
        assert_eq!(registry.len(), 1);
        registry.register("b", "B", json!({}));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn is_empty_becomes_false_after_register() {
        let mut registry = ToolRegistry::new();
        assert!(registry.is_empty());
        registry.register("x", "X", json!({}));
        assert!(!registry.is_empty());
    }

    // ---- duplicate / overwrite ----

    #[test]
    fn duplicate_name_overwrites_description() {
        let mut registry = ToolRegistry::new();
        registry.register("tool", "v1", json!({}));
        registry.register("tool", "v2", json!({}));

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("tool").unwrap().description, "v2");
    }

    #[test]
    fn duplicate_preserves_insertion_order_position() {
        let mut registry = ToolRegistry::new();
        registry.register("first", "F", json!({}));
        registry.register("second", "S", json!({}));
        // Overwrite "first" — it should keep its original position.
        registry.register("first", "F2", json!({}));

        let tools = registry.tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "first");
        assert_eq!(tools[0].description, "F2");
        assert_eq!(tools[1].name, "second");
    }

    // ---- schema storage ----

    #[test]
    fn mcp_tool_stores_schema() {
        let schema = json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": { "type": "string" }
            }
        });
        let mut registry = ToolRegistry::new();
        registry.register("search", "Search things", schema.clone());

        let tool = registry.get("search").unwrap();
        assert_eq!(tool.schema, schema);
    }

    // ---- to_tool_list ----

    #[test]
    fn to_tool_list_empty_registry() {
        let registry = ToolRegistry::new();
        let list = registry.to_tool_list();
        assert!(list.is_empty());
    }

    #[test]
    fn to_tool_list_converts_all_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(
            "search",
            "Search things",
            json!({"type": "object", "properties": {"q": {"type": "string"}}}),
        );
        registry.register(
            "list",
            "List items",
            json!({"type": "object", "properties": {}}),
        );

        let list = registry.to_tool_list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name.as_ref(), "search");
        assert_eq!(
            list[0].description.as_deref(),
            Some("Search things")
        );
        assert_eq!(list[1].name.as_ref(), "list");
    }

    #[test]
    fn to_tool_list_preserves_insertion_order() {
        let mut registry = ToolRegistry::new();
        registry.register("c", "C", json!({"type": "object"}));
        registry.register("a", "A", json!({"type": "object"}));
        registry.register("b", "B", json!({"type": "object"}));

        let list = registry.to_tool_list();
        let names: Vec<&str> = list.iter().map(|t| t.name.as_ref()).collect();
        assert_eq!(names, vec!["c", "a", "b"]);
    }

    #[test]
    fn to_tool_list_schema_roundtrips() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });
        let mut registry = ToolRegistry::new();
        registry.register("echo", "Echo", schema.clone());

        let list = registry.to_tool_list();
        let rmcp_schema = list[0].schema_as_json_value();
        assert_eq!(rmcp_schema, schema);
    }

    // ---- McpServerInfo trait impls ----

    #[test]
    fn server_info_is_cloneable() {
        let info = McpServerInfo::new("s", "1.0", "d");
        let cloned = info.clone();
        assert_eq!(cloned.name, "s");
        assert_eq!(cloned.version, "1.0");
        assert_eq!(cloned.description, "d");
    }

    #[test]
    fn server_info_debug_contains_fields() {
        let info = McpServerInfo::new("myserver", "2.0.0", "my desc");
        let debug = format!("{info:?}");
        assert!(debug.contains("myserver"));
        assert!(debug.contains("2.0.0"));
        assert!(debug.contains("my desc"));
    }

    #[test]
    fn server_info_with_empty_strings() {
        let info = McpServerInfo::new("", "", "");
        assert_eq!(info.name, "");
        assert_eq!(info.version, "");
        assert_eq!(info.description, "");
    }

    #[test]
    fn server_info_with_unicode() {
        let info = McpServerInfo::new(
            "\u{8981}",         // kanji for "kaname"
            "0.1.0",
            "\u{30b5}\u{30fc}\u{30d0}\u{30fc}", // "server" in katakana
        );
        assert_eq!(info.name, "\u{8981}");
        assert_eq!(info.description, "\u{30b5}\u{30fc}\u{30d0}\u{30fc}");
    }

    // ---- McpTool ----

    #[test]
    fn mcp_tool_debug_contains_name() {
        let mut registry = ToolRegistry::new();
        registry.register("dbg_tool", "Debug test", json!({}));
        let tool = registry.get("dbg_tool").unwrap();
        let debug = format!("{tool:?}");
        assert!(debug.contains("dbg_tool"));
    }

    #[test]
    fn mcp_tool_clone_is_independent() {
        let mut registry = ToolRegistry::new();
        registry.register("orig", "Original", json!({"type": "object"}));
        let tool = registry.get("orig").unwrap().clone();
        // Mutate registry; cloned tool is unaffected.
        registry.register("orig", "Replaced", json!({}));
        assert_eq!(tool.description, "Original");
    }

    // ---- ToolRegistry: Default ----

    #[test]
    fn default_registry_equals_new() {
        let from_new = ToolRegistry::new();
        let from_default = ToolRegistry::default();
        assert_eq!(from_new.len(), from_default.len());
        assert!(from_default.is_empty());
    }

    // ---- ToolRegistry: Clone ----

    #[test]
    fn cloned_registry_is_independent() {
        let mut original = ToolRegistry::new();
        original.register("a", "A", json!({}));
        let cloned = original.clone();

        // Mutate original -- clone should be unaffected.
        original.register("b", "B", json!({}));
        assert_eq!(cloned.len(), 1);
        assert_eq!(original.len(), 2);
    }

    #[test]
    fn cloned_registry_preserves_order() {
        let mut registry = ToolRegistry::new();
        registry.register("x", "X", json!({}));
        registry.register("y", "Y", json!({}));
        let cloned = registry.clone();
        let names: Vec<&str> = cloned.tools().iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["x", "y"]);
    }

    // ---- ToolRegistry: Debug ----

    #[test]
    fn registry_debug_does_not_panic() {
        let mut registry = ToolRegistry::new();
        registry.register("tool1", "T1", json!({"type": "object"}));
        let debug = format!("{registry:?}");
        assert!(debug.contains("tool1"));
    }

    // ---- ToolRegistry: multiple overwrites ----

    #[test]
    fn triple_overwrite_keeps_single_entry() {
        let mut registry = ToolRegistry::new();
        registry.register("tool", "v1", json!({"v": 1}));
        registry.register("tool", "v2", json!({"v": 2}));
        registry.register("tool", "v3", json!({"v": 3}));

        assert_eq!(registry.len(), 1);
        let tool = registry.get("tool").unwrap();
        assert_eq!(tool.description, "v3");
        assert_eq!(tool.schema, json!({"v": 3}));
    }

    // ---- ToolRegistry: empty name / description ----

    #[test]
    fn register_with_empty_name() {
        let mut registry = ToolRegistry::new();
        registry.register("", "Empty name tool", json!({}));
        assert_eq!(registry.len(), 1);
        let tool = registry.get("").unwrap();
        assert_eq!(tool.name, "");
    }

    #[test]
    fn register_with_empty_description() {
        let mut registry = ToolRegistry::new();
        registry.register("tool", "", json!({}));
        let tool = registry.get("tool").unwrap();
        assert_eq!(tool.description, "");
    }

    // ---- to_tool_list: non-object schema wrapping ----

    #[test]
    fn to_tool_list_wraps_non_object_schema_as_properties() {
        // When the schema is not a JSON object (e.g., an array or string),
        // to_tool_list should wrap it in a {"type":"object","properties": ...} envelope.
        let mut registry = ToolRegistry::new();
        registry.register("weird", "Weird schema", json!("not an object"));

        let list = registry.to_tool_list();
        let schema = list[0].schema_as_json_value();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"], json!("not an object"));
    }

    #[test]
    fn to_tool_list_wraps_array_schema() {
        let mut registry = ToolRegistry::new();
        registry.register("arr", "Array schema", json!([1, 2, 3]));

        let list = registry.to_tool_list();
        let schema = list[0].schema_as_json_value();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"], json!([1, 2, 3]));
    }

    #[test]
    fn to_tool_list_wraps_null_schema() {
        let mut registry = ToolRegistry::new();
        registry.register("nul", "Null schema", json!(null));

        let list = registry.to_tool_list();
        let schema = list[0].schema_as_json_value();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"], json!(null));
    }

    #[test]
    fn to_tool_list_passes_through_object_schema_with_extra_fields() {
        // An object schema with additional fields (required, additionalProperties)
        // should be preserved as-is.
        let schema = json!({
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string" }
            },
            "additionalProperties": false
        });
        let mut registry = ToolRegistry::new();
        registry.register("strict", "Strict schema", schema.clone());

        let list = registry.to_tool_list();
        let rmcp_schema = list[0].schema_as_json_value();
        assert_eq!(rmcp_schema, schema);
    }

    // ---- to_tool_list: tool description ----

    #[test]
    fn to_tool_list_carries_description() {
        let mut registry = ToolRegistry::new();
        registry.register("desc_test", "A detailed description", json!({"type": "object"}));
        let list = registry.to_tool_list();
        assert_eq!(
            list[0].description.as_deref(),
            Some("A detailed description")
        );
    }

    // ---- to_tool_list: large registry ----

    #[test]
    fn to_tool_list_handles_many_tools() {
        let mut registry = ToolRegistry::new();
        for i in 0..100 {
            registry.register(
                format!("tool_{i}"),
                format!("Tool number {i}"),
                json!({"type": "object"}),
            );
        }
        let list = registry.to_tool_list();
        assert_eq!(list.len(), 100);
        // First and last should be in order.
        assert_eq!(list[0].name.as_ref(), "tool_0");
        assert_eq!(list[99].name.as_ref(), "tool_99");
    }

    // ---- get after overwrite ----

    #[test]
    fn get_returns_latest_after_overwrite() {
        let mut registry = ToolRegistry::new();
        registry.register("t", "old", json!({"old": true}));
        registry.register("t", "new", json!({"new": true}));
        let tool = registry.get("t").unwrap();
        assert_eq!(tool.description, "new");
        assert_eq!(tool.schema, json!({"new": true}));
    }

    // ---- tools() with single entry ----

    #[test]
    fn tools_returns_single_entry_correctly() {
        let mut registry = ToolRegistry::new();
        registry.register("only", "The only one", json!({"type": "object"}));
        let tools = registry.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "only");
    }
}
