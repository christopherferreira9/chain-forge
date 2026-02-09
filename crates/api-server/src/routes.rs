//! Route definitions for the Chain Forge REST API.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers;

/// Create the API router with all routes
pub fn create_routes() -> Router {
    Router::new()
        // Node listing and info
        .route("/api/v1/nodes", get(handlers::list_nodes))
        .route("/api/v1/nodes/{node_id}", get(handlers::get_node))
        .route(
            "/api/v1/nodes/{node_id}/accounts",
            get(handlers::get_node_accounts),
        )
        .route(
            "/api/v1/nodes/{node_id}/transactions",
            get(handlers::get_node_transactions),
        )
        .route(
            "/api/v1/nodes/{node_id}/transactions/{signature}",
            get(handlers::get_transaction_detail),
        )
        // Health check
        .route("/api/v1/health", post(handlers::health_check))
        // Node control
        .route("/api/v1/nodes", post(handlers::start_node))
        .route("/api/v1/nodes/{node_id}", delete(handlers::stop_node))
        .route("/api/v1/nodes/{node_id}/fund", post(handlers::fund_account))
        // Registry management
        .route("/api/v1/registry/cleanup", post(handlers::cleanup_registry))
}
