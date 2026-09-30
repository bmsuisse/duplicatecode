#!/bin/sh
# Installs the latest duplicatecode release binary into ~/.local/bin (or $INSTALL_DIR).
# Usage: curl -fsSL https://raw.githubusercontent.com/bmsuisse/duplicatecode/main/install.sh | sh
# Prefer `uv tool install duplicatecode` if you have uv.
set -eu

REPO="bmsuisse/duplicatecode"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-gnu ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  *) echo "unsupported platform: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

url="https://github.com/$REPO/releases/latest/download/duplicatecode-$target.tar.gz"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $url"
curl -fsSL "$url" | tar -xz -C "$tmp"
mkdir -p "$INSTALL_DIR"
install -m 755 "$tmp/duplicatecode" "$INSTALL_DIR/duplicatecode"
echo "Installed to $INSTALL_DIR/duplicatecode"
case ":$PATH:" in *":$INSTALL_DIR:"*) ;; *) echo "Note: add $INSTALL_DIR to your PATH" ;; esac
