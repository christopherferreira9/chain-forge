#!/usr/bin/env bash
set -euo pipefail

# ─── Colors ──────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

info()  { echo -e "${BLUE}ℹ${NC} $1"; }
ok()    { echo -e "${GREEN}✓${NC} $1"; }
warn()  { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# ─── Step 1: Pre-flight checks ──────────────────────────────────

echo ""
echo -e "${BOLD}Chain Forge Release${NC}"
echo "─────────────────────────────────────"
echo ""

# Check required tools
command -v jq >/dev/null 2>&1 || error "jq is required. Install with: brew install jq"
command -v cargo >/dev/null 2>&1 || error "cargo is required."
command -v git-cliff >/dev/null 2>&1 || error "git-cliff is required. Install with: cargo install git-cliff"

# Must be on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    error "Must be on 'main' branch. Currently on '$CURRENT_BRANCH'."
fi

# Working tree must be clean
if [[ -n "$(git status --porcelain)" ]]; then
    error "Working tree is dirty. Commit or stash changes first."
fi

# Fetch and verify up-to-date with remote
info "Fetching latest from origin..."
git fetch origin main --quiet
LOCAL=$(git rev-parse main)
REMOTE=$(git rev-parse origin/main)
if [[ "$LOCAL" != "$REMOTE" ]]; then
    error "Local main is not up to date with origin/main. Run: git pull origin main"
fi
ok "Local main is up to date with origin/main"

# ─── Step 2: Determine version ───────────────────────────────────

CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
echo ""
echo -e "Current version: ${BOLD}${CURRENT_VERSION}${NC}"
echo ""
read -rp "Enter new version (e.g., 0.2.0): " NEW_VERSION

# Validate semver format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    error "Invalid semver format. Expected X.Y.Z"
fi

if [[ "$NEW_VERSION" == "$CURRENT_VERSION" ]]; then
    error "New version must differ from current version ($CURRENT_VERSION)."
fi

echo ""
ok "Version will be bumped: ${CURRENT_VERSION} → ${NEW_VERSION}"

# ─── Step 3: Select packages to publish ──────────────────────────

echo ""
echo -e "${BOLD}Select packages to publish:${NC}"
echo ""

read -rp "  Publish Rust crates to crates.io? [Y/n] " PUBLISH_RUST
PUBLISH_RUST=${PUBLISH_RUST:-Y}
[[ "$PUBLISH_RUST" =~ ^[Yy] ]] && PUBLISH_RUST=true || PUBLISH_RUST=false

read -rp "  Publish @chain-forge/solana to npm? [Y/n] " PUBLISH_NPM_SOLANA
PUBLISH_NPM_SOLANA=${PUBLISH_NPM_SOLANA:-Y}
[[ "$PUBLISH_NPM_SOLANA" =~ ^[Yy] ]] && PUBLISH_NPM_SOLANA=true || PUBLISH_NPM_SOLANA=false

read -rp "  Publish @chain-forge/bitcoin to npm? [Y/n] " PUBLISH_NPM_BITCOIN
PUBLISH_NPM_BITCOIN=${PUBLISH_NPM_BITCOIN:-Y}
[[ "$PUBLISH_NPM_BITCOIN" =~ ^[Yy] ]] && PUBLISH_NPM_BITCOIN=true || PUBLISH_NPM_BITCOIN=false

read -rp "  Build cf-solana binary for GitHub release? [Y/n] " BUILD_CF_SOLANA
BUILD_CF_SOLANA=${BUILD_CF_SOLANA:-Y}
[[ "$BUILD_CF_SOLANA" =~ ^[Yy] ]] && BUILD_CF_SOLANA=true || BUILD_CF_SOLANA=false

read -rp "  Build cf-bitcoin binary for GitHub release? [Y/n] " BUILD_CF_BITCOIN
BUILD_CF_BITCOIN=${BUILD_CF_BITCOIN:-Y}
[[ "$BUILD_CF_BITCOIN" =~ ^[Yy] ]] && BUILD_CF_BITCOIN=true || BUILD_CF_BITCOIN=false

# ─── Step 4: Confirmation ────────────────────────────────────────

echo ""
echo -e "${BOLD}Release summary:${NC}"
echo "  Version:              v${NEW_VERSION}"
echo "  Rust crates:          ${PUBLISH_RUST}"
echo "  @chain-forge/solana:  ${PUBLISH_NPM_SOLANA}"
echo "  @chain-forge/bitcoin: ${PUBLISH_NPM_BITCOIN}"
echo "  cf-solana binary:     ${BUILD_CF_SOLANA}"
echo "  cf-bitcoin binary:    ${BUILD_CF_BITCOIN}"
echo ""
read -rp "Proceed? [Y/n] " CONFIRM
CONFIRM=${CONFIRM:-Y}
[[ "$CONFIRM" =~ ^[Yy] ]] || { echo "Aborted."; exit 0; }

# ─── Step 5: Create release branch ──────────────────────────────

RELEASE_BRANCH="release/v${NEW_VERSION}"
echo ""
info "Creating branch: ${RELEASE_BRANCH}"
git checkout -b "$RELEASE_BRANCH"

# ─── Step 6: Bump versions ──────────────────────────────────────

info "Bumping workspace version to ${NEW_VERSION}..."

# Update root Cargo.toml workspace version
sed -i '' "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml

# Update npm package versions
for pkg_json in npm/@chain-forge/solana/package.json npm/@chain-forge/bitcoin/package.json; do
    if [[ -f "$pkg_json" ]]; then
        jq --arg v "$NEW_VERSION" '.version = $v' "$pkg_json" > "${pkg_json}.tmp"
        mv "${pkg_json}.tmp" "$pkg_json"
        ok "Updated ${pkg_json}"
    fi
done

# Verify the workspace compiles
info "Running cargo check..."
if ! cargo check --workspace 2>/dev/null; then
    error "cargo check failed after version bump. Please fix and retry."
fi
ok "Workspace compiles successfully"

# ─── Step 7: Update CHANGELOG ───────────────────────────────────

info "Generating changelog with git-cliff..."
git-cliff --tag "v${NEW_VERSION}" -o CHANGELOG.md

echo ""
warn "Opening CHANGELOG.md for review. Edit as needed, then save and close."
echo ""
read -rp "Press Enter to open editor..."
${EDITOR:-vim} CHANGELOG.md

# ─── Step 8: Write release manifest ─────────────────────────────

info "Writing .release.json..."
cat > .release.json << EOF
{
  "version": "${NEW_VERSION}",
  "rust_crates": ${PUBLISH_RUST},
  "npm_solana": ${PUBLISH_NPM_SOLANA},
  "npm_bitcoin": ${PUBLISH_NPM_BITCOIN},
  "binaries": {
    "cf-solana": ${BUILD_CF_SOLANA},
    "cf-bitcoin": ${BUILD_CF_BITCOIN}
  }
}
EOF
ok "Release manifest written"

# ─── Step 9: Commit and push ────────────────────────────────────

info "Committing changes..."
git add Cargo.toml Cargo.lock CHANGELOG.md .release.json \
    npm/@chain-forge/solana/package.json \
    npm/@chain-forge/bitcoin/package.json

git commit -m "$(cat <<HEREDOC
chore(release): prepare v${NEW_VERSION}

Packages to publish:
- Rust crates: ${PUBLISH_RUST}
- @chain-forge/solana (npm): ${PUBLISH_NPM_SOLANA}
- @chain-forge/bitcoin (npm): ${PUBLISH_NPM_BITCOIN}
- cf-solana binary: ${BUILD_CF_SOLANA}
- cf-bitcoin binary: ${BUILD_CF_BITCOIN}
HEREDOC
)"

