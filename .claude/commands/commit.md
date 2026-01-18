# PR Commit Agent

**Goal**: Create conventional commits with automated feedback loop for pre-commit hook failures.

## Core Principles

All commits must: Follow conventional commit format • Pass pre-commit hooks • Be descriptive and specific • Use automated fixes when possible

## Workflow

```bash
# 1. Analyze staged changes
git diff --cached --name-only && git diff --cached

# 2. Generate & execute commit
git commit -m "<type>[optional scope]: <description>"

# 3. If pre-commit fails → Fix issues → Re-stage → Re-commit
git add <files> && git commit -m "<message>"
```

```

## Conventional Commit Format

**Structure**: `<type>[optional scope]: <description>`

**Types**: feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert

**Examples**:

```bash
# ✅ Good
feat: add support for Solana program deployment
fix: resolve wallet connection timeout on cold start
fix: resolve the solana client init issues
refactor: decouple component A from B
test: add tests to component A
docs: update docs to reflect the changes of the Solana Client

# ❌ Bad (too vague, not imperative, or abbreviated)
Add some stuff
fixing bug
refactor code
tests
update docs
upgrade rn
```


## Quick Commands

```bash
git diff --cached --name-only                            # Check staged files
git diff --cached                                        # See staged changes
git commit -m "<type>[scope]: <description>"             # Commit
git add <files>                                          # Re-stage files
```

## Success Metrics

Conventional format • No lint/format errors • Descriptive message

