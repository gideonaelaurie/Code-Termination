#!/bin/bash
# A simple script for users to download and install the pre-compiled game binary
set -e

# USER_CONFIG: Set this to your GitHub repository to enable direct release downloads
REPO="gideonaelaurie/Code-Termination"
LATEST_RELEASE_URL="https://github.com/$REPO/releases/latest/download/code-termination"

echo "=== Code-Termination Game Downloader ==="
if [ "$REPO" = "your-github-username/Code-Termination" ]; then
    echo "Warning: Distributor has not configured the REPO path in download.sh yet."
    echo "To configure, edit download.sh and replace REPO with your GitHub repository name."
    echo "Example: REPO=\"john-doe/Code-Termination\""
    exit 1
fi

echo "Downloading latest pre-compiled binary from $LATEST_RELEASE_URL..."
curl -L -o code-termination "$LATEST_RELEASE_URL"
chmod +x code-termination

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    echo "Notice: /usr/local/bin is not writable. Installing to ~/.local/bin instead."
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

mv code-termination "$INSTALL_DIR/code-termination"

# Create a desktop launcher (Linux shortcut)
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
cat <<EOF > "$DESKTOP_DIR/code-termination.desktop"
[Desktop Entry]
Type=Application
Name=Code-Termination
Comment=Cyberpunk Hacker Platformer Game
Exec=code-termination
Icon=utilities-terminal
Terminal=false
Categories=Game;
EOF

echo "=== Download & Install Complete! ==="
echo "Code-Termination successfully installed to $INSTALL_DIR/code-termination!"
echo "You can now run it by typing 'code-termination' in any terminal."
