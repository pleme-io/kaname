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
}
