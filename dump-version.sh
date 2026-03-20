#!/bin/bash
#
# dump-version.sh - Prepare a new release version for gopher-mcp-rust
#
# Usage:
#   ./dump-version.sh [OPTIONS] [VERSION]
#
# Options:
#   --skip-crates   Skip publishing to crates.io (default: publish)
#   --dry-run       Show what would be done without making changes
#   --help          Show this help message
#
# Arguments:
#   VERSION - Optional. Format: X.Y.Z or X.Y.Z.E
#             If not provided, uses latest gopher-orch release version (X.Y.Z)
#             If provided as X.Y.Z.E, X.Y.Z must match gopher-orch version
#
# This script will:
#   1. Fetch latest version from gopher-orch releases
#   2. Validate and determine the target version
#   3. Update Cargo.toml version
#   4. Update CHANGELOG.md ([Unreleased] -> [X.Y.Z] - date)
#   5. Create git tag vX.Y.Z
#   6. Commit the changes
#   7. Publish to crates.io (unless --skip-crates is specified)
#
# After running this script:
#   1. Review the changes: git show HEAD
#   2. Push to release: git push origin br_release vX.Y.Z
#
# Environment variables:
#   CARGO_REGISTRY_TOKEN - crates.io API token (required for publishing)
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Files
CHANGELOG_FILE="CHANGELOG.md"
CARGO_TOML="Cargo.toml"

# Options
PUBLISH_CRATES=true
DRY_RUN=false
INPUT_VERSION=""

# Parse options
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-crates|--no-crates)
            PUBLISH_CRATES=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS] [VERSION]"
            echo ""
            echo "Options:"
            echo "  --skip-crates   Skip publishing to crates.io (default: publish)"
            echo "  --dry-run       Show what would be done without making changes"
            echo "  --help          Show this help message"
            echo ""
            echo "Arguments:"
            echo "  VERSION         Version to release (default: latest gopher-orch version)"
            echo "                  Format: X.Y.Z or X.Y.Z.E"
            echo ""
            echo "Examples:"
            echo "  $0                      # Release to GitHub and crates.io"
            echo "  $0 0.1.2                # Release specific version"
            echo "  $0 --skip-crates        # Release to GitHub only"
            echo "  $0 --dry-run            # Preview changes without executing"
            echo ""
            echo "Environment variables:"
            echo "  CARGO_REGISTRY_TOKEN    crates.io API token (required for publishing)"
            echo ""
            exit 0
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
        *)
            INPUT_VERSION="$1"
            shift
            ;;
    esac
