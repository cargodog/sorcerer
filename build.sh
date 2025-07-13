#!/bin/bash
set -e

echo "🔨 Building Sorcerer..."
cargo build --release

echo "🏺 Building Agent container image..."
# Try podman first, then docker
if command -v podman > /dev/null 2>&1; then
    echo "Using Podman..."
    podman build --network=host -f agent/Containerfile -t sorcerer-agent:latest .
elif command -v docker > /dev/null 2>&1; then
    echo "Using Docker..."
    docker build -f agent/Containerfile -t sorcerer-agent:latest .
else
    echo "❌ Neither Podman nor Docker found. Please install one of them:"
    echo "  For Podman: sudo pacman -S podman"
    echo "  For Docker: sudo pacman -S docker"
    exit 1
fi

echo "✨ Build complete!"
echo ""
echo "To get started:"
echo "  ./target/release/srcrr create Mickey"
echo "  ./target/release/srcrr list"
echo ""
echo "If using Podman for the first time, make sure the socket is running:"
echo "  systemctl --user start podman.socket"