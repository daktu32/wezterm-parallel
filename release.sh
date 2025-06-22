#!/bin/bash

set -e

# WezTerm Parallel Development Framework Release Script
# Version: 1.0.0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
CURRENT_VERSION=""
NEW_VERSION=""
RELEASE_DIR="releases"
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Function to print colored output
print_color() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check prerequisites
check_prerequisites() {
    print_color "$YELLOW" "Checking prerequisites..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        print_color "$RED" "Error: Rust/Cargo not found"
        exit 1
    fi
    
    # Check for Git
    if ! command -v git &> /dev/null; then
        print_color "$RED" "Error: Git not found"
        exit 1
    fi
    
    # Check for GitHub CLI (optional but recommended)
    if ! command -v gh &> /dev/null; then
        print_color "$YELLOW" "Warning: GitHub CLI not found. Manual release upload required."
    fi
    
    print_color "$GREEN" "✓ Prerequisites checked"
}

# Function to get current version from Cargo.toml
get_current_version() {
    CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
    print_color "$BLUE" "Current version: $CURRENT_VERSION"
}

# Function to validate version format
validate_version() {
    local version=$1
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        print_color "$RED" "Error: Invalid version format. Use semantic versioning (e.g., 1.2.3 or 1.2.3-beta)"
        exit 1
    fi
}

# Function to update version in files
update_version() {
    local old_version=$1
    local new_version=$2
    
    print_color "$YELLOW" "Updating version from $old_version to $new_version..."
    
    # Update Cargo.toml
    sed -i.bak "s/^version = \"$old_version\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
    
    # Update Cargo.lock
    cargo update -p wezterm-multi-dev
    
    # Update install.sh
    if [ -f "install.sh" ]; then
        sed -i.bak "s/Version: $old_version/Version: $new_version/" install.sh
        rm install.sh.bak
    fi
    
    # Update this script
    sed -i.bak "s/Version: $old_version/Version: $new_version/" release.sh
    rm release.sh.bak
    
    print_color "$GREEN" "✓ Version updated in all files"
}

# Function to run tests
run_tests() {
    print_color "$YELLOW" "Running tests..."
    cargo test --release
    if [ $? -eq 0 ]; then
        print_color "$GREEN" "✓ All tests passed"
    else
        print_color "$RED" "✗ Tests failed. Aborting release."
        exit 1
    fi
}

# Function to build release binaries
build_release() {
    local version=$1
    
    print_color "$YELLOW" "Building release binaries..."
    
    # Clean previous builds
    cargo clean
    
    # Build release binary
    cargo build --release
    
    if [ $? -ne 0 ]; then
        print_color "$RED" "✗ Build failed"
        exit 1
    fi
    
    print_color "$GREEN" "✓ Release build successful"
}

