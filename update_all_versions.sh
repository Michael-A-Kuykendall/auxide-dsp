#!/usr/bin/env bash

# Workspace script to update all README ecosystem tables with current versions
# Run this from the workspace root before commits to keep all versions in sync

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔄 Updating all Auxide README ecosystem tables...${NC}"

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

echo -e "${YELLOW}Current versions:${NC}"
echo "  auxide: $AUXIDE_VERSION"
echo "  auxide-dsp: $DSP_VERSION"
echo "  auxide-io: $IO_VERSION"
echo "  auxide-midi: $MIDI_VERSION"

# Function to update a single README
update_readme() {
    local crate_name=$1
    local readme_path="$crate_name/README.md"

    if [ ! -f "$readme_path" ]; then
        echo -e "${RED}Warning: $readme_path not found, skipping${NC}"
        return
    fi

    echo -e "${YELLOW}Updating $crate_name README...${NC}"

    # Create the new table content with current crate bolded
    if [ "$crate_name" = "../auxide" ]; then
        TABLE_CONTENT="| Crate | Description | Version |
|-------|-------------|---------|
| **[auxide](https://github.com/Michael-A-Kuykendall/auxide)** | Real-time-safe audio graph kernel | $AUXIDE_VERSION |
| [auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp) | DSP nodes library | $DSP_VERSION |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | $IO_VERSION |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | $MIDI_VERSION |"
    elif [ "$crate_name" = "." ]; then
        TABLE_CONTENT="| Crate | Description | Version |
|-------|-------------|---------|
| [auxide](https://github.com/Michael-A-Kuykendall/auxide) | Real-time-safe audio graph kernel | $AUXIDE_VERSION |
| **[auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp)** | DSP nodes library | $DSP_VERSION |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | $IO_VERSION |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | $MIDI_VERSION |"
    elif [ "$crate_name" = "../auxide-io" ]; then
        TABLE_CONTENT="| Crate | Description | Version |
|-------|-------------|---------|
| [auxide](https://github.com/Michael-A-Kuykendall/auxide) | Real-time-safe audio graph kernel | $AUXIDE_VERSION |
| [auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp) | DSP nodes library | $DSP_VERSION |
| **[auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io)** | Audio I/O layer | $IO_VERSION |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | $MIDI_VERSION |"
    elif [ "$crate_name" = "../auxide-midi" ]; then
        TABLE_CONTENT="| Crate | Description | Version |
|-------|-------------|---------|
| [auxide](https://github.com/Michael-A-Kuykendall/auxide) | Real-time-safe audio graph kernel | $AUXIDE_VERSION |
| [auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp) | DSP nodes library | $DSP_VERSION |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | $IO_VERSION |
| **[auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi)** | MIDI integration | $MIDI_VERSION |"
    fi

    # Replace the table using awk (more reliable than sed for multi-line)
    awk -v table="$TABLE_CONTENT" '
    BEGIN { in_table = 0 }
    /## Auxide Ecosystem/ { print; print table; in_table = 1; next }
    /## Status/ && in_table { in_table = 0; print ""; print; next }
    !in_table { print }
    ' "$readme_path" > "${readme_path}.tmp" && mv "${readme_path}.tmp" "$readme_path"

    echo -e "${GREEN}✅ $crate_name README updated${NC}"
}

# Update all READMEs
update_readme "../auxide"
update_readme "."
update_readme "../auxide-io"
update_readme "../auxide-midi"

echo -e "${GREEN}🎉 All README ecosystem tables updated with current versions!${NC}"
echo -e "${BLUE}💡 Remember to run this script before commits to keep versions in sync${NC}"