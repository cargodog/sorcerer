#!/bin/bash
set -e

echo "🪝 Installing Git hooks for Sorcerer..."

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOKS_DIR="$PROJECT_ROOT/hooks"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

# Check if we're in a git repository
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "❌ Error: Not in a git repository. Please run this script from the project root."
    exit 1
fi

# Check if hooks directory exists
if [ ! -d "$HOOKS_DIR" ]; then
    echo "❌ Error: hooks/ directory not found. Please ensure it exists in the project root."
    exit 1
fi

# List of hooks to install
hooks=("pre-commit" "commit-msg" "pre-push")

echo "📁 Hooks directory: $HOOKS_DIR"
echo "📁 Git hooks directory: $GIT_HOOKS_DIR"
echo ""

# Install each hook
for hook in "${hooks[@]}"; do
    hook_source="$HOOKS_DIR/$hook"
    hook_target="$GIT_HOOKS_DIR/$hook"
    
    if [ ! -f "$hook_source" ]; then
        echo "⚠️  Warning: $hook not found in hooks/ directory, skipping..."
        continue
    fi
    
    echo "🔗 Installing $hook hook..."
    
    # Remove existing hook if it exists
    if [ -f "$hook_target" ] || [ -L "$hook_target" ]; then
        echo "   Removing existing $hook hook..."
        rm -f "$hook_target"
    fi
    
    # Create symlink
    ln -s "$hook_source" "$hook_target"
    
    # Make sure the source hook is executable
    chmod +x "$hook_source"
    
    echo "   ✅ $hook hook installed successfully"
done

echo ""
echo "🎉 Git hooks installation complete!"
echo ""
echo "📋 Installed hooks:"
echo "   • pre-commit: Runs format, lint, build, and security checks"
echo "   • commit-msg: Validates commit message format and quality"
echo "   • pre-push: Runs tests and comprehensive security checks"
echo ""
echo "🔍 To verify installation, run:"
echo "   ls -la .git/hooks/"
echo ""
echo "💡 To bypass hooks (use sparingly):"
echo "   git commit --no-verify"
echo "   git push --no-verify"
echo ""
echo "🛠️  To update hooks, just run this script again!"