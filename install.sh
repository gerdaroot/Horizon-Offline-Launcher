#!/bin/sh
# Installs Horizon Offline Launcher on Linux (x86-64) or macOS (Apple Silicon), for this user only.
#   curl -fsSL https://github.com/gerdaroot/horizon-offline-launcher/releases/latest/download/install.sh | sh
# Linux: ~/.local/share/horizon-offline-launcher, `horizon-offline-launcher` in ~/.local/bin, and
# the launcher adds itself to the applications menu when it first starts.
# macOS: ~/Applications/Horizon Offline Launcher/, `horizon-offline-launcher` in ~/.local/bin.
set -eu

REPO="gerdaroot/horizon-offline-launcher"
BASE="https://github.com/$REPO/releases/latest/download"

case "$(uname -s)" in
  Linux)
    ASSET="horizon-offline-launcher-linux-x86_64.tar.gz"
    DIR="${XDG_DATA_HOME:-$HOME/.local/share}/horizon-offline-launcher"
    ;;
  Darwin)
    ASSET="horizon-offline-launcher-macos-arm64.zip"
    DIR="$HOME/Applications/Horizon Offline Launcher"
    ;;
  *)
    echo "Unsupported system: $(uname -s). On Windows run install.ps1." >&2
    exit 1
    ;;
esac

echo "Horizon Offline Launcher"
echo "  Downloading $ASSET"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
curl -fL --progress-bar "$BASE/$ASSET" -o "$TMP/$ASSET"

mkdir -p "$DIR"
case "$ASSET" in
  *.tar.gz) tar -xzf "$TMP/$ASSET" -C "$TMP" ;;
  *.zip) (cd "$TMP" && unzip -q "$ASSET") ;;
esac
mv -f "$TMP/horizon-launcher" "$DIR/horizon-offline-launcher"
chmod +x "$DIR/horizon-offline-launcher"
# macOS asks before opening downloaded programs that aren't notarized.
if [ "$(uname -s)" = Darwin ]; then
  xattr -dr com.apple.quarantine "$DIR" 2>/dev/null || true
fi

mkdir -p "$HOME/.local/bin"
ln -sf "$DIR/horizon-offline-launcher" "$HOME/.local/bin/horizon-offline-launcher"
echo "  Installed in $DIR"
echo "  Start it with: horizon-offline-launcher"
case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) echo "  (~/.local/bin is not on your PATH: add it, or run $DIR/horizon-offline-launcher)" ;;
esac
echo "  Next: open the launcher, press Prepare (it uses your Minecraft 26.3), then Play."
