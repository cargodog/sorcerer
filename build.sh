#!/bin/bash
set -e

echo "🔨 Building Sorcerer..."
cargo build --release

echo "🏺 Building Apprentice container image..."
# Try podman first, then docker
if command -v podman > /dev/null 2>&1; then
    echo "Using Podman..."
    podman build --network=host -f apprentice/Containerfile -t sorcerer-apprentice:latest .
elif command -v docker > /dev/null 2>&1; then
    echo "Using Docker..."
    docker build -f apprentice/Containerfile -t sorcerer-apprentice:latest .
else
    echo "❌ Neither Podman nor Docker found. Please install one of them:"
    echo "  For Podman: sudo pacman -S podman"
    echo "  For Docker: sudo pacman -S docker"
    exit 1
fi

echo "✨ Build complete!"
echo ""
echo "To get started:"
echo "  export ANTHROPIC_API_KEY='your-key-here'"
echo "  ./target/release/srcrr summon Mickey"
echo "  ./target/release/srcrr tell Mickey 'Hello, world!'"
echo ""
echo "If using Podman for the first time, make sure the socket is running:"
echo "  systemctl --user start podman.socket"