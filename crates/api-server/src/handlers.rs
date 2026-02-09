//! Request handlers for the Chain Forge REST API.

use axum::{extract::Path, http::StatusCode, Json};
use chain_forge_bitcoin_accounts::AccountsStorage as BitcoinAccountsStorage;
use chain_forge_bitcoin_core::InstanceInfo as BitcoinInstanceInfo;
use chain_forge_bitcoin_rpc::BitcoinRpcClient;
use chain_forge_common::{ChainType, NodeInfo, NodeRegistry, NodeStatus};
use chain_forge_config::Config;
use chain_forge_solana_accounts::AccountsStorage as SolanaAccountsStorage;
use chain_forge_solana_rpc::SolanaRpcClient;
use serde::{Deserialize, Serialize};

/// Response wrapper for API responses
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// Node info for API response (includes additional runtime info)
#[derive(Serialize)]
pub struct NodeInfoResponse {
    pub node_id: String,
    pub name: Option<String>,
    pub chain: String,
    pub instance_id: String,
    pub rpc_url: String,
    pub rpc_port: u16,
    pub accounts_count: u32,
    pub status: String,
    pub started_at: Option<String>,
}

impl From<NodeInfo> for NodeInfoResponse {
    fn from(info: NodeInfo) -> Self {
        Self {
            node_id: info.node_id,
            name: info.name,
            chain: info.chain.to_string(),
            instance_id: info.instance_id,
            rpc_url: info.rpc_url,
            rpc_port: info.rpc_port,
            accounts_count: info.accounts_count,
            status: info.status.to_string(),
            started_at: info.started_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Account info for API response
#[derive(Serialize)]
pub struct AccountInfo {
    pub index: usize,
    pub address: String,
    pub balance: f64,
}

/// Request to start a new node
#[derive(Deserialize)]
pub struct StartNodeRequest {
    pub chain: String,
    #[serde(default = "default_instance")]
    pub instance: String,
    pub name: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_accounts")]
    pub accounts: u32,
    #[serde(default = "default_balance")]
    pub balance: f64,
}

fn default_instance() -> String {
    "default".to_string()
}

fn default_port() -> u16 {
    8899
}

fn default_accounts() -> u32 {
    10
}

fn default_balance() -> f64 {
    100.0
}

/// Request to fund an account
#[derive(Deserialize)]
pub struct FundAccountRequest {
    pub address: String,
    pub amount: f64,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthCheckResponse {
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub unknown: usize,
}

/// Start node response
#[derive(Serialize)]
pub struct StartNodeResponse {
    pub message: String,
    pub command: String,
    pub chain: String,
    pub instance: String,
    pub port: u16,
}

/// Stop node response
#[derive(Serialize)]
pub struct StopNodeResponse {
    pub message: String,
    pub instruction: String,
    pub node_id: String,
}

/// Fund response
#[derive(Serialize)]
pub struct FundResponse {
    pub success: bool,
    pub txid_or_signature: String,
    pub address: String,
    pub amount: f64,
}

/// Transaction info for API response
#[derive(Serialize)]
pub struct TransactionInfo {
    pub signature: String,
    pub slot: u64,
    pub err: Option<String>,
    pub memo: Option<String>,
    pub block_time: Option<i64>,
    pub confirmation_status: Option<String>,
    pub account: String,
}

/// Balance change in a transaction
#[derive(Serialize)]
pub struct BalanceChangeInfo {
    pub account: String,
    pub before: f64,
    pub after: f64,
    pub change: f64,
}

/// Detailed transaction info
#[derive(Serialize)]
pub struct TransactionDetailInfo {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub fee: f64,
    pub err: Option<String>,
    pub balance_changes: Vec<BalanceChangeInfo>,
}

/// Cleanup response
#[derive(Serialize)]
pub struct CleanupResponse {
    pub removed: usize,
    pub remaining: usize,
    pub removed_nodes: Vec<String>,
}

/// List all registered nodes
pub async fn list_nodes() -> (StatusCode, Json<ApiResponse<Vec<NodeInfoResponse>>>) {
    let registry = NodeRegistry::new();

    match registry.list() {
        Ok(nodes) => {
            let response: Vec<NodeInfoResponse> = nodes.into_iter().map(Into::into).collect();
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to list nodes: {}", e))),
        ),
    }
}

/// Get a specific node by ID
pub async fn get_node(
    Path(node_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<NodeInfoResponse>>) {
    let registry = NodeRegistry::new();

    match registry.get(&node_id) {
        Ok(Some(node)) => (
            StatusCode::OK,
            Json(ApiResponse::success(NodeInfoResponse::from(node))),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Node not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to get node: {}", e))),
        ),
    }
}

/// Get accounts for a specific node with live balances from the blockchain
pub async fn get_node_accounts(
    Path(node_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<AccountInfo>>>) {
    let registry = NodeRegistry::new();

    let node = match registry.get(&node_id) {
        Ok(Some(node)) => node,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Node not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to get node: {}", e))),
            );
        }
    };

    let accounts: Vec<AccountInfo> = match node.chain {
        ChainType::Solana => {
            let accounts_file = Config::data_dir()
                .join("solana")
                .join("instances")
                .join(&node.instance_id)
                .join("accounts.json");
            let storage = SolanaAccountsStorage::with_path(accounts_file);

            match storage.load() {
                Ok(mut accounts) => {
                    // Fetch live balances from the blockchain
                    let rpc_client = SolanaRpcClient::new(node.rpc_url.clone());
                    let _ = rpc_client.update_balances(&mut accounts);

                    accounts
                        .into_iter()
                        .enumerate()
                        .map(|(i, acc)| AccountInfo {
                            index: i,
                            address: acc.public_key,
                            balance: acc.balance,
                        })
                        .collect()
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load accounts: {}",
                            e
                        ))),
                    );
                }
            }
        }
        ChainType::Bitcoin => {
            let accounts_file = Config::data_dir()
                .join("bitcoin")
                .join("instances")
                .join(&node.instance_id)
                .join("accounts.json");
            let storage = BitcoinAccountsStorage::with_path(accounts_file);

            match storage.load() {
                Ok(mut accounts) => {
                    // Fetch live balances from the blockchain
                    if let Ok(info) = BitcoinInstanceInfo::load(&node.instance_id) {
                        if let Ok(rpc_client) = BitcoinRpcClient::new_with_wallet(
                            info.rpc_url,
                            info.rpc_user,
                            info.rpc_password,
                            "chain-forge",
                        ) {
                            let _ = rpc_client.update_balances(&mut accounts);
                        }
                    }

                    accounts
                        .into_iter()
                        .enumerate()
                        .map(|(i, acc)| AccountInfo {
                            index: i,
                            address: acc.address,
                            balance: acc.balance,
                        })
                        .collect()
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load accounts: {}",
                            e
                        ))),
                    );
                }
            }
        }
    };

    (StatusCode::OK, Json(ApiResponse::success(accounts)))
}

