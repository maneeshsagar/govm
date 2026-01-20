#!/usr/bin/env bash
#
# govm installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/maneeshsagar/govm/main/install.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
GOVM_VERSION="${GOVM_VERSION:-latest}"
GOVM_ROOT="${GOVM_ROOT:-$HOME/.govm}"
GOVM_BIN="$GOVM_ROOT/bin"
GOVM_SHIMS="$GOVM_ROOT/shims"
GITHUB_REPO="maneeshsagar/govm"  # Update this with your actual repo

print_banner() {
    echo -e "${CYAN}"
    echo '   ____  _____   ____  __ '
    echo '  / ___||  _  | |  _ \|  |'
    echo ' | |  _ | | | | | |_) |  |'
    echo ' | |_| || |_| | |  __/|  |___'
    echo '  \____||_____| |_|   |_____|'
    echo -e "${NC}"
    echo -e "${BOLD}Go Version Manager - Installer${NC}"
    echo ""
}

info() {
    echo -e "${BLUE}→${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="linux";;
        Darwin*)    os="darwin";;
        MINGW*|MSYS*|CYGWIN*) os="windows";;
        *)          error "Unsupported operating system: $(uname -s)";;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64";;
        aarch64|arm64)  arch="arm64";;
        i386|i686)      arch="386";;
        *)              error "Unsupported architecture: $(uname -m)";;
    esac

    echo "${os}-${arch}"
}

detect_shell() {
    local shell_name
    shell_name=$(basename "$SHELL")
    echo "$shell_name"
}

get_shell_config() {
    local shell_name="$1"
    case "$shell_name" in
        bash)
            if [[ -f "$HOME/.bash_profile" ]]; then
                echo "$HOME/.bash_profile"
            else
                echo "$HOME/.bashrc"
            fi
            ;;
        zsh)
            echo "$HOME/.zshrc"
            ;;
        fish)
            echo "$HOME/.config/fish/config.fish"
            ;;
        *)
            echo "$HOME/.profile"
            ;;
    esac
}

create_directories() {
    info "Creating govm directories..."
    mkdir -p "$GOVM_BIN"
    mkdir -p "$GOVM_SHIMS"
    mkdir -p "$GOVM_ROOT/versions"
    success "Created $GOVM_ROOT"
}

download_binary() {
    local platform="$1"
    local tmp_dir
    tmp_dir=$(mktemp -d)
    local binary_name="govm"
    
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="govm.exe"
    fi

    info "Downloading govm for ${platform}..."
    
    # For now, we'll build from source or use a local binary
    # In production, this would download from GitHub releases:
    # local download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/govm-${platform}"
    
    # Check if we're running locally (for development)
    local local_binary="$HOME/Desktop/org-54/govm/target/release/govm"
    if [[ -f "$local_binary" ]]; then
        info "Using local build..."
        cp "$local_binary" "$GOVM_BIN/govm"
    else
        # Download from GitHub releases (update URL when published)
        local download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/govm-${platform}.tar.gz"
        
        if command -v curl &> /dev/null; then
            curl -fsSL "$download_url" -o "$tmp_dir/govm.tar.gz" || {
                error "Failed to download govm. Please check your internet connection."
            }
        elif command -v wget &> /dev/null; then
            wget -q "$download_url" -O "$tmp_dir/govm.tar.gz" || {
                error "Failed to download govm. Please check your internet connection."
            }
        else
            error "Neither curl nor wget found. Please install one of them."
        fi
        
        tar -xzf "$tmp_dir/govm.tar.gz" -C "$tmp_dir"
        cp "$tmp_dir/govm" "$GOVM_BIN/govm"
    fi
    
    chmod +x "$GOVM_BIN/govm"
    rm -rf "$tmp_dir"
    
    success "Downloaded govm binary"
}

create_shims() {
    info "Creating shims..."
    
    local govm_bin="$GOVM_BIN/govm"
    
    # Create shim for 'go'
    cat > "$GOVM_SHIMS/go" << EOF
#!/bin/sh
# Shim created by govm installer
exec "$govm_bin" exec "go" "\$@"
EOF
    chmod +x "$GOVM_SHIMS/go"
    
    # Create shim for 'gofmt'
    cat > "$GOVM_SHIMS/gofmt" << EOF
#!/bin/sh
# Shim created by govm installer
exec "$govm_bin" exec "gofmt" "\$@"
EOF
    chmod +x "$GOVM_SHIMS/gofmt"
    
    success "Created shims for go and gofmt"
}

configure_shell() {
    local shell_name="$1"
    local config_file="$2"
    
    info "Configuring shell ($shell_name)..."
    
    # Check if already configured
    if grep -q "GOVM_ROOT" "$config_file" 2>/dev/null; then
        warn "govm already configured in $config_file"
        return
    fi
    
    # Backup config file
    if [[ -f "$config_file" ]]; then
        cp "$config_file" "${config_file}.backup.$(date +%Y%m%d%H%M%S)"
    fi
    
    local config_content
    
    case "$shell_name" in
        fish)
            config_content='
# govm - Go Version Manager
set -gx GOVM_ROOT "$HOME/.govm"
fish_add_path "$GOVM_ROOT/shims"
fish_add_path "$GOVM_ROOT/bin"
'
            ;;
        *)
            config_content='
# govm - Go Version Manager
export GOVM_ROOT="$HOME/.govm"
export PATH="$GOVM_ROOT/shims:$GOVM_ROOT/bin:$PATH"
'
            ;;
    esac
    
    echo "$config_content" >> "$config_file"
    success "Added govm to $config_file"
}

verify_installation() {
    info "Verifying installation..."
    
    # Source the new PATH temporarily
    export PATH="$GOVM_SHIMS:$GOVM_BIN:$PATH"
    
    if command -v govm &> /dev/null; then
        success "govm installed successfully!"
        echo ""
        "$GOVM_BIN/govm" --version
    else
        error "Installation verification failed"
    fi
}

print_next_steps() {
    local shell_name="$1"
    local config_file="$2"
    
    echo ""
    echo -e "${BOLD}${GREEN}Installation complete!${NC}"
    echo ""
    echo -e "${BOLD}Next steps:${NC}"
    echo ""
    echo -e "  ${CYAN}1.${NC} Restart your terminal or run:"
    echo -e "     ${YELLOW}source $config_file${NC}"
    echo ""
    echo -e "  ${CYAN}2.${NC} Install a Go version:"
    echo -e "     ${YELLOW}govm use 1.22.0${NC}"
    echo ""
    echo -e "  ${CYAN}3.${NC} Verify installation:"
    echo -e "     ${YELLOW}go version${NC}"
    echo ""
    echo -e "${BOLD}Useful commands:${NC}"
    echo -e "  govm list-remote     List available Go versions"
    echo -e "  govm use <version>   Install & switch to a version"
    echo -e "  govm versions        List installed versions"
    echo -e "  govm --help          Show all commands"
    echo ""
    echo -e "${BLUE}Documentation:${NC} https://github.com/${GITHUB_REPO}"
    echo ""
}

main() {
    print_banner
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: $platform"
    
    # Detect shell
    local shell_name
    shell_name=$(detect_shell)
    local config_file
    config_file=$(get_shell_config "$shell_name")
    info "Detected shell: $shell_name ($config_file)"
    
    echo ""
    
    # Create directories
    create_directories
    
    # Download binary
    download_binary "$platform"
    
    # Create shims
    create_shims
    
    # Configure shell
    configure_shell "$shell_name" "$config_file"
    
    # Verify installation
    verify_installation
    
    # Print next steps
    print_next_steps "$shell_name" "$config_file"
}

# Run installer
main "$@"
