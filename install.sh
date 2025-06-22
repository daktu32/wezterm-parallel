#!/bin/bash

set -e

# WezTerm Parallel Development Framework Installer
# Version: 1.0.0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default installation directory
DEFAULT_INSTALL_DIR="$HOME/.wezterm-parallel"

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
        print_color "$RED" "Error: Rust/Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    # Check for WezTerm
    if ! command -v wezterm &> /dev/null; then
        print_color "$RED" "Error: WezTerm not found. Please install WezTerm from https://wezfurlong.org/wezterm/installation"
        exit 1
    fi
    
    # Check for Git
    if ! command -v git &> /dev/null; then
        print_color "$RED" "Error: Git not found. Please install Git."
        exit 1
    fi
    
    print_color "$GREEN" "✓ All prerequisites met"
}

# Function to build the project
build_project() {
    print_color "$YELLOW" "Building WezTerm Parallel Development Framework..."
    
    # Build in release mode
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_color "$GREEN" "✓ Build successful"
    else
        print_color "$RED" "✗ Build failed"
        exit 1
    fi
}

# Function to install binaries
install_binaries() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    
    print_color "$YELLOW" "Installing to $install_dir..."
    
    # Create installation directory
    mkdir -p "$install_dir/bin"
    
    # Copy binary
    cp target/release/wezterm-multi-dev "$install_dir/bin/"
    chmod +x "$install_dir/bin/wezterm-multi-dev"
    
    print_color "$GREEN" "✓ Binary installed"
}

# Function to install configuration files
install_configs() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    
    print_color "$YELLOW" "Installing configuration files..."
    
    # Create config directories
    mkdir -p "$install_dir/config/templates"
    mkdir -p "$install_dir/lua"
    
    # Copy config templates
    cp -r config/templates/* "$install_dir/config/templates/" 2>/dev/null || true
    
    # Copy Lua scripts
    cp -r wezterm-config/* "$install_dir/lua/" 2>/dev/null || true
    
    # Create default config if not exists
    if [ ! -f "$install_dir/config/config.yaml" ]; then
        cat > "$install_dir/config/config.yaml" << EOF
# WezTerm Parallel Development Framework Configuration
version: "1.0"

# Network settings
network:
  bind_address: "127.0.0.1"
  port: 7600
  socket_path: "/tmp/wezterm-parallel.sock"

# Process management
process:
  max_processes: 10
  default_shell: "/bin/bash"
  startup_timeout: 5000  # ms

# Workspace settings
workspace:
  default_workspace: "default"
  max_workspaces: 5
  persistence_enabled: true
  persistence_path: "~/.wezterm-parallel/state"

# Dashboard settings
dashboard:
  enabled: true
  port: 7601
  refresh_interval: 1000  # ms

# Logging
logging:
  level: "info"
  file: "~/.wezterm-parallel/logs/wezterm-parallel.log"
  max_size: "10MB"
  max_backups: 3
EOF
    fi
    
    print_color "$GREEN" "✓ Configuration files installed"
}

# Function to setup shell integration
setup_shell_integration() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    
    print_color "$YELLOW" "Setting up shell integration..."
    
    # Detect shell
    local shell_name=$(basename "$SHELL")
    local rc_file=""
    
    case "$shell_name" in
        bash)
            rc_file="$HOME/.bashrc"
            ;;
        zsh)
            rc_file="$HOME/.zshrc"
            ;;
        fish)
            rc_file="$HOME/.config/fish/config.fish"
            ;;
        *)
            print_color "$YELLOW" "⚠ Unknown shell: $shell_name. Please add $install_dir/bin to your PATH manually."
            return
            ;;
    esac
    
    # Add to PATH if not already present
    if [ -f "$rc_file" ]; then
        if ! grep -q "$install_dir/bin" "$rc_file"; then
            echo "" >> "$rc_file"
            echo "# WezTerm Parallel Development Framework" >> "$rc_file"
            echo "export PATH=\"$install_dir/bin:\$PATH\"" >> "$rc_file"
            print_color "$GREEN" "✓ Added to PATH in $rc_file"
            print_color "$YELLOW" "  Please run: source $rc_file"
        else
            print_color "$GREEN" "✓ PATH already configured"
        fi
    fi
}

# Function to create systemd service (optional)
create_systemd_service() {
    local install_dir="${1:-$DEFAULT_INSTALL_DIR}"
    
    if [ "$2" != "--with-systemd" ]; then
        return
    fi
    
    print_color "$YELLOW" "Creating systemd service..."
    
    local service_file="$HOME/.config/systemd/user/wezterm-parallel.service"
    mkdir -p "$HOME/.config/systemd/user"
    
    cat > "$service_file" << EOF
[Unit]
Description=WezTerm Parallel Development Framework
After=network.target

[Service]
Type=simple
ExecStart=$install_dir/bin/wezterm-multi-dev server
Restart=on-failure
RestartSec=5
Environment="HOME=$HOME"
Environment="PATH=$PATH"

[Install]
WantedBy=default.target
EOF
    
    systemctl --user daemon-reload
    print_color "$GREEN" "✓ Systemd service created"
    print_color "$YELLOW" "  To enable: systemctl --user enable wezterm-parallel"
    print_color "$YELLOW" "  To start: systemctl --user start wezterm-parallel"
}

# Function to run tests
run_tests() {
    print_color "$YELLOW" "Running tests..."
    cargo test
    if [ $? -eq 0 ]; then
        print_color "$GREEN" "✓ All tests passed"
    else
        print_color "$RED" "✗ Some tests failed"
        exit 1
    fi
}

# Main installation function
main() {
    print_color "$GREEN" "=== WezTerm Parallel Development Framework Installer ==="
    echo ""
    
    # Parse arguments
    local install_dir="$DEFAULT_INSTALL_DIR"
    local with_systemd=false
    local skip_tests=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --prefix)
                install_dir="$2"
                shift 2
                ;;
            --with-systemd)
                with_systemd=true
                shift
                ;;
            --skip-tests)
                skip_tests=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --prefix DIR      Install to DIR (default: $DEFAULT_INSTALL_DIR)"
                echo "  --with-systemd    Create systemd user service"
                echo "  --skip-tests      Skip running tests"
                echo "  --help            Show this help message"
                exit 0
                ;;
            *)
                print_color "$RED" "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Run installation steps
    check_prerequisites
    
    if [ "$skip_tests" != true ]; then
        run_tests
    fi
    
    build_project
    install_binaries "$install_dir"
    install_configs "$install_dir"
    setup_shell_integration "$install_dir"
    
    if [ "$with_systemd" = true ]; then
        create_systemd_service "$install_dir"
    fi
    
    echo ""
    print_color "$GREEN" "=== Installation Complete ==="
    print_color "$YELLOW" "Installation directory: $install_dir"
    print_color "$YELLOW" "Binary: $install_dir/bin/wezterm-multi-dev"
    print_color "$YELLOW" "Config: $install_dir/config/config.yaml"
    echo ""
    print_color "$GREEN" "To get started:"
    print_color "$YELLOW" "  1. Reload your shell configuration or open a new terminal"
    print_color "$YELLOW" "  2. Run: wezterm-multi-dev --help"
    print_color "$YELLOW" "  3. Start the server: wezterm-multi-dev server"
    echo ""
    print_color "$GREEN" "Happy coding with WezTerm Parallel! 🚀"
}

# Run main function
main "$@"