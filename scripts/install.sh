#!/bin/sh
# Gapline CLI installer for Linux and macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/Code-Barru/gapline/main/scripts/install.sh | sh
set -eu

REPO="Code-Barru/gapline"
BINARY_NAME="gapline"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"
GITHUB_DOWNLOAD="https://github.com/${REPO}/releases/download"

# --- Defaults ---
INSTALL_DIR="${GAPLINE_INSTALL_DIR:-/usr/local/bin}"
VERSION="${GAPLINE_VERSION:-}"
YES="false"

# --- Color helpers ---
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    RESET=''
fi

info() {
    printf "${BLUE}info${RESET} %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${RESET} %s\n" "$1"
}

error() {
    printf "${RED}error${RESET} %s\n" "$1" >&2
}

success() {
    printf "${GREEN}done${RESET} %s\n" "$1"
}

die() {
    error "$1"
    exit 1
}

# --- Usage ---
usage() {
    cat <<EOF
Gapline CLI installer

Usage:
    install.sh [OPTIONS]

Options:
    -y, --yes           Non-interactive mode (accept all defaults)
    -v, --version VER   Install a specific version (e.g. 0.3.0)
    -d, --dir DIR       Custom install directory (default: /usr/local/bin)
    -h, --help          Show this help message

Environment variables:
    GAPLINE_VERSION       Same as --version
    GAPLINE_INSTALL_DIR   Same as --dir

Examples:
    curl -fsSL .../install.sh | sh
    curl -fsSL .../install.sh | sh -s -- --version 0.3.0
    curl -fsSL .../install.sh | sh -s -- --dir ~/.local/bin
EOF
    exit 0
}

# --- Argument parsing ---
while [ $# -gt 0 ]; do
    case "$1" in
        -y|--yes)
            YES="true"
            shift
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            die "Unknown option: $1 (use --help for usage)"
            ;;
    esac
done

# Auto non-interactive when piped
if [ ! -t 0 ]; then
    YES="true"
fi

# --- OS and architecture detection ---
detect_platform() {
    os=$(uname -s)
    case "$os" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="macos" ;;
        *)       die "Unsupported operating system: $os" ;;
    esac

    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)   ARCH="x86_64" ;;
        aarch64|arm64)  ARCH="aarch64" ;;
        *)              die "Unsupported architecture: $arch" ;;
    esac

    if [ "$OS" = "linux" ]; then
        TARGET="${ARCH}-unknown-linux-gnu"
        ARCHIVE_EXT="tar.gz"
    elif [ "$OS" = "macos" ]; then
        TARGET="${ARCH}-apple-darwin"
        ARCHIVE_EXT="tar.gz"
    fi
}

# --- HTTP downloader detection ---
detect_downloader() {
    if command -v curl >/dev/null 2>&1; then
        DOWNLOADER="curl"
    elif command -v wget >/dev/null 2>&1; then
        DOWNLOADER="wget"
    else
        die "Neither curl nor wget found. Please install one of them."
    fi
}

download_to_file() {
    url="$1"
    dest="$2"
    if [ "$DOWNLOADER" = "curl" ]; then
        curl -fsSL -o "$dest" "$url"
    else
        wget -q -O "$dest" "$url"
    fi
}

download_to_stdout() {
    url="$1"
    if [ "$DOWNLOADER" = "curl" ]; then
        curl -fsSL "$url"
    else
        wget -q -O - "$url"
    fi
}

# --- Version resolution ---
resolve_version() {
    if [ -n "$VERSION" ]; then
        TAG="cli@v${VERSION}"
        info "Using specified version: ${VERSION}"
    else
        info "Fetching latest version..."
        response=$(download_to_stdout "$GITHUB_API" 2>&1) || {
            die "Failed to fetch releases from GitHub API. Try passing --version explicitly."
        }
        TAG=$(printf '%s' "$response" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\(cli@[^"]*\)".*/\1/p' | head -1)
        if [ -z "$TAG" ]; then
            die "Could not find a CLI release. Try passing --version explicitly."
        fi
        VERSION=$(printf '%s' "$TAG" | sed 's/^cli@v//')
        info "Latest version: ${VERSION}"
    fi
}

# --- Checksum verification ---
verify_checksum() {
    archive_path="$1"
    archive_name="$2"
    checksum_url="${GITHUB_DOWNLOAD}/${TAG}/checksums.sha256"

    checksum_file="${tmpdir}/checksums.sha256"
    if download_to_file "$checksum_url" "$checksum_file" 2>/dev/null; then
        expected=$(grep "$archive_name" "$checksum_file" | cut -d ' ' -f 1)
        if [ -z "$expected" ]; then
            warn "Archive not found in checksums file, skipping verification."
            return 0
        fi

        if command -v sha256sum >/dev/null 2>&1; then
            actual=$(sha256sum "$archive_path" | cut -d ' ' -f 1)
        elif command -v shasum >/dev/null 2>&1; then
            actual=$(shasum -a 256 "$archive_path" | cut -d ' ' -f 1)
        else
            warn "No SHA-256 tool found, skipping checksum verification."
            return 0
        fi

        if [ "$actual" != "$expected" ]; then
            die "Checksum verification failed! Expected: ${expected}, Got: ${actual}"
        fi
        success "Checksum verified."
    else
        warn "Checksums file not available for this release, skipping verification."
    fi
}

# --- Sudo helper ---
elevate() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    elif command -v doas >/dev/null 2>&1; then
        doas "$@"
    else
        die "Cannot write to ${INSTALL_DIR}. Run as root or use --dir to pick a writable location."
    fi
}

# --- Install binary ---
install_binary() {
    src="${tmpdir}/${BINARY_NAME}-${TARGET}/${BINARY_NAME}"

    if [ -w "$INSTALL_DIR" ] || { [ -w "$(dirname "$INSTALL_DIR")" ] && [ ! -e "$INSTALL_DIR" ]; }; then
        mkdir -p "$INSTALL_DIR"
        cp "$src" "${INSTALL_DIR}/${BINARY_NAME}"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    else
        info "Elevated privileges required to install to ${INSTALL_DIR}"
        elevate mkdir -p "$INSTALL_DIR"
        elevate cp "$src" "${INSTALL_DIR}/${BINARY_NAME}"
        elevate chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    fi
}

# --- Main ---
main() {
    printf "${BOLD}Gapline CLI Installer${RESET}\n\n"

    detect_platform
    detect_downloader
    resolve_version

    archive_name="${BINARY_NAME}-${TARGET}.${ARCHIVE_EXT}"
    download_url="${GITHUB_DOWNLOAD}/${TAG}/${archive_name}"

    info "Platform: ${OS} (${ARCH})"
    info "Target: ${TARGET}"
    info "Install directory: ${INSTALL_DIR}"

    # Create temp directory with cleanup trap
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    # Download
    info "Downloading ${archive_name}..."
    download_to_file "$download_url" "${tmpdir}/${archive_name}" || \
        die "Download failed. Check that version ${VERSION} exists for target ${TARGET}."

    # Verify checksum
    verify_checksum "${tmpdir}/${archive_name}" "$archive_name"

    # Extract
    info "Extracting..."
    tar xzf "${tmpdir}/${archive_name}" -C "$tmpdir"

    # Install
    install_binary

    # Verify
    if "${INSTALL_DIR}/${BINARY_NAME}" --version >/dev/null 2>&1; then
        installed_version=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>&1)
        success "Installed ${installed_version} to ${INSTALL_DIR}/${BINARY_NAME}"
    else
        success "Installed gapline v${VERSION} to ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    printf "\nRun '${BOLD}gapline --help${RESET}' to get started.\n"
}

main
