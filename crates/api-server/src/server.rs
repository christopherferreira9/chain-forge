//! HTTP server setup for the Chain Forge REST API.

use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use crate::routes::create_routes;

/// Start the API server on the specified port
pub async fn start_server(port: u16) -> eyre::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_routes().layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!(
        "🚀 Chain Forge API Server starting on http://localhost:{}",
        port
    );
    println!("   API endpoints:");
    println!("   - GET    /api/v1/nodes                   - List all nodes");
    println!("   - GET    /api/v1/nodes/{{node_id}}         - Get specific node");
    println!("   - GET    /api/v1/nodes/{{node_id}}/accounts     - Get node accounts");
    println!("   - GET    /api/v1/nodes/{{node_id}}/transactions - Get node transactions");
    println!("   - POST   /api/v1/health                        - Health check all nodes");
    println!("   - POST   /api/v1/nodes                   - Start a new node");
    println!("   - DELETE /api/v1/nodes/{{node_id}}         - Stop a node");
    println!("   - POST   /api/v1/nodes/{{node_id}}/fund    - Fund an account");
    println!("   - POST   /api/v1/registry/cleanup         - Remove non-running nodes");
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
