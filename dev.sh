#!/bin/bash
# Development runner: Build DLL and run Rust example
# Usage: ./dev.sh [--release] [--no-example] [--no-rebuild]

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Parse arguments
RELEASE=false
NO_EXAMPLE=false
NO_REBUILD=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE=true
            shift
            ;;
        --no-example)
            NO_EXAMPLE=true
            shift
            ;;
        --no-rebuild)
            NO_REBUILD=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Determine preset and example mode
PRESET=$([ "$RELEASE" = true ] && echo "msvc-release" || echo "msvc-debug")
BUILD_EXAMPLE=$([ "$NO_EXAMPLE" = true ] && echo "false" || echo "true")

write_title() {
    echo -e "\n${YELLOW}========================================${NC}"
    echo -e "${YELLOW}$1${NC}"
    echo -e "${YELLOW}========================================\n${NC}"
}

write_error_exit() {
    echo -e "${RED}$1${NC}"
    exit 1
}

write_title "spoutdx-ffi Development Build"
echo -e "${GREEN}Preset: $PRESET${NC}"
echo -e "${GREEN}Build Example: $BUILD_EXAMPLE${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Step 1: Build C++ DLL
if [ "$NO_REBUILD" != "true" ]; then
    write_title "Step 1: Building C++ DLL"
    
    cd "$SCRIPT_DIR"
    
    # Configure
    echo -e "${GREEN}Configuring CMake...${NC}"
    cmake --preset "$PRESET" || write_error_exit "CMake configuration failed"
    
    # Build
    echo -e "${GREEN}Building DLL...${NC}"
    cmake --build --preset "$PRESET" || write_error_exit "CMake build failed"
    
    echo -e "${GREEN}✓ C++ DLL built successfully${NC}"
else
    echo -e "${YELLOW}⊘ Skipping DLL rebuild${NC}"
fi

# Step 2: Run Rust Example
if [ "$BUILD_EXAMPLE" = "true" ]; then
    write_title "Step 2: Running Rust Example"
    
    cd "$SCRIPT_DIR/examples"
    
    echo -e "${GREEN}Running example...${NC}"
    cargo run $([ "$RELEASE" = "true" ] && echo "--release" || echo "") || write_error_exit "Example execution failed"
    
    echo -e "\n${GREEN}✓ Example ran successfully${NC}"
else
    echo -e "${YELLOW}⊘ Skipping example execution${NC}"
fi

write_title "Build Complete!"
DLL_DIR=$([ "$RELEASE" = true ] && echo "Release" || echo "Debug")
BIN_DIR=$([ "$RELEASE" = true ] && echo "release" || echo "debug")
echo -e "${GREEN}DLL Location: ./out/build/$PRESET/$DLL_DIR/spoutdx_ffi.dll${NC}"
echo -e "${GREEN}Example Binary: ./examples/target/$BIN_DIR/ping${NC}"
