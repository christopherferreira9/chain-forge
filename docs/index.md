---
layout: home

hero:
  name: "Chain Forge"
  text: "Multi-Chain Development Tool Suite"
  tagline: Foundry-inspired local blockchain development for Solana and beyond
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/christopherferreira9/chain-forge

features:
  - icon: ⚡
    title: Rapid Local Development
    details: Spin up local blockchain test validators with pre-funded accounts in seconds. Focus on building, not infrastructure.

  - icon: 🔗
    title: Multi-Chain Ready
    details: Built with a unified ChainProvider interface. Currently supports Solana with more chains coming soon.

  - icon: 🛠️
    title: Powerful CLI
    details: Simple commands to start validators, manage accounts, fund wallets, and more. Designed for developer productivity.

  - icon: 📦
    title: TypeScript SDK
    details: First-class TypeScript support with @chain-forge/solana package for seamless integration into your projects.

  - icon: 🔑
    title: BIP39/BIP44 Accounts
    details: Generate deterministic wallets using standard mnemonics and derivation paths. Compatible with popular wallet software.

  - icon: ⚙️
    title: Flexible Configuration
    details: TOML-based configuration with multiple profiles. Global or project-level settings to match your workflow.
---

## Quick Start

Install Solana CLI tools first:

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

Then install Chain Forge:

```bash
cargo install chain-forge-solana-cli
```

Start a local validator with pre-funded accounts:

```bash
cf-solana start --accounts 5 --balance 100
```

## Use Cases

- **Smart Contract Development**: Test Solana programs locally before deploying to devnet or mainnet
- **dApp Development**: Build frontends against a local blockchain with controlled state
- **CI/CD Testing**: Integrate into automated testing pipelines for reproducible environments
- **Learning**: Experiment with blockchain development without spending real tokens

## Why Chain Forge?

Chain Forge brings the ergonomics of Ethereum's Foundry to multi-chain development. It wraps official blockchain tooling (like `solana-test-validator`) with a unified, developer-friendly interface that works across different chains.

No need to remember chain-specific commands or deal with inconsistent tooling. One CLI, one configuration format, one way of managing local development across all your blockchain projects.
