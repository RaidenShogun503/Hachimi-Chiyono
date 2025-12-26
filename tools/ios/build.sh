#!/bin/bash
set -e

# Ensure target is installed
# rustup target add aarch64-apple-ios

echo "Building for iOS (aarch64)..."
cargo build --release --target aarch64-apple-ios

OUTPUT_DIR="target/aarch64-apple-ios/release"
LIB_NAME="libhachimi.dylib"

if [ -f "$OUTPUT_DIR/$LIB_NAME" ]; then
    echo "Build successful: $OUTPUT_DIR/$LIB_NAME"
    
    # Sign safely if ldid is present
    if command -v ldid &> /dev/null; then
        echo "Signing with ldid..."
        ldid -S "$OUTPUT_DIR/$LIB_NAME"
    else
        echo "Warning: ldid not found. Binary is unsigned."
    fi
else
    echo "Build failed!"
    exit 1
fi
