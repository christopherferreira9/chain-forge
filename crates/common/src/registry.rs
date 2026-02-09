//! Unified node registry for tracking all running blockchain nodes across chains.
//!
//! This module provides a central registry that tracks all running nodes
//! (Solana, Bitcoin, etc.) and their status. The registry is persisted to disk
//! and supports file locking for safe concurrent access.

// Allow fs2 trait methods that have the same name as std methods stabilized in Rust 1.89
#![allow(clippy::incompatible_msrv)]

use crate::{ChainError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

// Import FileExt trait for file locking
use fs2::FileExt;

/// Type of blockchain chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Solana,
    Bitcoin,
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::Solana => write!(f, "solana"),
            ChainType::Bitcoin => write!(f, "bitcoin"),
        }
    }
}

/// Status of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Running,
    Stopped,
    Unknown,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Running => write!(f, "running"),
            NodeStatus::Stopped => write!(f, "stopped"),
            NodeStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about a registered node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique node identifier: "{chain}:{instance_id}"
    pub node_id: String,
    /// Human-readable name for the node
    pub name: Option<String>,
    /// Type of blockchain
    pub chain: ChainType,
    /// Instance ID within the chain
    pub instance_id: String,
    /// RPC URL for connecting to the node
    pub rpc_url: String,
    /// RPC port
    pub rpc_port: u16,
    /// Number of accounts configured
    pub accounts_count: u32,
    /// Current status
    pub status: NodeStatus,
    /// When the node was started
    pub started_at: Option<DateTime<Utc>>,
}

impl NodeInfo {
    /// Create a new NodeInfo
    pub fn new(
        chain: ChainType,
        instance_id: &str,
        name: Option<String>,
        rpc_url: String,
        rpc_port: u16,
        accounts_count: u32,
    ) -> Self {
        Self {
            node_id: format!("{}:{}", chain, instance_id),
            name,
            chain,
            instance_id: instance_id.to_string(),
            rpc_url,
            rpc_port,
            accounts_count,
            status: NodeStatus::Running,
            started_at: Some(Utc::now()),
        }
    }

    /// Get a display name (name if set, otherwise instance_id)
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.instance_id)
    }
}

/// Registry data stored on disk
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegistryData {
    nodes: HashMap<String, NodeInfo>,
}

/// Node registry for tracking all running nodes
pub struct NodeRegistry {
    registry_path: PathBuf,
    backup_path: PathBuf,
}

impl NodeRegistry {
    /// Create a new NodeRegistry using the default data directory
    pub fn new() -> Self {
        let data_dir = Self::data_dir();
        let registry_path = data_dir.join("registry.json");
        let backup_path = data_dir.join("registry.json.bak");

        Self {
            registry_path,
            backup_path,
        }
    }

    /// Create a NodeRegistry with a custom path (mainly for testing)
    pub fn with_path(registry_path: PathBuf) -> Self {
        let backup_path = registry_path.with_extension("json.bak");
        Self {
            registry_path,
            backup_path,
        }
    }

