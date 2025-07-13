#!/bin/bash

# Demo script for Sorcerer Agent Mode
# This script demonstrates the new agent capabilities

set -e

echo "🧙‍♂️ Sorcerer Agent Mode Demo"
echo "================================"

# Check if API key is set
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ ANTHROPIC_API_KEY not set. Please set it first:"
    echo "export ANTHROPIC_API_KEY='your-key-here'"
    exit 1
fi

# Build the updated image
echo "🔨 Building apprentice image with agent capabilities..."
podman build -f apprentice/Containerfile -t sorcerer-apprentice:latest . || docker build -f apprentice/Containerfile -t sorcerer-apprentice:latest .

# Build the CLI
echo "🔨 Building sorcerer CLI..."
cargo build --release

# Clean up any existing apprentices
echo "🧹 Cleaning up any existing apprentices..."
./target/release/srcrr rm -a 2>/dev/null || true

# Summon a simple chat apprentice (no system prompt)
echo "🌟 Summoning simple chat apprentice 'Merlin'..."
./target/release/srcrr summon Merlin --no-system-prompt

# Summon an autonomous apprentice (default)
echo "🤖 Summoning autonomous apprentice 'AutoWizard'..."
./target/release/srcrr summon AutoWizard

# Check status
echo "📊 Checking apprentice status..."
./target/release/srcrr ls

echo "📋 Test 1: Simple chat apprentice conversation"
echo "=============================================="
echo "Asking Merlin a simple question (no system prompt - chat only)..."
./target/release/srcrr tell Merlin "What is 2+2?"

echo ""
echo "🤖 Test 2: Autonomous apprentice with commands"
echo "=============================================="
echo "Asking AutoWizard to perform autonomous tasks..."
./target/release/srcrr tell AutoWizard "Please search for all Rust files in the project and tell me how many you found. Then create a simple report file called project-analysis.md with your findings."

echo ""
echo "📄 Checking if the agent created the report file..."
if [ -f "project-analysis.md" ]; then
    echo "✅ Report file created successfully!"
    echo "📖 Contents:"
    cat project-analysis.md
else
    echo "⚠️ Report file not found - agent may still be working or there was an issue"
fi

echo ""
echo "🔍 Full conversation history with AutoWizard:"
echo "============================================="
./target/release/srcrr show AutoWizard

echo ""
echo "🧹 Demo complete! Cleaning up..."
./target/release/srcrr rm -a

echo "✨ Demo finished successfully!"
echo ""
echo "💡 Key differences:"
echo "  - Simple chat apprentices (--no-system-prompt) just chat using Claude"
echo "  - Autonomous apprentices (default) can execute commands autonomously"
echo "  - Autonomous responses are parsed as JSON command structures"
echo "  - Autonomous apprentices can read/write files, execute commands, search code, etc."