/// Perform health check on all nodes
pub async fn health_check() -> (StatusCode, Json<ApiResponse<HealthCheckResponse>>) {
    let registry = NodeRegistry::new();

    let nodes = match registry.list() {
        Ok(nodes) => nodes,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to list nodes: {}", e))),
            );
        }
    };

    let mut running = 0;
    let mut stopped = 0;
    let mut unknown = 0;

    for node in &nodes {
        let is_running = match node.chain {
            ChainType::Solana => {
                let client = SolanaRpcClient::new(node.rpc_url.clone());
                client.is_validator_running()
            }
            ChainType::Bitcoin => {
                if let Ok(info) = BitcoinInstanceInfo::load(&node.instance_id) {
                    match BitcoinRpcClient::new_with_wallet(
                        info.rpc_url,
                        info.rpc_user,
                        info.rpc_password,
                        "chain-forge",
                    ) {
                        Ok(client) => client.is_node_running(),
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
        };

        // Update registry status
        let new_status = if is_running {
            running += 1;
            NodeStatus::Running
        } else {
            // Check if it was previously running
            match node.status {
                NodeStatus::Running => {
                    stopped += 1;
                    NodeStatus::Stopped
                }
                NodeStatus::Stopped => {
                    stopped += 1;
                    NodeStatus::Stopped
                }
                NodeStatus::Unknown => {
                    unknown += 1;
                    NodeStatus::Unknown
                }
            }
        };

        // Update status in registry
        let _ = registry.update_status(&node.node_id, new_status);
    }

    let response = HealthCheckResponse {
        total: nodes.len(),
        running,
        stopped,
        unknown,
    };

    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// Start a new node (returns immediately with instance info)
/// Note: Actually starting nodes requires running them in the background
/// which would need more sophisticated process management.
/// For now, this returns instructions on how to start the node.
pub async fn start_node(
    Json(req): Json<StartNodeRequest>,
) -> (StatusCode, Json<ApiResponse<StartNodeResponse>>) {
    let chain = match req.chain.to_lowercase().as_str() {
        "solana" => "solana",
        "bitcoin" => "bitcoin",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "Invalid chain. Use 'solana' or 'bitcoin'",
                )),
            );
        }
    };

    // Build the command to start the node
    let cmd = match chain {
        "solana" => format!(
            "cf-solana start --instance {} --port {} --accounts {} --balance {}{}",
            req.instance,
            req.port,
            req.accounts,
            req.balance,
            req.name
                .as_ref()
                .map(|n| format!(" --name \"{}\"", n))
                .unwrap_or_default()
        ),
        "bitcoin" => format!(
            "cf-bitcoin start --instance {} --rpc-port {} --accounts {} --balance {}{}",
            req.instance,
            req.port,
            req.accounts,
            req.balance,
            req.name
                .as_ref()
                .map(|n| format!(" --name \"{}\"", n))
                .unwrap_or_default()
        ),
        _ => unreachable!(),
    };

    let response = StartNodeResponse {
        message: "Node start requires running the CLI command in a separate terminal".to_string(),
        command: cmd,
        chain: chain.to_string(),
        instance: req.instance,
        port: req.port,
    };

    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// Stop a node (marks as stopped in registry)
