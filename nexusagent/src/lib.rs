pub mod core;

pub use core::memory::tree::{MemoryTree, SqliteMemoryTree};
pub use core::pev::PEVEngine;
pub use core::token_juice::TokenJuiceCompressor;
pub use core::tools::registry::InMemoryToolRegistry;
pub use core::config::Config;

pub struct NexusAgent {
    pub pev: PEVEngine,
    pub memory: SqliteMemoryTree,
    pub tools: InMemoryToolRegistry,
    pub config: Config,
}

impl NexusAgent {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;
        Ok(Self {
            pev: PEVEngine::new(),
            memory: SqliteMemoryTree::new(),
            tools: InMemoryToolRegistry::new(),
            config,
        })
    }

    pub async fn execute_task(&mut self, task: &str) -> Result<String, Box<dyn std::error::Error>> {
        let context = self.memory.build_context_summary()?;
        let result = self.pev.end_to_end_execute(task).await?;
        let _ = self.memory.insert(task, "user_task");
        let _ = self.memory.insert(&result.summary, "agent_result");
        Ok(result.summary)
    }
}

impl Default for NexusAgent {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
