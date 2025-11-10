#!/bin/bash

# TidyFiles macOS Installation Helper
# This script removes the quarantine attribute to bypass Gatekeeper

set -e

echo "🧹 TidyFiles macOS Installation Helper"
echo "======================================"
echo ""

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ Error: This script is only for macOS"
    exit 1
fi

# Find TidyFiles.app
APP_PATH=""

# Check common locations
if [ -d "/Applications/TidyFiles.app" ]; then
    APP_PATH="/Applications/TidyFiles.app"
elif [ -d "$HOME/Applications/TidyFiles.app" ]; then
    APP_PATH="$HOME/Applications/TidyFiles.app"
elif [ -d "./TidyFiles.app" ]; then
    APP_PATH="./TidyFiles.app"
fi

# If not found, ask user
if [ -z "$APP_PATH" ]; then
    echo "📁 TidyFiles.app not found in common locations."
    echo "Please drag and drop TidyFiles.app here and press Enter:"
    read -r APP_PATH

    # Remove quotes if user dragged and dropped
    APP_PATH="${APP_PATH//\'/}"
fi

# Verify the path exists
if [ ! -d "$APP_PATH" ]; then
    echo "❌ Error: TidyFiles.app not found at: $APP_PATH"
    exit 1
fi

echo "📍 Found TidyFiles at: $APP_PATH"
echo ""
echo "🔓 Removing quarantine attribute..."

# Remove quarantine attribute
if xattr -cr "$APP_PATH" 2>/dev/null; then
    echo "✅ Success! TidyFiles is now ready to use."
    echo ""
    echo "You can now open TidyFiles from:"
    echo "  • Applications folder"
    echo "  • Spotlight (Cmd+Space, type 'TidyFiles')"
    echo "  • Launchpad"
    echo ""

    # Ask if user wants to open now
    read -p "Would you like to open TidyFiles now? (y/n): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "$APP_PATH"
    fi
else
    echo "⚠️  Could not remove quarantine attribute automatically."
    echo ""
    echo "Please try these alternative methods:"
    echo ""
    echo "Method 1: Right-click to Open"
    echo "  1. Find TidyFiles in Applications folder"
    echo "  2. Right-click (or Control+click) on TidyFiles.app"
    echo "  3. Select 'Open' from the menu"
    echo "  4. Click 'Open' in the dialog"
    echo ""
    echo "Method 2: System Settings"
    echo "  1. Go to System Settings → Privacy & Security"
    echo "  2. Scroll down to find TidyFiles"
    echo "  3. Click 'Open Anyway'"
    echo ""
fi
