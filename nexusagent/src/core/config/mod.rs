use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LLMConfig,
    pub memory: MemoryConfig,
    pub docker: DockerConfig,
    pub plugins: PluginsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub storage_path: String,
    pub max_nodes: usize,
    pub enable_summary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub enabled: bool,
    pub sandbox_image: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    pub enabled: Vec<String>,
    pub plugin_dir: String,
    pub mcp_servers: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LLMConfig {
                model: "gpt-4".to_string(),
                api_key: None,
                base_url: None,
                temperature: 0.7,
                max_tokens: 2000,
            },
            memory: MemoryConfig {
                storage_path: "./data/memory".to_string(),
                max_nodes: 10000,
                enable_summary: true,
            },
            docker: DockerConfig {
                enabled: false,
                sandbox_image: "rust:latest".to_string(),
                timeout_seconds: 30,
            },
            plugins: PluginsConfig {
                enabled: vec![],
                plugin_dir: "./plugins".to_string(),
                mcp_servers: HashMap::new(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = std::path::Path::new("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write("config.toml", content)?;
        Ok(())
    }
}