    /// Get the data directory path
    fn data_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".chain-forge")
    }

    /// Ensure the data directory exists
    fn ensure_data_dir(&self) -> Result<()> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Load registry data with file locking
    fn load(&self) -> Result<RegistryData> {
        if !self.registry_path.exists() {
            return Ok(RegistryData::default());
        }

        let file = File::open(&self.registry_path)?;
        file.lock_shared().map_err(|e| {
            ChainError::Other(format!("Failed to acquire shared lock on registry: {}", e))
        })?;

        let mut contents = String::new();
        let mut reader = std::io::BufReader::new(&file);
        reader.read_to_string(&mut contents)?;

        file.unlock()
            .map_err(|e| ChainError::Other(format!("Failed to release lock on registry: {}", e)))?;

        // Try to parse, if corrupted try backup
        match serde_json::from_str(&contents) {
            Ok(data) => Ok(data),
            Err(e) => {
                eprintln!(
                    "Warning: Registry file corrupted ({}), attempting recovery from backup",
                    e
                );
                self.load_backup()
            }
        }
    }

    /// Load from backup file
    fn load_backup(&self) -> Result<RegistryData> {
        if !self.backup_path.exists() {
            eprintln!("Warning: No backup available, starting with empty registry");
            return Ok(RegistryData::default());
        }

        let contents = fs::read_to_string(&self.backup_path)?;
        match serde_json::from_str(&contents) {
            Ok(data) => {
                eprintln!("Successfully recovered from backup");
                Ok(data)
            }
            Err(e) => {
                eprintln!("Warning: Backup also corrupted ({}), starting fresh", e);
                Ok(RegistryData::default())
            }
        }
    }

    /// Save registry data atomically with file locking
    fn save(&self, data: &RegistryData) -> Result<()> {
        self.ensure_data_dir()?;

        // Create backup of current file if it exists
        if self.registry_path.exists() {
            fs::copy(&self.registry_path, &self.backup_path)?;
        }

        // Write to temp file first
        let temp_path = self.registry_path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(data)?;

        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_path)?;

            file.lock_exclusive().map_err(|e| {
                ChainError::Other(format!("Failed to acquire exclusive lock: {}", e))
            })?;

            file.write_all(json.as_bytes())?;
            file.sync_all()?;

            file.unlock()
                .map_err(|e| ChainError::Other(format!("Failed to release lock: {}", e)))?;
        }

        // Atomic rename
        fs::rename(&temp_path, &self.registry_path)?;

        Ok(())
    }

    /// Register a new node
    pub fn register(&self, node: NodeInfo) -> Result<()> {
        let mut data = self.load()?;
        data.nodes.insert(node.node_id.clone(), node);
        self.save(&data)
    }

    /// Unregister a node by its node_id
    pub fn unregister(&self, node_id: &str) -> Result<()> {
        let mut data = self.load()?;
        data.nodes.remove(node_id);
        self.save(&data)
    }

    /// Update a node's status
    pub fn update_status(&self, node_id: &str, status: NodeStatus) -> Result<()> {
        let mut data = self.load()?;
        if let Some(node) = data.nodes.get_mut(node_id) {
            node.status = status;
            self.save(&data)?;
        }
        Ok(())
    }

    /// Get a specific node by ID
    pub fn get(&self, node_id: &str) -> Result<Option<NodeInfo>> {
        let data = self.load()?;
        Ok(data.nodes.get(node_id).cloned())
    }

    /// List all registered nodes
    pub fn list(&self) -> Result<Vec<NodeInfo>> {
        let data = self.load()?;
        Ok(data.nodes.values().cloned().collect())
    }

    /// List nodes by chain type
    pub fn list_by_chain(&self, chain: ChainType) -> Result<Vec<NodeInfo>> {
        let data = self.load()?;
        Ok(data
            .nodes
            .values()
            .filter(|n| n.chain == chain)
            .cloned()
            .collect())
    }

    /// Mark all nodes of a chain type as stopped
    pub fn mark_all_stopped(&self, chain: ChainType) -> Result<()> {
        let mut data = self.load()?;
        for node in data.nodes.values_mut() {
            if node.chain == chain {
                node.status = NodeStatus::Stopped;
            }
        }
        self.save(&data)
    }

    /// Clear all stopped nodes from the registry
    pub fn clear_stopped(&self) -> Result<()> {
        let mut data = self.load()?;
        data.nodes
            .retain(|_, node| node.status != NodeStatus::Stopped);
        self.save(&data)
    }

    /// Generate a node_id from chain and instance_id
    pub fn node_id(chain: ChainType, instance_id: &str) -> String {
        format!("{}:{}", chain, instance_id)
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_registry() -> (NodeRegistry, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_registry.json");
        let registry = NodeRegistry::with_path(path);
        (registry, dir)
    }

    #[test]
    fn test_node_info_creation() {
        let node = NodeInfo::new(
            ChainType::Solana,
            "test-instance",
            Some("Test Node".to_string()),
            "http://localhost:8899".to_string(),
            8899,
            10,
        );

        assert_eq!(node.node_id, "solana:test-instance");
        assert_eq!(node.display_name(), "Test Node");
        assert_eq!(node.chain, ChainType::Solana);
        assert_eq!(node.status, NodeStatus::Running);
    }

    #[test]
    fn test_node_info_without_name() {
        let node = NodeInfo::new(
            ChainType::Bitcoin,
            "btc-dev",
            None,
            "http://localhost:18443".to_string(),
            18443,
            5,
        );

        assert_eq!(node.display_name(), "btc-dev");
    }

    #[test]
    fn test_register_and_list() {
        let (registry, _dir) = create_test_registry();

        let node = NodeInfo::new(
            ChainType::Solana,
            "dev1",
            Some("Development".to_string()),
            "http://localhost:8899".to_string(),
            8899,
            10,
        );

        registry.register(node.clone()).unwrap();

        let nodes = registry.list().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, "solana:dev1");
    }

    #[test]
    fn test_unregister() {
        let (registry, _dir) = create_test_registry();

        let node = NodeInfo::new(
            ChainType::Solana,
            "dev1",
            None,
            "http://localhost:8899".to_string(),
            8899,
            10,
        );

        registry.register(node).unwrap();
        assert_eq!(registry.list().unwrap().len(), 1);

        registry.unregister("solana:dev1").unwrap();
        assert_eq!(registry.list().unwrap().len(), 0);
    }

    #[test]
    fn test_update_status() {
        let (registry, _dir) = create_test_registry();

        let node = NodeInfo::new(
            ChainType::Bitcoin,
            "btc1",
            None,
            "http://localhost:18443".to_string(),
            18443,
            5,
        );

        registry.register(node).unwrap();
        registry
            .update_status("bitcoin:btc1", NodeStatus::Stopped)
            .unwrap();

        let fetched = registry.get("bitcoin:btc1").unwrap().unwrap();
        assert_eq!(fetched.status, NodeStatus::Stopped);
    }

    #[test]
    fn test_list_by_chain() {
        let (registry, _dir) = create_test_registry();

        let solana_node = NodeInfo::new(
            ChainType::Solana,
            "sol1",
            None,
            "http://localhost:8899".to_string(),
            8899,
            10,
        );

        let bitcoin_node = NodeInfo::new(
            ChainType::Bitcoin,
            "btc1",
            None,
            "http://localhost:18443".to_string(),
            18443,
            5,
        );

        registry.register(solana_node).unwrap();
        registry.register(bitcoin_node).unwrap();

        let solana_nodes = registry.list_by_chain(ChainType::Solana).unwrap();
        assert_eq!(solana_nodes.len(), 1);
        assert_eq!(solana_nodes[0].chain, ChainType::Solana);

        let bitcoin_nodes = registry.list_by_chain(ChainType::Bitcoin).unwrap();
        assert_eq!(bitcoin_nodes.len(), 1);
        assert_eq!(bitcoin_nodes[0].chain, ChainType::Bitcoin);
    }

    #[test]
    fn test_clear_stopped() {
        let (registry, _dir) = create_test_registry();

        let mut node1 = NodeInfo::new(
            ChainType::Solana,
            "sol1",
            None,
            "http://localhost:8899".to_string(),
            8899,
            10,
        );
        node1.status = NodeStatus::Stopped;

        let node2 = NodeInfo::new(
            ChainType::Bitcoin,
            "btc1",
            None,
            "http://localhost:18443".to_string(),
            18443,
            5,
        );

        registry.register(node1).unwrap();
        registry.register(node2).unwrap();
        assert_eq!(registry.list().unwrap().len(), 2);

        registry.clear_stopped().unwrap();
        let remaining = registry.list().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].chain, ChainType::Bitcoin);
    }

    #[test]
    fn test_node_id_helper() {
        assert_eq!(
            NodeRegistry::node_id(ChainType::Solana, "dev"),
            "solana:dev"
        );
        assert_eq!(
            NodeRegistry::node_id(ChainType::Bitcoin, "test"),
            "bitcoin:test"
        );
    }

    #[test]
    fn test_chain_type_display() {
        assert_eq!(format!("{}", ChainType::Solana), "solana");
        assert_eq!(format!("{}", ChainType::Bitcoin), "bitcoin");
    }

    #[test]
    fn test_node_status_display() {
        assert_eq!(format!("{}", NodeStatus::Running), "running");
        assert_eq!(format!("{}", NodeStatus::Stopped), "stopped");
        assert_eq!(format!("{}", NodeStatus::Unknown), "unknown");
    }
}
