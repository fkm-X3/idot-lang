#!/usr/bin/env bash
set -euo pipefail

REPO="fkm-X3/idot-lang"
INSTALL_DIR="${IDOT_HOME:-"$HOME/.idot"}"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="${IDOT_VERSION:-latest}"

say() { printf "\033[1;32m==>\033[0m %s\n" "$*"; }
err() { printf "\033[1;31m==>\033[0m %s\n" "$*" >&2; }

detect_platform() {
    local kernel arch
    kernel=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$kernel" in
        linux)  os="linux"  ;;
        darwin) os="macos"  ;;
        *)      err "unsupported OS: $kernel"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "unsupported architecture: $arch"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

download_url() {
    local platform=$1 ext

    if command -v curl &>/dev/null; then
        dl() { curl -fsSL "$1" -o "$2"; }
    elif command -v wget &>/dev/null; then
        dl() { wget -q "$1" -O "$2"; }
    else
        err "need curl or wget to download"; exit 1
    fi

    if [[ "$platform" == windows-* ]]; then
        ext="zip"
    else
        ext="tar.gz"
    fi

    if [[ "$VERSION" == "latest" ]]; then
        api_url="https://api.github.com/repos/$REPO/releases/latest"
        tag=$(curl -fsSL "$api_url" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": "//;s/".*//')
        if [[ -z "$tag" ]]; then
            err "could not determine latest release"; exit 1
        fi
    else
        tag="$VERSION"
    fi

    echo "https://github.com/$REPO/releases/download/$tag/idot-${tag}-${platform}.${ext}"
}

install_binaries() {
    local platform=$1 url=$2 tmp
    tmp=$(mktemp -d)
    mkdir -p "$BIN_DIR"

    say "downloading $url"
    dl "$url" "$tmp/idot-archive"

    pushd "$tmp" &>/dev/null
    if [[ "$url" == *.zip ]]; then
        unzip -q idot-archive
    else
        tar xzf idot-archive
    fi
    find "$tmp" -name 'idot' -type f -exec cp {} "$BIN_DIR/" \; 2>/dev/null || true
    find "$tmp" -name 'matrix' -type f -exec cp {} "$BIN_DIR/" \; 2>/dev/null || true
    popd &>/dev/null

    chmod +x "$BIN_DIR/idot" "$BIN_DIR/matrix" 2>/dev/null || true
    rm -rf "$tmp"

    say "installed to $BIN_DIR"
}

update_shell() {
    local rc line
    case "$SHELL" in
        *zsh)  rc="$HOME/.zshrc"  ;;
        *bash) rc="$HOME/.bashrc" ;;
        *fish) rc="$HOME/.config/fish/config.fish" ;;
        *)     rc="$HOME/.profile" ;;
    esac

    line='export PATH="$HOME/.idot/bin:$PATH"'

    if ! grep -qxF "$line" "$rc" 2>/dev/null; then
        echo "" >> "$rc"
        echo "$line" >> "$rc"
        say "added ~/.idot/bin to PATH in $rc"
    fi
}

main() {
    local platform url
    platform=$(detect_platform)
    url=$(download_url "$platform")
    install_binaries "$platform" "$url"
    [[ "${IDOT_NO_PATH_UPDATE:-}" != "1" ]] && update_shell
    say "done! restart your shell or run: export PATH=\"\$HOME/.idot/bin:\$PATH\""
}

main
