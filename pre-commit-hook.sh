#!/usr/bin/env bash

# Pre-commit hook to update README ecosystem tables with current versions
# Install this as .git/hooks/pre-commit in each crate repository

set -e

# Only run if README.md exists and has ecosystem table
if [ ! -f "README.md" ] || ! grep -q "## Auxide Ecosystem" README.md; then
    exit 0
fi

# Check if update_all_versions.sh exists in auxide-dsp (workspace script)
if [ -f "../auxide-dsp/update_all_versions.sh" ]; then
    echo "🔄 Updating README ecosystem table with current versions..."
    cd ../auxide-dsp && ./update_all_versions.sh > /dev/null 2>&1
    echo "✅ README updated"
else
    echo "⚠️  Workspace version script not found, skipping README update"
fi