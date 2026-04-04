#!/bin/sh
# Headway CLI installer for Linux and macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/Code-Barru/headway/main/scripts/install.sh | sh
set -eu

REPO="Code-Barru/headway"
BINARY_NAME="headway"
DEFAULT_INSTALL_DIR="$HOME/.headway/bin"
GITHUB_API="https://api.github.com/repos/${REPO}/releases"
GITHUB_DOWNLOAD="https://github.com/${REPO}/releases/download"

# --- Defaults ---
INSTALL_DIR="${HEADWAY_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
VERSION="${HEADWAY_VERSION:-}"
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
Headway CLI installer

Usage:
    install.sh [OPTIONS]

Options:
    -y, --yes           Non-interactive mode (accept all defaults)
    -v, --version VER   Install a specific version (e.g. 0.3.0)
    -d, --dir DIR       Custom install directory (default: ~/.headway/bin)
    -h, --help          Show this help message

Environment variables:
    HEADWAY_VERSION       Same as --version
    HEADWAY_INSTALL_DIR   Same as --dir

Examples:
    curl -fsSL .../install.sh | sh
    curl -fsSL .../install.sh | sh -s -- --version 0.3.0
    curl -fsSL .../install.sh | sh -s -- --dir /usr/local/bin
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

# --- PATH configuration ---
configure_path() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            return 0
            ;;
    esac

    shell_name=$(basename "${SHELL:-/bin/sh}")

    case "$shell_name" in
        bash)
            if [ "$OS" = "macos" ]; then
                rc_file="$HOME/.bash_profile"
            else
                rc_file="$HOME/.bashrc"
            fi
            path_line="export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
        zsh)
            rc_file="$HOME/.zshrc"
            path_line="export PATH=\"${INSTALL_DIR}:\$PATH\""
            ;;
        fish)
            rc_file="${XDG_CONFIG_HOME:-$HOME/.config}/fish/config.fish"
            path_line="fish_add_path \"${INSTALL_DIR}\""
            ;;
        *)
            rc_file=""
            path_line=""
            ;;
    esac

    if [ -z "$rc_file" ]; then
        warn "Could not detect your shell configuration file."
        warn "Manually add ${INSTALL_DIR} to your PATH."
        return 0
    fi

    if [ -f "$rc_file" ] && grep -qF "$INSTALL_DIR" "$rc_file" 2>/dev/null; then
        return 0
    fi

    if [ "$YES" = "true" ]; then
        add_path="y"
    else
        printf "Add %s to PATH in %s? [Y/n] " "$INSTALL_DIR" "$rc_file"
        read -r add_path </dev/tty || add_path="y"
    fi

    case "$add_path" in
        [nN]*)
            warn "Skipping PATH configuration. Add it manually:"
            warn "  ${path_line}"
            ;;
        *)
            printf '\n# Headway CLI\n%s\n' "$path_line" >> "$rc_file"
            success "Added ${INSTALL_DIR} to PATH in ${rc_file}"
            info "Restart your shell or run: source ${rc_file}"
            ;;
    esac
}

# --- Main ---
main() {
    printf "${BOLD}Headway CLI Installer${RESET}\n\n"

    detect_platform
    detect_downloader
    resolve_version

    archive_name="${BINARY_NAME}-${TARGET}.${ARCHIVE_EXT}"
    download_url="${GITHUB_DOWNLOAD}/${TAG}/${archive_name}"

    info "Platform: ${OS} (${ARCH})"
    info "Target: ${TARGET}"

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
    mkdir -p "$INSTALL_DIR"
    cp "${tmpdir}/${BINARY_NAME}-${TARGET}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Verify
    if "${INSTALL_DIR}/${BINARY_NAME}" --version >/dev/null 2>&1; then
        installed_version=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>&1)
        success "Installed ${installed_version} to ${INSTALL_DIR}/${BINARY_NAME}"
    else
        success "Installed headway v${VERSION} to ${INSTALL_DIR}/${BINARY_NAME}"
    fi

    # Configure PATH
    configure_path

    printf "\nRun '${BOLD}headway --help${RESET}' to get started.\n"
}

main
