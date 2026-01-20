#!/usr/bin/env bash
#
# govm uninstaller script
# Usage: curl -fsSL https://raw.githubusercontent.com/maneeshsagar/govm/main/uninstall.sh | bash
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

GOVM_ROOT="${GOVM_ROOT:-$HOME/.govm}"

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
}

remove_from_shell_config() {
    local config_file="$1"
    
    if [[ ! -f "$config_file" ]]; then
        return
    fi
    
    if grep -q "GOVM_ROOT" "$config_file" 2>/dev/null; then
        info "Removing govm from $config_file..."
        
        # Create backup
        cp "$config_file" "${config_file}.backup.$(date +%Y%m%d%H%M%S)"
        
        # Remove govm lines (handles both comment and export lines)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed
            sed -i '' '/# govm - Go Version Manager/d' "$config_file"
            sed -i '' '/GOVM_ROOT/d' "$config_file"
        else
            # GNU sed
            sed -i '/# govm - Go Version Manager/d' "$config_file"
            sed -i '/GOVM_ROOT/d' "$config_file"
        fi
        
        success "Removed govm from $config_file"
    fi
}

main() {
    echo -e "${BOLD}govm Uninstaller${NC}"
    echo ""
    
    if [[ ! -d "$GOVM_ROOT" ]]; then
        warn "govm directory not found (${GOVM_ROOT})"
        echo "Checking shell configs anyway..."
        echo ""
    else
        # Show what will be removed
        local versions_count=$(find "$GOVM_ROOT/versions" -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')
        versions_count=$((versions_count - 1))  # Subtract 1 for the directory itself
        
        echo -e "${YELLOW}This will remove:${NC}"
        echo "  • govm binary and shims"
        if [[ $versions_count -gt 0 ]]; then
            echo "  • $versions_count installed Go version(s)"
        fi
        echo "  • All data in $GOVM_ROOT"
        echo "  • Shell configuration for govm"
        echo ""
    fi
    
    read -p "Are you sure you want to uninstall govm? [y/N] " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Uninstall cancelled"
        exit 0
    fi
    
    echo ""
    
    # Remove from shell configs
    remove_from_shell_config "$HOME/.zshrc"
    remove_from_shell_config "$HOME/.bashrc"
    remove_from_shell_config "$HOME/.bash_profile"
    remove_from_shell_config "$HOME/.profile"
    remove_from_shell_config "$HOME/.config/fish/config.fish"
    
    # Remove govm directory
    if [[ -d "$GOVM_ROOT" ]]; then
        info "Removing $GOVM_ROOT..."
        rm -rf "$GOVM_ROOT"
        success "Removed govm directory and all Go versions"
    fi
    
    echo ""
    success "govm has been completely uninstalled!"
    echo ""
    echo -e "  Restart your terminal or run: ${YELLOW}exec \$SHELL${NC}"
    echo ""
}

main "$@"
