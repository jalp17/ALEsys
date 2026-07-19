#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# ALEsys Agent Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/jalp17/ALEsys/master/scripts/install-agent.sh | bash
# Or:    ./install-agent.sh --server ws://host:3000/ws/agent --token mytoken
# =============================================================================

REPO="jalp17/ALEsys"
BINARY="alesys-agent"
INSTALL_DIR="${ALESYS_AGENT_DIR:-$HOME/.local/bin}"

# Parse args
SERVER=""
TOKEN=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --server) SERVER="$2"; shift 2 ;;
        --token) TOKEN="$2"; shift 2 ;;
        --dir) INSTALL_DIR="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--server ws://host:port/ws/agent] [--token TOKEN] [--dir /install/path]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Detect platform
detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) echo "Unsupported OS: $os"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) echo "Unsupported arch: $arch"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# Get latest version from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | cut -d'"' -f4
}

# Download binary
download_agent() {
    local version="$1"
    local platform="$2"
    local ext=""

    if [[ "$platform" == *"windows"* ]]; then
        ext=".exe"
    fi

    local asset_name="alesys-agent-${platform}${ext}"
    local url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"

    echo "Downloading ${asset_name}..."
    mkdir -p "$INSTALL_DIR"
    curl -fsSL -o "${INSTALL_DIR}/${BINARY}${ext}" "$url"
    chmod +x "${INSTALL_DIR}/${BINARY}${ext}"
    echo "Installed to: ${INSTALL_DIR}/${BINARY}${ext}"
}

# Create systemd service (Linux only)
create_systemd_service() {
    if [[ "$(uname -s)" != "Linux" ]] || ! command -v systemctl &>/dev/null; then
        return
    fi

    if [[ -z "$SERVER" ]] || [[ -z "$TOKEN" ]]; then
        return
    fi

    local service_dir="$HOME/.config/systemd/user"
    mkdir -p "$service_dir"

    cat > "${service_dir}/alesys-agent.service" << EOF
[Unit]
Description=ALEsys Remote Agent
After=network.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/${BINARY} --server ${SERVER} --token ${TOKEN}
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

    systemctl --user daemon-reload
    systemctl --user enable alesys-agent
    systemctl --user start alesys-agent

    echo "Systemd service installed and started."
    echo "Check status: systemctl --user status alesys-agent"
}

# Main
main() {
    echo "ALEsys Agent Installer"
    echo "====================="

    local platform
    platform="$(detect_platform)"
    echo "Platform: ${platform}"

    local version
    version="$(get_latest_version)"
    if [[ -z "$version" ]]; then
        echo "Could not determine latest version. Using master."
        version="master"
    fi
    echo "Version: ${version}"

    download_agent "$version" "$platform"

    if [[ -n "$SERVER" ]] && [[ -n "$TOKEN" ]]; then
        create_systemd_service
        echo ""
        echo "Agent started. Check logs: journalctl --user -u alesys-agent -f"
    else
        echo ""
        echo "To start the agent:"
        echo "  ${INSTALL_DIR}/${BINARY} --server ws://your-server:3000/ws/agent --token YOUR_TOKEN"
    fi

    echo ""
    echo "Done!"
}

main "$@"
