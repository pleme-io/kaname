# Kaname (要) -- MCP Server Scaffold

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


## Build & Test

```bash
cargo build          # compile
cargo test           # 71 unit tests
```

## Architecture

Extracts the common MCP server boilerplate from ayatsuri and hikyaku into reusable types: tool registration, response formatting, and rmcp type re-exports.

### Module Map

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Re-exports + rmcp/schemars re-exports |
| `src/server.rs` | `McpServerInfo`, `McpTool`, `ToolRegistry` (36 tests) |
| `src/response.rs` | `ToolResponse` -- success/error/text helpers wrapping `CallToolResult` (35 tests) |

### Key Types

- **`McpServerInfo`** -- server name, version, description metadata
- **`McpTool`** -- tool definition with name, description, JSON Schema
- **`ToolRegistry`** -- collects tools, preserves insertion order, converts to rmcp `Tool` list
- **`ToolResponse`** -- `success(&impl Serialize)`, `error(&str)`, `text(&str)` builders

### Usage Pattern

```rust
use kaname::{ToolRegistry, ToolResponse, McpServerInfo};

let info = McpServerInfo::new("my-server", "0.1.0", "Does things");

let mut registry = ToolRegistry::new();
registry.register("greet", "Say hello", serde_json::json!({
    "type": "object",
    "properties": { "name": { "type": "string" } }
}));

// In a tool handler:
let response = ToolResponse::success(&serde_json::json!({"greeting": "hello"}));
let error = ToolResponse::error("something broke");
```

## Consumers

- **ayatsuri** -- window manager MCP server
- **hikyaku** -- email client MCP server
