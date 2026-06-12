#!/bin/bash
set -e

echo "=== Code-Termination Game Installer ==="
echo "This script will compile and install Code-Termination as a system command."

# 1. Detect OS and check dependencies
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    OS="unknown"
fi

echo "Detected OS: $OS"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Please install Rust using: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 2. Build the game in release mode
echo "Compiling the game in release mode (this might take a minute)..."
cargo build --release

# 3. Choose installation directory
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    echo "Notice: /usr/local/bin is not writable by current user. Installing to ~/.local/bin instead."
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    
    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "Warning: $INSTALL_DIR is not in your PATH."
        echo "You can add it by running: echo 'export PATH=\"\$PATH:\$HOME/.local/bin\"' >> ~/.bashrc"
    fi
fi

# 4. Copy binary
echo "Installing command to $INSTALL_DIR/code-termination..."
cp target/release/Code-Termination "$INSTALL_DIR/code-termination"
chmod +x "$INSTALL_DIR/code-termination"

# 5. Create a desktop launcher (Linux shortcut)
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

echo "=== Installation Complete! ==="
echo "You can now run the game by typing 'code-termination' in any terminal,"
echo "or by launching 'Code-Termination' from your applications menu."
