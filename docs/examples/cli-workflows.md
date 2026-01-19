# CLI Workflows

Common workflows using the `cf-solana` command-line tool.

## Development Workflow

```bash
# Start validator with generous resources
cf-solana start --accounts 20 --balance 500

# In another terminal, work with accounts
cf-solana accounts
cf-solana fund <ADDRESS> 100
```

## Testing Workflow

```bash
# Use fixed mnemonic for reproducibility
cf-solana start --mnemonic "test test test test test test test test test test test junk"

# Run tests
npm test
```

## CI/CD Workflow

```bash
# Lightweight configuration
cf-solana start --accounts 3 --balance 10 &
sleep 5
npm test
```

## Multi-Validator Workflow

```bash
# Terminal 1
cf-solana start --port 8899

# Terminal 2
cf-solana start --port 8900
```

## See Also

- [CLI Reference](../solana/cli)
- [Configuration](../solana/configuration)