/// Note: Actually stopping nodes would require process management.
/// For now, this just updates the registry.
pub async fn stop_node(
    Path(node_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<StopNodeResponse>>) {
    let registry = NodeRegistry::new();

    // Check if node exists
    match registry.get(&node_id) {
        Ok(Some(node)) => {
            // Update status to stopped
            if let Err(e) = registry.update_status(&node_id, NodeStatus::Stopped) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update status: {}",
                        e
                    ))),
                );
            }

            let instruction = match node.chain {
                ChainType::Solana => format!(
                    "Press Ctrl+C in the terminal running 'cf-solana start --instance {}'",
                    node.instance_id
                ),
                ChainType::Bitcoin => format!(
                    "Press Ctrl+C in the terminal running 'cf-bitcoin start --instance {}'",
                    node.instance_id
                ),
            };

            let response = StopNodeResponse {
                message: "Node marked as stopped. To actually stop the node:".to_string(),
                instruction,
                node_id,
            };

            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Node not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to get node: {}", e))),
        ),
    }
}

/// Fund an account on a specific node
pub async fn fund_account(
    Path(node_id): Path<String>,
    Json(req): Json<FundAccountRequest>,
) -> (StatusCode, Json<ApiResponse<FundResponse>>) {
    let registry = NodeRegistry::new();

    let node = match registry.get(&node_id) {
        Ok(Some(node)) => node,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Node not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to get node: {}", e))),
            );
        }
    };

    match node.chain {
        ChainType::Solana => {
            let client = SolanaRpcClient::new(node.rpc_url.clone());
            if !client.is_validator_running() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error("Solana validator is not running")),
                );
            }

            match client.request_airdrop(&req.address, req.amount) {
                Ok(signature) => {
                    let response = FundResponse {
                        success: true,
                        txid_or_signature: signature,
                        address: req.address,
                        amount: req.amount,
                    };
                    (StatusCode::OK, Json(ApiResponse::success(response)))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Airdrop failed: {}", e))),
                ),
            }
        }
        ChainType::Bitcoin => {
            let info = match BitcoinInstanceInfo::load(&node.instance_id) {
                Ok(info) => info,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load instance info: {}",
                            e
                        ))),
                    );
                }
            };

            let client = match BitcoinRpcClient::new_with_wallet(
                info.rpc_url,
                info.rpc_user,
                info.rpc_password,
                "chain-forge",
            ) {
                Ok(client) => client,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to create RPC client: {}",
                            e
                        ))),
                    );
                }
            };

            if !client.is_node_running() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error("Bitcoin node is not running")),
                );
            }

            match client.send_to_address(&req.address, req.amount) {
                Ok(txid) => {
                    // Mine a block to confirm
                    let _ = client
                        .get_new_address(Some("mining"))
                        .and_then(|addr| client.mine_blocks(1, &addr));

                    let response = FundResponse {
                        success: true,
                        txid_or_signature: txid,
                        address: req.address,
                        amount: req.amount,
                    };
                    (StatusCode::OK, Json(ApiResponse::success(response)))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Transaction failed: {}", e))),
                ),
            }
        }
    }
}

