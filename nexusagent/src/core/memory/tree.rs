use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub content: String,
    pub token_count: usize,
    pub score: f32,
    pub source_ref: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Token limit exceeded")]
    TokenLimitExceeded,
}

pub trait MemoryTree {
    fn search_entities(&self, query: &str, limit: usize) -> Result<Vec<MemoryNode>, MemoryError>;
    fn query_topic(&self, topic: &str, days: u32) -> Result<Vec<MemoryNode>, MemoryError>;
    fn query_source(&self, source: &str, limit: usize) -> Result<Vec<MemoryNode>, MemoryError>;
    fn query_global(&self, scope: &str, summary: bool) -> Result<Vec<MemoryNode>, MemoryError>;
    fn drill_down(&self, node_id: &str, depth: u32) -> Result<Vec<MemoryNode>, MemoryError>;
    fn fetch_leaves(&self, node_ids: &[String]) -> Result<Vec<MemoryNode>, MemoryError>;
    fn insert(&mut self, content: &str, source: &str) -> Result<String, MemoryError>;
    fn build_context_summary(&self) -> Result<String, MemoryError>;
}

pub struct SqliteMemoryTree {
    nodes: HashMap<String, MemoryNode>,
    root_id: String,
}

impl SqliteMemoryTree {
    pub fn new() -> Self {
        let root_id = uuid::Uuid::new_v4().to_string();
        let root = MemoryNode {
            node_id: root_id.clone(),
            parent_id: None,
            child_ids: Vec::new(),
            content: "Root".to_string(),
            token_count: 0,
            score: 0.0,
            source_ref: "system".to_string(),
            created_at: Utc::now(),
        };
        let mut nodes = HashMap::new();
        nodes.insert(root_id.clone(), root);
        Self { nodes, root_id }
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count() * 1.3 as usize
    }
}

impl MemoryTree for SqliteMemoryTree {
    fn search_entities(&self, query: &str, limit: usize) -> Result<Vec<MemoryNode>, MemoryError> {
        let mut results: Vec<_> = self.nodes.values()
            .filter(|node| node.content.to_lowercase().contains(&query.to_lowercase()))
            .cloned()
            .collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        Ok(results)
    }

    fn query_topic(&self, topic: &str, days: u32) -> Result<Vec<MemoryNode>, MemoryError> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        let mut results: Vec<_> = self.nodes.values()
            .filter(|node| node.created_at > cutoff && node.content.to_lowercase().contains(&topic.to_lowercase()))
            .cloned()
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(results)
    }

    fn query_source(&self, source: &str, limit: usize) -> Result<Vec<MemoryNode>, MemoryError> {
        let mut results: Vec<_> = self.nodes.values()
            .filter(|node| node.source_ref == source)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        results.truncate(limit);
        Ok(results)
    }

    fn query_global(&self, scope: &str, summary: bool) -> Result<Vec<MemoryNode>, MemoryError> {
        let mut results: Vec<_> = self.nodes.values().cloned().collect();
        if summary {
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            results.truncate(10);
        }
        Ok(results)
    }

    fn drill_down(&self, node_id: &str, depth: u32) -> Result<Vec<MemoryNode>, MemoryError> {
        let mut result = Vec::new();
        let mut current = vec![node_id.to_string()];
        for _ in 0..=depth {
            let mut next_level = Vec::new();
            for id in current {
                if let Some(node) = self.nodes.get(&id) {
                    result.push(node.clone());
                    next_level.extend(node.child_ids.clone());
                }
            }
            current = next_level;
        }
        Ok(result)
    }

    fn fetch_leaves(&self, node_ids: &[String]) -> Result<Vec<MemoryNode>, MemoryError> {
        let mut leaves = Vec::new();
        for id in node_ids {
            if let Some(node) = self.nodes.get(id) {
                if node.child_ids.is_empty() {
                    leaves.push(node.clone());
                } else {
                    leaves.extend(self.fetch_leaves(&node.child_ids)?);
                }
            }
        }
        Ok(leaves)
    }

    fn insert(&mut self, content: &str, source: &str) -> Result<String, MemoryError> {
        let node_id = uuid::Uuid::new_v4().to_string();
        let token_count = self.estimate_tokens(content);
        let node = MemoryNode {
            node_id: node_id.clone(),
            parent_id: Some(self.root_id.clone()),
            child_ids: Vec::new(),
            content: content.to_string(),
            token_count,
            score: 0.5,
            source_ref: source.to_string(),
            created_at: Utc::now(),
        };
        if let Some(root) = self.nodes.get_mut(&self.root_id) {
            root.child_ids.push(node_id.clone());
        }
        self.nodes.insert(node_id.clone(), node);
        Ok(node_id)
    }

    fn build_context_summary(&self) -> Result<String, MemoryError> {
        let mut summary = String::new();
        let recent = Utc::now() - Duration::days(7);
        let mut nodes: Vec<_> = self.nodes.values()
            .filter(|n| n.created_at > recent && n.node_id != self.root_id)
            .cloned()
            .collect();
        nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        nodes.truncate(20);
        for node in nodes {
            summary.push_str(&format!("- [{}] {}\n", node.source_ref, node.content));
        }
        Ok(summary)
    }
}

impl Default for SqliteMemoryTree {
    fn default() -> Self {
        Self::new()
    }
}
