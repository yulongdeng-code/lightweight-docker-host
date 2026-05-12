use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub handler: String,
    pub plugin_id: Option<String>,
    pub mcp_server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConnection {
    pub name: String,
    pub url: String,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Tool already exists: {0}")]
    ToolExists(String),
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("MCP error: {0}")]
    MCPError(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

#[derive(Debug, Error)]
pub enum MCPError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Tool call failed: {0}")]
    ToolCallFailed(String),
}

pub trait ToolRegistry: Send + Sync {
    fn register_tool(&mut self, tool: ToolDefinition) -> Result<(), ToolError>;
    fn deregister_tool(&mut self, name: &str) -> Result<(), ToolError>;
    fn get_tool(&self, name: &str) -> Result<&ToolDefinition, ToolError>;
    fn list_tools(&self, category: Option<&str>) -> Vec<&ToolDefinition>;
    fn search_tools(&self, query: &str) -> Vec<&ToolDefinition>;
    async fn discover_mcp_tools(&mut self, server_url: &str) -> Result<Vec<ToolDefinition>, ToolError>;
    async fn execute_tool(&self, call: &ToolCall) -> Result<serde_json::Value, ToolError>;
}

pub struct InMemoryToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    mcp_servers: HashMap<String, MCPConnection>,
}

impl InMemoryToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            mcp_servers: HashMap::new(),
        };
        registry.register_builtin_tools();
        registry
    }

    fn register_builtin_tools(&mut self) {
        let tools = vec![
            ToolDefinition {
                name: "echo".to_string(),
                description: "Echo back the input".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"]
                }),
                handler: "builtin.echo".to_string(),
                plugin_id: None,
                mcp_server: None,
            },
            ToolDefinition {
                name: "sum".to_string(),
                description: "Sum two numbers".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["a", "b"]
                }),
                handler: "builtin.sum".to_string(),
                plugin_id: None,
                mcp_server: None,
            },
        ];

        for tool in tools {
            let _ = self.register_tool(tool);
        }
    }

    fn execute_builtin_tool(&self, call: &ToolCall) -> Result<serde_json::Value, ToolError> {
        match call.name.as_str() {
            "echo" => {
                let message = call.parameters["message"].as_str().ok_or_else(|| {
                    ToolError::InvalidParams("Missing message".to_string())
                })?;
                Ok(serde_json::json!({ "result": message }))
            }
            "sum" => {
                let a = call.parameters["a"].as_f64().ok_or_else(|| {
                    ToolError::InvalidParams("Invalid number a".to_string())
                })?;
                let b = call.parameters["b"].as_f64().ok_or_else(|| {
                    ToolError::InvalidParams("Invalid number b".to_string())
                })?;
                Ok(serde_json::json!({ "result": a + b }))
            }
            _ => Err(ToolError::ToolNotFound(call.name.clone())),
        }
    }
}

impl ToolRegistry for InMemoryToolRegistry {
    fn register_tool(&mut self, tool: ToolDefinition) -> Result<(), ToolError> {
        if self.tools.contains_key(&tool.name) {
            return Err(ToolError::ToolExists(tool.name));
        }
        self.tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    fn deregister_tool(&mut self, name: &str) -> Result<(), ToolError> {
        self.tools.remove(name).ok_or_else(|| ToolError::ToolNotFound(name.to_string()))?;
        Ok(())
    }

    fn get_tool(&self, name: &str) -> Result<&ToolDefinition, ToolError> {
        self.tools.get(name).ok_or_else(|| ToolError::ToolNotFound(name.to_string()))
    }

    fn list_tools(&self, _category: Option<&str>) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    fn search_tools(&self, query: &str) -> Vec<&ToolDefinition> {
        let lower_query = query.to_lowercase();
        self.tools.values()
            .filter(|t| {
                t.name.to_lowercase().contains(&lower_query) ||
                t.description.to_lowercase().contains(&lower_query)
            })
            .collect()
    }

    async fn discover_mcp_tools(&mut self, server_url: &str) -> Result<Vec<ToolDefinition>, ToolError> {
        Ok(vec![])
    }

    async fn execute_tool(&self, call: &ToolCall) -> Result<serde_json::Value, ToolError> {
        if let Some(tool) = self.tools.get(&call.name) {
            if tool.mcp_server.is_some() {
                Err(ToolError::MCPError("MCP not implemented".to_string()))
            } else {
                self.execute_builtin_tool(call)
            }
        } else {
            Err(ToolError::ToolNotFound(call.name.clone()))
        }
    }
}

impl Default for InMemoryToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MCPClient {
    servers: HashMap<String, MCPConnection>,
}

impl MCPClient {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub async fn connect(&mut self, name: &str, url: &str) -> Result<(), MCPError> {
        self.servers.insert(name.to_string(), MCPConnection {
            name: name.to_string(),
            url: url.to_string(),
            tools: vec![],
        });
        Ok(())
    }

    pub async fn list_tools(&self, server: &str) -> Result<Vec<ToolDefinition>, MCPError> {
        self.servers.get(server)
            .ok_or_else(|| MCPError::ConnectionFailed("Server not found".to_string()))
            .map(|conn| conn.tools.clone())
    }

    pub async fn call_tool(&self, server: &str, tool: &str, params: serde_json::Value) -> Result<serde_json::Value, MCPError> {
        let _ = server;
        let _ = tool;
        let _ = params;
        Ok(serde_json::json!({}))
    }
}

impl Default for MCPClient {
    fn default() -> Self {
        Self::new()
    }
}
