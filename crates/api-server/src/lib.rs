//! Chain Forge REST API Server
//!
//! This crate provides an HTTP REST API for monitoring and controlling
//! blockchain nodes managed by Chain Forge. Used by the web dashboard.

pub mod handlers;
pub mod routes;
pub mod server;

pub use server::start_server;