# Function to create release artifacts
create_artifacts() {
    local version=$1
    local release_name="wezterm-parallel-${version}-${PLATFORM}-${ARCH}"
    local release_path="${RELEASE_DIR}/${release_name}"
    
    print_color "$YELLOW" "Creating release artifacts..."
    
    # Create release directory
    mkdir -p "$release_path"
    
    # Copy binary
    cp target/release/wezterm-multi-dev "$release_path/"
    
    # Copy configuration files
    mkdir -p "$release_path/config/templates"
    cp -r config/templates/* "$release_path/config/templates/" 2>/dev/null || true
    
    # Copy Lua scripts
    mkdir -p "$release_path/wezterm-config"
    cp -r wezterm-config/* "$release_path/wezterm-config/" 2>/dev/null || true
    
    # Copy documentation
    cp README.md "$release_path/"
    cp LICENSE "$release_path/" 2>/dev/null || true
    cp CHANGELOG.md "$release_path/" 2>/dev/null || true
    
    # Copy install script
    cp install.sh "$release_path/"
    chmod +x "$release_path/install.sh"
    
    # Create version file
    echo "$version" > "$release_path/VERSION"
    
    # Create tarball
    cd "$RELEASE_DIR"
    tar -czf "${release_name}.tar.gz" "${release_name}"
    cd ..
    
    # Create checksum
    cd "$RELEASE_DIR"
    if command -v sha256sum &> /dev/null; then
        sha256sum "${release_name}.tar.gz" > "${release_name}.tar.gz.sha256"
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "${release_name}.tar.gz" > "${release_name}.tar.gz.sha256"
    fi
    cd ..
    
    print_color "$GREEN" "✓ Release artifacts created: ${RELEASE_DIR}/${release_name}.tar.gz"
}

# Function to create git tag
create_git_tag() {
    local version=$1
    local tag_name="v${version}"
    
    print_color "$YELLOW" "Creating git tag $tag_name..."
    
    # Check if tag already exists
    if git rev-parse "$tag_name" >/dev/null 2>&1; then
        print_color "$RED" "Error: Tag $tag_name already exists"
        exit 1
    fi
    
    # Create annotated tag
    git tag -a "$tag_name" -m "Release version $version"
    
    print_color "$GREEN" "✓ Git tag created: $tag_name"
}

# Function to create GitHub release
create_github_release() {
    local version=$1
    local tag_name="v${version}"
    local release_name="wezterm-parallel-${version}-${PLATFORM}-${ARCH}"
    
    if ! command -v gh &> /dev/null; then
        print_color "$YELLOW" "Skipping GitHub release (gh CLI not found)"
        return
    fi
    
    print_color "$YELLOW" "Creating GitHub release..."
    
    # Create release notes
    local release_notes="## WezTerm Parallel Development Framework ${version}

### Release Date: $(date +%Y-%m-%d)

### Platform: ${PLATFORM}-${ARCH}

### Installation
\`\`\`bash
# Download and extract
curl -L https://github.com/YOUR_USERNAME/wezterm-parallel/releases/download/${tag_name}/${release_name}.tar.gz -o ${release_name}.tar.gz
tar -xzf ${release_name}.tar.gz
cd ${release_name}

# Run installer
./install.sh
\`\`\`

### What's New
- See CHANGELOG.md for details

### Checksums
- SHA256: See ${release_name}.tar.gz.sha256
"
    
    # Create draft release
    gh release create "$tag_name" \
        --title "Release ${version}" \
        --notes "$release_notes" \
        --draft \
        "${RELEASE_DIR}/${release_name}.tar.gz" \
        "${RELEASE_DIR}/${release_name}.tar.gz.sha256"
    
    if [ $? -eq 0 ]; then
        print_color "$GREEN" "✓ GitHub release draft created"
        print_color "$YELLOW" "  Visit GitHub to review and publish the release"
    else
        print_color "$RED" "✗ Failed to create GitHub release"
    fi
}

# Function to commit version changes
commit_version_changes() {
    local version=$1
    
    print_color "$YELLOW" "Committing version changes..."
    
    git add Cargo.toml Cargo.lock install.sh release.sh
    git commit -m "chore: bump version to ${version}

- Updated version in Cargo.toml
- Updated version in install.sh
- Updated version in release.sh
- Regenerated Cargo.lock"
    
    print_color "$GREEN" "✓ Version changes committed"
}

# Main release function
main() {
    print_color "$GREEN" "=== WezTerm Parallel Development Framework Release Tool ==="
    echo ""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                NEW_VERSION="$2"
                shift 2
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-tag)
                SKIP_TAG=true
                shift
                ;;
            --skip-github)
                SKIP_GITHUB=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --version VERSION    Set the new version (required)"
                echo "  --skip-tests         Skip running tests"
                echo "  --skip-tag           Skip creating git tag"
                echo "  --skip-github        Skip creating GitHub release"
                echo "  --help               Show this help message"
                echo ""
                echo "Example:"
                echo "  $0 --version 1.0.1"
                exit 0
                ;;
            *)
                print_color "$RED" "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Check if version is provided
    if [ -z "$NEW_VERSION" ]; then
        print_color "$RED" "Error: Version not specified. Use --version VERSION"
        exit 1
    fi
    
    # Validate version
    validate_version "$NEW_VERSION"
    
    # Run release steps
    check_prerequisites
    get_current_version
    
    # Confirm release
    print_color "$YELLOW" "Ready to release version $NEW_VERSION (current: $CURRENT_VERSION)"
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_color "$RED" "Release cancelled"
        exit 0
    fi
    
    # Update version
    update_version "$CURRENT_VERSION" "$NEW_VERSION"
    
    # Run tests unless skipped
    if [ "$SKIP_TESTS" != true ]; then
        run_tests
    fi
    
    # Build release
    build_release "$NEW_VERSION"
    
    # Create artifacts
    create_artifacts "$NEW_VERSION"
    
    # Commit changes
    commit_version_changes "$NEW_VERSION"
    
    # Create tag unless skipped
    if [ "$SKIP_TAG" != true ]; then
        create_git_tag "$NEW_VERSION"
    fi
    
    # Create GitHub release unless skipped
    if [ "$SKIP_GITHUB" != true ]; then
        create_github_release "$NEW_VERSION"
    fi
    
    echo ""
    print_color "$GREEN" "=== Release Complete ==="
    print_color "$YELLOW" "Version: $NEW_VERSION"
    print_color "$YELLOW" "Artifacts: ${RELEASE_DIR}/wezterm-parallel-${NEW_VERSION}-${PLATFORM}-${ARCH}.tar.gz"
    
    if [ "$SKIP_TAG" != true ]; then
        print_color "$YELLOW" "Git tag: v${NEW_VERSION}"
    fi
    
    echo ""
    print_color "$GREEN" "Next steps:"
    print_color "$YELLOW" "  1. Push commits: git push origin main"
    if [ "$SKIP_TAG" != true ]; then
        print_color "$YELLOW" "  2. Push tags: git push origin v${NEW_VERSION}"
    fi
    if [ "$SKIP_GITHUB" != true ] && command -v gh &> /dev/null; then
        print_color "$YELLOW" "  3. Review and publish the GitHub release draft"
    fi
    print_color "$YELLOW" "  4. Announce the release to users"
    echo ""
    print_color "$GREEN" "Release process completed successfully! 🎉"
}

# Run main function
main "$@"