info "Pushing to origin..."
git push -u origin "$RELEASE_BRANCH"

# ─── Step 10: Create PR ─────────────────────────────────────────

if command -v gh >/dev/null 2>&1; then
    echo ""
    info "Creating pull request..."

    PR_BODY="$(cat <<HEREDOC
## Release v${NEW_VERSION}

### Packages
| Package | Publish |
|---------|---------|
| Rust crates (crates.io) | ${PUBLISH_RUST} |
| @chain-forge/solana (npm) | ${PUBLISH_NPM_SOLANA} |
| @chain-forge/bitcoin (npm) | ${PUBLISH_NPM_BITCOIN} |
| cf-solana binary | ${BUILD_CF_SOLANA} |
| cf-bitcoin binary | ${BUILD_CF_BITCOIN} |

### Checklist
- [ ] CHANGELOG.md reviewed
- [ ] Version numbers correct
- [ ] CI checks pass

> Merging this PR will trigger the release workflow automatically.
HEREDOC
)"

    gh pr create \
        --title "Release v${NEW_VERSION}" \
        --body "$PR_BODY" \
        --base main

    ok "Pull request created!"
else
    echo ""
    warn "gh CLI not found. Create the PR manually:"
    echo "  Title: Release v${NEW_VERSION}"
    echo "  Base:  main"
    echo "  Head:  ${RELEASE_BRANCH}"
fi

echo ""
echo -e "${GREEN}${BOLD}Release v${NEW_VERSION} prepared successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review the PR"
echo "  2. Ensure CI checks pass"
echo "  3. Merge to main to trigger the release workflow"