done

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  gopher-mcp-rust Release Version Dump${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN MODE - No changes will be made${NC}"
    echo ""
fi

if [ "$PUBLISH_CRATES" = true ]; then
    echo -e "${CYAN}Publishing to: GitHub + crates.io${NC}"
else
    echo -e "${YELLOW}Publishing to: GitHub only (--skip-crates)${NC}"
fi
echo ""

# -----------------------------------------------------------------------------
# Step 1: Fetch latest gopher-orch version from GitHub releases
# -----------------------------------------------------------------------------
echo -e "${YELLOW}Step 1: Fetching latest gopher-orch version...${NC}"

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
    echo "Install it with: brew install gh"
    echo "Then authenticate: gh auth login"
    exit 1
fi

# Fetch latest release from gopher-orch using gh CLI (handles private repo auth)
GOPHER_ORCH_TAG=$(gh release view --repo GopherSecurity/gopher-orch --json tagName -q '.tagName' 2>/dev/null)

if [ -z "$GOPHER_ORCH_TAG" ]; then
    echo -e "${RED}Error: Could not fetch latest gopher-orch release${NC}"
    echo "Make sure you have access to GopherSecurity/gopher-orch repository."
    echo "Run 'gh auth login' to authenticate if needed."
    exit 1
fi

# Remove 'v' prefix if present (e.g., v0.1.1 -> 0.1.1)
GOPHER_ORCH_VERSION="${GOPHER_ORCH_TAG#v}"

if [ -z "$GOPHER_ORCH_VERSION" ]; then
    echo -e "${RED}Error: Could not parse gopher-orch version from release${NC}"
    exit 1
fi

echo -e "  Latest gopher-orch version: ${GREEN}$GOPHER_ORCH_VERSION${NC}"

# Validate gopher-orch version format (X.Y.Z)
if ! echo "$GOPHER_ORCH_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo -e "${RED}Error: gopher-orch version '$GOPHER_ORCH_VERSION' is not in X.Y.Z format${NC}"
    exit 1
fi

# -----------------------------------------------------------------------------
# Step 2: Determine target version
# -----------------------------------------------------------------------------
echo ""
echo -e "${YELLOW}Step 2: Determining target version...${NC}"

if [ -z "$INPUT_VERSION" ]; then
    # No argument provided, use gopher-orch version directly
    TARGET_VERSION="$GOPHER_ORCH_VERSION"
    CARGO_VERSION="$TARGET_VERSION"
    echo -e "  No version argument provided"
    echo -e "  Using gopher-orch version: ${GREEN}$TARGET_VERSION${NC}"
else
    # Version argument provided, validate it
    # Format should be X.Y.Z or X.Y.Z.E
    if echo "$INPUT_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
        # X.Y.Z format - must match gopher-orch exactly
        if [ "$INPUT_VERSION" != "$GOPHER_ORCH_VERSION" ]; then
            echo -e "${RED}Error: Version $INPUT_VERSION does not match gopher-orch version $GOPHER_ORCH_VERSION${NC}"
            exit 1
        fi
        TARGET_VERSION="$INPUT_VERSION"
        CARGO_VERSION="$TARGET_VERSION"
    elif echo "$INPUT_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$'; then
        # X.Y.Z.E format - first 3 parts must match gopher-orch
        INPUT_BASE=$(echo "$INPUT_VERSION" | sed -E 's/^([0-9]+\.[0-9]+\.[0-9]+)\.[0-9]+$/\1/')
        INPUT_EXT=$(echo "$INPUT_VERSION" | sed -E 's/^[0-9]+\.[0-9]+\.[0-9]+\.([0-9]+)$/\1/')
        if [ "$INPUT_BASE" != "$GOPHER_ORCH_VERSION" ]; then
            echo -e "${RED}Error: Version base $INPUT_BASE does not match gopher-orch version $GOPHER_ORCH_VERSION${NC}"
            echo "Extended version X.Y.Z.E must have X.Y.Z matching gopher-orch."
            exit 1
        fi
        TARGET_VERSION="$INPUT_VERSION"
        # Convert X.Y.Z.E to X.Y.Z-E for Cargo.toml (semver pre-release format)
        CARGO_VERSION="${INPUT_BASE}-${INPUT_EXT}"
        echo -e "  ${CYAN}Note: Cargo version will be ${CARGO_VERSION} (semver format)${NC}"
    else
        echo -e "${RED}Error: Invalid version format '$INPUT_VERSION'${NC}"
        echo "Expected format: X.Y.Z or X.Y.Z.E"
        exit 1
    fi
    echo -e "  Using provided version: ${GREEN}$TARGET_VERSION${NC}"
fi

TAG_VERSION="v$TARGET_VERSION"

# -----------------------------------------------------------------------------
# Step 3: Check if tag already exists
# -----------------------------------------------------------------------------
echo ""
echo -e "${YELLOW}Step 3: Checking existing tags...${NC}"

if git tag -l | grep -q "^$TAG_VERSION$"; then
    echo -e "${RED}Error: Tag $TAG_VERSION already exists${NC}"
    echo "If you want to re-release, delete the tag first:"
    echo "  git tag -d $TAG_VERSION"
    echo "  git push origin :refs/tags/$TAG_VERSION"
    exit 1
fi

echo -e "  Tag ${GREEN}$TAG_VERSION${NC} is available"

# -----------------------------------------------------------------------------
# Step 4: Update Cargo.toml version
# -----------------------------------------------------------------------------
echo ""
echo -e "${YELLOW}Step 4: Updating Cargo.toml...${NC}"

if [ ! -f "$CARGO_TOML" ]; then
    echo -e "${RED}Error: Cargo.toml not found${NC}"
    exit 1
fi

# Get current version (matches X.Y.Z or X.Y.Z-N format)
CURRENT_VERSION=$(grep -E '^version = "' "$CARGO_TOML" | head -1 | sed -E 's/version = "([^"]+)"/\1/')
echo -e "  Current version: ${YELLOW}$CURRENT_VERSION${NC}"

if [ "$DRY_RUN" = false ]; then
    # Update version in Cargo.toml (use CARGO_VERSION for semver compatibility)
    sed -i.bak -E "s/^version = \"[^\"]+\"/version = \"$CARGO_VERSION\"/" "$CARGO_TOML"
    rm -f "${CARGO_TOML}.bak"
fi

echo -e "  Updated to: ${GREEN}$CARGO_VERSION${NC}"

# -----------------------------------------------------------------------------
# Step 5: Check [Unreleased] section has content
# -----------------------------------------------------------------------------
echo ""
echo -e "${YELLOW}Step 5: Checking [Unreleased] section...${NC}"

if [ ! -f "$CHANGELOG_FILE" ]; then
    echo -e "${YELLOW}Warning: $CHANGELOG_FILE not found, creating one...${NC}"
    if [ "$DRY_RUN" = false ]; then
        cat > "$CHANGELOG_FILE" << EOF
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of gopher-mcp-rust SDK
- Rust bindings for gopher-orch native library
- OAuth 2.0 authentication support (feature-gated)
- MCP (Model Context Protocol) client implementation
- Runtime library loading via libloading

---

[Unreleased]: https://github.com/GopherSecurity/gopher-mcp-rust/compare/HEAD
EOF
    fi
fi

# Extract content between [Unreleased] and next ## section
UNRELEASED_CONTENT=$(sed -n '/^## \[Unreleased\]/,/^## \[/p' "$CHANGELOG_FILE" | \
    grep -v "^## \[" | grep -v "^$" | head -20)

if [ -z "$UNRELEASED_CONTENT" ]; then
    echo -e "${YELLOW}Warning: [Unreleased] section in CHANGELOG.md appears empty${NC}"
    echo "You may want to add release notes before continuing."
    if [ "$DRY_RUN" = false ]; then
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
else
    echo -e "  ${GREEN}[Unreleased] section has content${NC}"
    echo "  Preview:"
    echo "$UNRELEASED_CONTENT" | head -5 | sed 's/^/    /'
fi

# -----------------------------------------------------------------------------
# Step 6: Update CHANGELOG.md
# -----------------------------------------------------------------------------
echo ""
echo -e "${YELLOW}Step 6: Updating CHANGELOG.md...${NC}"

TODAY=$(date +%Y-%m-%d)
REPO_URL="https://github.com/GopherSecurity/gopher-mcp-rust"

if [ "$DRY_RUN" = false ]; then
    # Create backup
    cp "$CHANGELOG_FILE" "${CHANGELOG_FILE}.bak"

    # Find the line number of [Unreleased] header
    UNRELEASED_LINE=$(grep -n "^## \[Unreleased\]" "$CHANGELOG_FILE" | head -1 | cut -d: -f1)

    if [ -z "$UNRELEASED_LINE" ]; then
        echo -e "${RED}Error: Could not find [Unreleased] section in CHANGELOG.md${NC}"
        rm -f "${CHANGELOG_FILE}.bak"
        exit 1
    fi

    # Find the previous version for link generation
    PREV_VERSION=$(grep -E "^## \[[0-9]+\.[0-9]+\.[0-9]+" "$CHANGELOG_FILE" | head -1 | sed -E 's/^## \[([^]]+)\].*/\1/')

    # Check if there's a links section at the bottom (starts with --- or [Unreleased]:)
    HAS_LINKS_SECTION=$(grep -c "^\[Unreleased\]:" "$CHANGELOG_FILE" || true)

    # Find where links section starts (look for --- separator or [Unreleased]: link)
    if [ "$HAS_LINKS_SECTION" -gt 0 ]; then
        # Find the --- line before [Unreleased]: link, or the [Unreleased]: line itself
        LINKS_LINE=$(grep -n "^\[Unreleased\]:" "$CHANGELOG_FILE" | head -1 | cut -d: -f1)
        # Check if there's a --- separator before it
        SEPARATOR_LINE=$(grep -n "^---$" "$CHANGELOG_FILE" | tail -1 | cut -d: -f1)
        if [ -n "$SEPARATOR_LINE" ] && [ "$SEPARATOR_LINE" -lt "$LINKS_LINE" ]; then
            LINKS_LINE=$SEPARATOR_LINE
        fi
    else
        LINKS_LINE=""
    fi

    # Build new CHANGELOG content
    {
        # 1. Header section (everything before [Unreleased])
        head -n $((UNRELEASED_LINE - 1)) "$CHANGELOG_FILE"

        # 2. New [Unreleased] section (empty)
        echo "## [Unreleased]"
        echo ""

        # 3. New version section with today's date
        echo "## [$TARGET_VERSION] - $TODAY"

        # 4. Content after old [Unreleased] header until links section or EOF
        if [ -n "$LINKS_LINE" ]; then
            # Get content between [Unreleased] header and links section
            tail -n +$((UNRELEASED_LINE + 1)) "$CHANGELOG_FILE" | head -n $((LINKS_LINE - UNRELEASED_LINE - 1))
        else
            # No links section, get everything after [Unreleased] header
            tail -n +$((UNRELEASED_LINE + 1)) "$CHANGELOG_FILE"
        fi

        # 5. Add/Update links section
        echo ""
        echo "---"
        echo ""
        # [Unreleased] link pointing to compare from new version to HEAD
        echo "[Unreleased]: ${REPO_URL}/compare/v${TARGET_VERSION}...HEAD"
        # Add new version link
        if [ -n "$PREV_VERSION" ]; then
            echo "[$TARGET_VERSION]: ${REPO_URL}/compare/v${PREV_VERSION}...v${TARGET_VERSION}"
        else
            echo "[$TARGET_VERSION]: ${REPO_URL}/releases/tag/v${TARGET_VERSION}"
        fi
        # Keep existing version links (skip old [Unreleased] link and current version)
        if [ "$HAS_LINKS_SECTION" -gt 0 ]; then
            grep -E "^\[[0-9]+\.[0-9]+\.[0-9]+" "$CHANGELOG_FILE" | grep -v "^\[$TARGET_VERSION\]" || true
        fi
    } > "${CHANGELOG_FILE}.new"

    mv "${CHANGELOG_FILE}.new" "$CHANGELOG_FILE"
    rm -f "${CHANGELOG_FILE}.bak"
fi

echo -e "  ${GREEN}CHANGELOG.md updated${NC}"
echo -e "  [Unreleased] -> [$TARGET_VERSION] - $TODAY"

# -----------------------------------------------------------------------------
# Step 7: Commit changes and create tag
# -----------------------------------------------------------------------------
echo ""
echo -e "${YELLOW}Step 7: Committing changes and creating tag...${NC}"

if [ "$DRY_RUN" = false ]; then
    # Show what changed
    echo ""
    echo -e "${CYAN}Changes to be committed:${NC}"
    git diff --stat "$CARGO_TOML" "$CHANGELOG_FILE"

    echo ""
    echo -e "${CYAN}Committing...${NC}"

    git add "$CARGO_TOML" "$CHANGELOG_FILE"
    git commit -m "Release version $TARGET_VERSION

Prepare release v$TARGET_VERSION:
- Update Cargo.toml: version = \"$TARGET_VERSION\"
- Update CHANGELOG.md: [Unreleased] -> [$TARGET_VERSION] - $TODAY

gopher-orch version: $GOPHER_ORCH_VERSION

Changes in this release:
$(echo "$UNRELEASED_CONTENT" | head -10)
"

    # Create annotated tag
    echo ""
    echo -e "${CYAN}Creating tag $TAG_VERSION...${NC}"
    git tag -a "$TAG_VERSION" -m "Release $TARGET_VERSION

gopher-orch version: $GOPHER_ORCH_VERSION

Changes:
$(echo "$UNRELEASED_CONTENT" | head -15)
"
else
    echo -e "  ${YELLOW}[DRY RUN] Would commit Cargo.toml and CHANGELOG.md${NC}"
    echo -e "  ${YELLOW}[DRY RUN] Would create tag $TAG_VERSION${NC}"
fi

# -----------------------------------------------------------------------------
# Step 8: Publish to crates.io
# -----------------------------------------------------------------------------
echo ""
if [ "$PUBLISH_CRATES" = true ]; then
    echo -e "${YELLOW}Step 8: Publishing to crates.io...${NC}"

    if [ "$DRY_RUN" = true ]; then
        echo -e "  ${YELLOW}[DRY RUN] Would run: cargo publish${NC}"
        echo -e "  ${CYAN}Verifying package...${NC}"
        cargo publish --dry-run 2>&1 | head -20 || true
    else
        echo -e "  ${CYAN}Running cargo publish...${NC}"

        # Publish
        if cargo publish; then
            echo -e "  ${GREEN}Successfully published to crates.io${NC}"
        else
            echo -e "${RED}Error: Failed to publish to crates.io${NC}"
            echo "The git commit and tag were created. You can manually publish later with:"
            echo "  cargo publish"
            exit 1
        fi
    fi
else
    echo -e "${YELLOW}Step 8: Skipping crates.io publish (--skip-crates)${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Release preparation complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Version:           ${CYAN}$TARGET_VERSION${NC}"
if [ "$CARGO_VERSION" != "$TARGET_VERSION" ]; then
    echo -e "Cargo version:     ${CYAN}$CARGO_VERSION${NC} (semver)"
fi
echo -e "Tag:               ${CYAN}$TAG_VERSION${NC}"
echo -e "gopher-orch:       ${CYAN}$GOPHER_ORCH_VERSION${NC}"
if [ "$PUBLISH_CRATES" = true ]; then
    echo -e "crates.io:         ${GREEN}Published${NC}"
else
    echo -e "crates.io:         ${YELLOW}Skipped${NC}"
fi
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Review the commit: git show HEAD"
echo "  2. Push to release:   git push origin br_release $TAG_VERSION"
echo ""
echo -e "${CYAN}After pushing:${NC}"
echo "  - CI workflow will create GitHub Release"
echo "  - Native libraries will be attached to release"
if [ "$PUBLISH_CRATES" = false ]; then
    echo "  - Publish to crates.io manually: cargo publish"
fi
echo ""
echo -e "${CYAN}Users can install with:${NC}"
if [ "$PUBLISH_CRATES" = true ]; then
    echo ""
    echo "  # From crates.io"
    echo "  [dependencies]"
    echo "  gopher-orch = \"$CARGO_VERSION\""
fi
echo ""
echo "  # From GitHub"
echo "  [dependencies]"
echo "  gopher-orch = { git = \"https://github.com/GopherSecurity/gopher-mcp-rust.git\", tag = \"$TAG_VERSION\" }"
echo ""
