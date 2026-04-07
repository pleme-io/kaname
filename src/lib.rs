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
