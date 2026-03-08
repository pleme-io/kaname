//! Hashira (柱) --- MCP server scaffold.
//!
//! Extracts the common boilerplate from karakuri and hikyaku's MCP servers:
//! tool registration, response formatting, and rmcp type re-exports.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use hashira::{ToolRegistry, ToolResponse, McpServerInfo};
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

mod response;
mod server;

pub use response::ToolResponse;
pub use server::{McpServerInfo, McpTool, ToolRegistry};

pub use rmcp;
pub use schemars;