/// Clean up the registry by removing nodes that are not currently running
pub async fn cleanup_registry() -> (StatusCode, Json<ApiResponse<CleanupResponse>>) {
    let registry = NodeRegistry::new();

    let nodes = match registry.list() {
        Ok(nodes) => nodes,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to list nodes: {}", e))),
            );
        }
    };

    let mut removed_nodes = Vec::new();

    for node in &nodes {
        let is_running = match node.chain {
            ChainType::Solana => {
                let client = SolanaRpcClient::new(node.rpc_url.clone());
                client.is_validator_running()
            }
            ChainType::Bitcoin => {
                if let Ok(info) = BitcoinInstanceInfo::load(&node.instance_id) {
                    match BitcoinRpcClient::new_with_wallet(
                        info.rpc_url,
                        info.rpc_user,
                        info.rpc_password,
                        "chain-forge",
                    ) {
                        Ok(client) => client.is_node_running(),
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
        };

        if !is_running {
            let _ = registry.update_status(&node.node_id, NodeStatus::Stopped);
            removed_nodes.push(node.node_id.clone());
        }
    }

    if let Err(e) = registry.clear_stopped() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to clean registry: {}",
                e
            ))),
        );
    }

    let remaining = nodes.len() - removed_nodes.len();
    let removed = removed_nodes.len();

    (
        StatusCode::OK,
        Json(ApiResponse::success(CleanupResponse {
            removed,
            remaining,
            removed_nodes,
        })),
    )
}

/// Get recent transactions for all accounts on a specific node
pub async fn get_node_transactions(
    Path(node_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<TransactionInfo>>>) {
    let registry = NodeRegistry::new();

    let node = match registry.get(&node_id) {
        Ok(Some(node)) => node,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Node not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to get node: {}", e))),
            );
        }
    };

    match node.chain {
        ChainType::Solana => {
            let accounts_file = Config::data_dir()
                .join("solana")
                .join("instances")
                .join(&node.instance_id)
                .join("accounts.json");
            let storage = SolanaAccountsStorage::with_path(accounts_file);

            let accounts = match storage.load() {
                Ok(accounts) => accounts,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load accounts: {}",
                            e
                        ))),
                    );
                }
            };

            let rpc_client = SolanaRpcClient::new(node.rpc_url.clone());
            if !rpc_client.is_validator_running() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error("Solana validator is not running")),
                );
            }

            let mut all_transactions: Vec<TransactionInfo> = Vec::new();

            for account in &accounts {
                match rpc_client.get_signatures_for_address(&account.public_key, Some(10)) {
                    Ok(signatures) => {
                        for sig in signatures {
                            all_transactions.push(TransactionInfo {
                                signature: sig.signature,
                                slot: sig.slot,
                                err: sig.err,
                                memo: sig.memo,
                                block_time: sig.block_time,
                                confirmation_status: sig.confirmation_status,
                                account: account.public_key.clone(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to get transactions for {}: {}",
                            account.public_key, e
                        );
                    }
                }
            }

            // Sort by slot descending (most recent first)
            all_transactions.sort_by(|a, b| b.slot.cmp(&a.slot));

            // Deduplicate by signature (same tx could appear for multiple accounts)
            all_transactions.dedup_by(|a, b| a.signature == b.signature);

            (StatusCode::OK, Json(ApiResponse::success(all_transactions)))
        }
        ChainType::Bitcoin => {
            // Load Bitcoin instance info and create wallet RPC client
            let info = match BitcoinInstanceInfo::load(&node.instance_id) {
                Ok(info) => info,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load instance info: {}",
                            e
                        ))),
                    );
                }
            };

            let rpc_client = match BitcoinRpcClient::new_with_wallet(
                info.rpc_url,
                info.rpc_user,
                info.rpc_password,
                "chain-forge",
            ) {
                Ok(client) => client,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to create RPC client: {}",
                            e
                        ))),
                    );
                }
            };

            if !rpc_client.is_node_running() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error("Bitcoin node is not running")),
                );
            }

            // Load known account addresses for filtering
            let accounts_file = Config::data_dir()
                .join("bitcoin")
                .join("instances")
                .join(&node.instance_id)
                .join("accounts.json");
            let storage = BitcoinAccountsStorage::with_path(accounts_file);

            let known_addresses: std::collections::HashSet<String> = match storage.load() {
                Ok(accounts) => accounts.into_iter().map(|a| a.address).collect(),
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load accounts: {}",
                            e
                        ))),
                    );
                }
            };

            // Get recent wallet transactions
            let wallet_txs = match rpc_client.list_transactions(100) {
                Ok(txs) => txs,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to list transactions: {}",
                            e
                        ))),
                    );
                }
            };

            // Filter to only transactions involving known accounts
            let mut all_transactions: Vec<TransactionInfo> = wallet_txs
                .into_iter()
                .filter(|tx| known_addresses.contains(&tx.address))
                .map(|tx| {
                    let confirmation_status = if tx.confirmations > 0 {
                        Some(format!("{} confirmations", tx.confirmations))
                    } else {
                        Some("unconfirmed".to_string())
                    };

                    TransactionInfo {
                        signature: tx.txid,
                        slot: tx.block_height,
                        err: None,
                        memo: None,
                        block_time: tx.block_time,
                        confirmation_status,
                        account: tx.address,
                    }
                })
                .collect();

            // Sort by block_time descending (most recent first)
            all_transactions.sort_by(|a, b| b.block_time.cmp(&a.block_time));

            // Deduplicate by txid (same tx could appear for sender and receiver)
            all_transactions.dedup_by(|a, b| a.signature == b.signature);

            (StatusCode::OK, Json(ApiResponse::success(all_transactions)))
        }
    }
}

