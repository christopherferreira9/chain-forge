# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-01

### Added

- Solana local validator management via `cf-solana` CLI
- Bitcoin node management via `cf-bitcoin` CLI
- TypeScript client packages (`@chain-forge/solana`, `@chain-forge/bitcoin`)
- BIP39/BIP44 account derivation for Solana and Bitcoin
- Multi-chain REST API server (`cf-api`)
- TOML-based configuration with profile support
- Program deployment support for Solana
- Development web dashboard
- VitePress documentation site

### Fixed

- Security audit configuration for Solana SDK transitive dependencies
- CI node version alignment
- ESLint configuration
