#!/usr/bin/env bash

# Script to update README ecosystem table with current version numbers
# Run this before commits to keep versions in sync

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Updating Auxide ecosystem table with current versions...${NC}"

# Function to extract version from Cargo.toml
get_version() {
    local crate_dir=$1
    local cargo_toml="$crate_dir/Cargo.toml"

    if [ ! -f "$cargo_toml" ]; then
        echo -e "${RED}Error: $cargo_toml not found${NC}"
        return 1
    fi

    # Extract version line and get the value
    grep '^version = ' "$cargo_toml" | sed 's/version = "\(.*\)"/\1/'
}

# Get versions for each crate
AUXIDE_VERSION=$(get_version "../auxide")
DSP_VERSION=$(get_version ".")
IO_VERSION=$(get_version "../auxide-io")
MIDI_VERSION=$(get_version "../auxide-midi")

echo "Found versions:"
echo "  auxide: $AUXIDE_VERSION"
echo "  auxide-dsp: $DSP_VERSION"
echo "  auxide-io: $IO_VERSION"
echo "  auxide-midi: $MIDI_VERSION"

# Update the README.md file
README="README.md"

if [ ! -f "$README" ]; then
    echo -e "${RED}Error: $README not found${NC}"
    exit 1
fi

# Create the new table content
TABLE_CONTENT="| Crate | Description | Version |
|-------|-------------|---------|
| [auxide](https://github.com/Michael-A-Kuykendall/auxide) | Real-time-safe audio graph kernel | $AUXIDE_VERSION |
| **[auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp)** | DSP nodes library | $DSP_VERSION |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | $IO_VERSION |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | $MIDI_VERSION |"

# Replace the table using awk (more reliable than sed for multi-line)
awk -v table="$TABLE_CONTENT" '
BEGIN { in_table = 0 }
/## Auxide Ecosystem/ { print; print table; in_table = 1; next }
/## Status/ && in_table { in_table = 0; print ""; print; next }
!in_table { print }
' "$README" > "${README}.tmp" && mv "${README}.tmp" "$README"

echo -e "${GREEN}✅ README ecosystem table updated with current versions${NC}"