/// Get detailed transaction info by signature for a specific node
pub async fn get_transaction_detail(
    Path((node_id, signature)): Path<(String, String)>,
) -> (StatusCode, Json<ApiResponse<TransactionDetailInfo>>) {
    let registry = NodeRegistry::new();

    let node = match registry.get(&node_id) {
        Ok(Some(node)) => node,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Node not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to get node: {}", e))),
            );
        }
    };

    match node.chain {
        ChainType::Solana => {
            let rpc_client = SolanaRpcClient::new(node.rpc_url.clone());
            if !rpc_client.is_validator_running() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error("Solana validator is not running")),
                );
            }

            match rpc_client.get_transaction(&signature) {
                Ok(detail) => {
                    let response = TransactionDetailInfo {
                        signature: detail.signature,
                        slot: detail.slot,
                        block_time: detail.block_time,
                        fee: detail.fee,
                        err: detail.err,
                        balance_changes: detail
                            .balance_changes
                            .into_iter()
                            .map(|bc| BalanceChangeInfo {
                                account: bc.account,
                                before: bc.before,
                                after: bc.after,
                                change: bc.change,
                            })
                            .collect(),
                    };
                    (StatusCode::OK, Json(ApiResponse::success(response)))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to get transaction: {}",
                        e
                    ))),
                ),
            }
        }
        ChainType::Bitcoin => {
            let info = match BitcoinInstanceInfo::load(&node.instance_id) {
                Ok(info) => info,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to load instance info: {}",
                            e
                        ))),
                    );
                }
            };

            let rpc_client = match BitcoinRpcClient::new_with_wallet(
                info.rpc_url,
                info.rpc_user,
                info.rpc_password,
                "chain-forge",
            ) {
                Ok(client) => client,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!(
                            "Failed to create RPC client: {}",
                            e
                        ))),
                    );
                }
            };

            if !rpc_client.is_node_running() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::error("Bitcoin node is not running")),
                );
            }

            match rpc_client.get_transaction_detail(&signature) {
                Ok(detail) => {
                    let balance_changes: Vec<BalanceChangeInfo> = detail
                        .details
                        .iter()
                        .map(|entry| {
                            let change = entry.amount;
                            // For "receive", amount is positive; for "send", it's negative
                            let (before, after) = if change >= 0.0 {
                                (0.0, change)
                            } else {
                                (change.abs(), 0.0)
                            };
                            BalanceChangeInfo {
                                account: entry.address.clone(),
                                before,
                                after,
                                change,
                            }
                        })
                        .collect();

                    // Fee is returned as negative by Bitcoin Core; use absolute value
                    let fee = detail.fee.map(|f| f.abs()).unwrap_or(0.0);

                    let response = TransactionDetailInfo {
                        signature: detail.txid,
                        slot: detail.block_height,
                        block_time: detail.block_time,
                        fee,
                        err: None,
                        balance_changes,
                    };
                    (StatusCode::OK, Json(ApiResponse::success(response)))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to get transaction: {}",
                        e
                    ))),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_forge_bitcoin_rpc::{
        BitcoinTransactionDetail, BitcoinTransactionInfo, BitcoinTxDetailEntry,
    };

    #[test]
    fn test_api_response_success() {
        let resp = ApiResponse::success("hello");
        assert!(resp.success);
        assert_eq!(resp.data, Some("hello"));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let resp: ApiResponse<String> = ApiResponse::error("something went wrong");
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_transaction_info_serialization() {
        let tx = TransactionInfo {
            signature: "abc123txid".to_string(),
            slot: 150,
            err: None,
            memo: None,
            block_time: Some(1700000000),
            confirmation_status: Some("6 confirmations".to_string()),
            account: "bcrt1qtest".to_string(),
        };

        let json = serde_json::to_value(&tx).unwrap();
        assert_eq!(json["signature"], "abc123txid");
        assert_eq!(json["slot"], 150);
        assert!(json["err"].is_null());
        assert_eq!(json["block_time"], 1700000000);
        assert_eq!(json["confirmation_status"], "6 confirmations");
        assert_eq!(json["account"], "bcrt1qtest");
    }

    #[test]
    fn test_transaction_detail_info_serialization() {
        let detail = TransactionDetailInfo {
            signature: "txid123".to_string(),
            slot: 200,
            block_time: Some(1700001000),
            fee: 0.00001,
            err: None,
            balance_changes: vec![
                BalanceChangeInfo {
                    account: "bcrt1qsender".to_string(),
                    before: 1.0,
                    after: 0.0,
                    change: -1.0,
                },
                BalanceChangeInfo {
                    account: "bcrt1qreceiver".to_string(),
                    before: 0.0,
                    after: 1.0,
                    change: 1.0,
                },
            ],
        };

        let json = serde_json::to_value(&detail).unwrap();
        assert_eq!(json["signature"], "txid123");
        assert_eq!(json["fee"], 0.00001);
        assert_eq!(json["balance_changes"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_bitcoin_tx_to_transaction_info_mapping() {
        // Simulate the mapping done in get_node_transactions for Bitcoin
        let btc_tx = BitcoinTransactionInfo {
            txid: "btctxid".to_string(),
            address: "bcrt1qknown".to_string(),
            category: "receive".to_string(),
            amount: 5.0,
            label: "account-0".to_string(),
            confirmations: 3,
            block_height: 101,
            block_time: Some(1700000000),
        };

        let confirmation_status = if btc_tx.confirmations > 0 {
            Some(format!("{} confirmations", btc_tx.confirmations))
        } else {
            Some("unconfirmed".to_string())
        };

        let mapped = TransactionInfo {
            signature: btc_tx.txid.clone(),
            slot: btc_tx.block_height,
            err: None,
            memo: None,
            block_time: btc_tx.block_time,
            confirmation_status,
            account: btc_tx.address.clone(),
        };

        assert_eq!(mapped.signature, "btctxid");
        assert_eq!(mapped.slot, 101);
        assert_eq!(mapped.block_time, Some(1700000000));
        assert_eq!(
            mapped.confirmation_status,
            Some("3 confirmations".to_string())
        );
        assert_eq!(mapped.account, "bcrt1qknown");
        assert!(mapped.err.is_none());
    }

    #[test]
    fn test_bitcoin_unconfirmed_tx_mapping() {
        let btc_tx = BitcoinTransactionInfo {
            txid: "mempooltx".to_string(),
            address: "bcrt1qaddr".to_string(),
            category: "receive".to_string(),
            amount: 1.0,
            label: "".to_string(),
            confirmations: 0,
            block_height: 0,
            block_time: None,
        };

        let confirmation_status = if btc_tx.confirmations > 0 {
            Some(format!("{} confirmations", btc_tx.confirmations))
        } else {
            Some("unconfirmed".to_string())
        };

        let mapped = TransactionInfo {
            signature: btc_tx.txid.clone(),
            slot: btc_tx.block_height,
            err: None,
            memo: None,
            block_time: btc_tx.block_time,
            confirmation_status,
            account: btc_tx.address.clone(),
        };

        assert_eq!(mapped.slot, 0);
        assert!(mapped.block_time.is_none());
        assert_eq!(mapped.confirmation_status, Some("unconfirmed".to_string()));
    }

    #[test]
    fn test_bitcoin_detail_to_balance_changes_mapping() {
        // Simulate the mapping done in get_transaction_detail for Bitcoin
        let detail = BitcoinTransactionDetail {
            txid: "abc123".to_string(),
            amount: -1.0,
            fee: Some(-0.00005),
            confirmations: 6,
            block_height: 110,
            block_time: Some(1700000000),
            details: vec![
                BitcoinTxDetailEntry {
                    address: "bcrt1qsender".to_string(),
                    category: "send".to_string(),
                    amount: -1.0,
                    label: Some("account-0".to_string()),
                },
                BitcoinTxDetailEntry {
                    address: "bcrt1qreceiver".to_string(),
                    category: "receive".to_string(),
                    amount: 1.0,
                    label: Some("account-1".to_string()),
                },
            ],
        };

        let balance_changes: Vec<BalanceChangeInfo> = detail
            .details
            .iter()
            .map(|entry| {
                let change = entry.amount;
                let (before, after) = if change >= 0.0 {
                    (0.0, change)
                } else {
                    (change.abs(), 0.0)
                };
                BalanceChangeInfo {
                    account: entry.address.clone(),
                    before,
                    after,
                    change,
                }
            })
            .collect();

        assert_eq!(balance_changes.len(), 2);

        // Sender: negative change
        assert_eq!(balance_changes[0].account, "bcrt1qsender");
        assert_eq!(balance_changes[0].change, -1.0);
        assert_eq!(balance_changes[0].before, 1.0);
        assert_eq!(balance_changes[0].after, 0.0);

        // Receiver: positive change
        assert_eq!(balance_changes[1].account, "bcrt1qreceiver");
        assert_eq!(balance_changes[1].change, 1.0);
        assert_eq!(balance_changes[1].before, 0.0);
        assert_eq!(balance_changes[1].after, 1.0);
    }

    #[test]
    fn test_bitcoin_fee_absolute_value() {
        // Bitcoin Core returns fees as negative values; we convert to absolute
        let btc_fee = Some(-0.00005_f64);
        let fee = btc_fee.map(|f| f.abs()).unwrap_or(0.0);
        assert_eq!(fee, 0.00005);

        let no_fee: Option<f64> = None;
        let fee = no_fee.map(|f| f.abs()).unwrap_or(0.0);
        assert_eq!(fee, 0.0);
    }

    #[test]
    fn test_transaction_dedup_and_sort() {
        // Simulate dedup + sort logic from get_node_transactions
        let mut transactions = vec![
            TransactionInfo {
                signature: "tx1".to_string(),
                slot: 100,
                err: None,
                memo: None,
                block_time: Some(1000),
                confirmation_status: Some("1 confirmations".to_string()),
                account: "addr1".to_string(),
            },
            TransactionInfo {
                signature: "tx2".to_string(),
                slot: 200,
                err: None,
                memo: None,
                block_time: Some(2000),
                confirmation_status: Some("1 confirmations".to_string()),
                account: "addr2".to_string(),
            },
            TransactionInfo {
                signature: "tx1".to_string(),
                slot: 100,
                err: None,
                memo: None,
                block_time: Some(1000),
                confirmation_status: Some("1 confirmations".to_string()),
                account: "addr3".to_string(),
            },
        ];

        // Sort by block_time descending
        transactions.sort_by(|a, b| b.block_time.cmp(&a.block_time));
        // Deduplicate by signature
        transactions.dedup_by(|a, b| a.signature == b.signature);

        assert_eq!(transactions.len(), 2);
        // Most recent first
        assert_eq!(transactions[0].signature, "tx2");
        assert_eq!(transactions[1].signature, "tx1");
    }
}
