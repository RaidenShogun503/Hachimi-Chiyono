#!/bin/bash
set -e

PKG_NAME="com.hachimi.edge"
PKG_VERSION="0.18.1"
PKG_DIR="hachimi-edge-ios"

echo "Packaging $PKG_NAME version $PKG_VERSION..."

# Clean previous build
rm -rf "$PKG_DIR"
rm -f "$PKG_NAME.deb"

# create structure
mkdir -p "$PKG_DIR/Library/MobileSubstrate/DynamicLibraries"

# Copy binary
cp "target/aarch64-apple-ios/release/libhachimi.dylib" "$PKG_DIR/Library/MobileSubstrate/DynamicLibraries/$PKG_NAME.dylib"

# Create plist filter
cat > "$PKG_DIR/Library/MobileSubstrate/DynamicLibraries/$PKG_NAME.plist" <<EOF
{
  Filter = {
    Bundles = ( "com.cygames.umamusume" );
  };
}
EOF

# Create control file
mkdir -p "$PKG_DIR/DEBIAN"
cat > "$PKG_DIR/DEBIAN/control" <<EOF
Package: $PKG_NAME
Name: Hachimi Edge
Version: $PKG_VERSION
Architecture: iphoneos-arm
Description: Game enhancement mod for iOS
Maintainer: Hachimi Team
Author: Hachimi Team
Section: Tweaks
Depends: mobilesubstrate
EOF

# Build deb
dpkg-deb -b "$PKG_DIR" "$PKG_NAME.deb"

echo "Package created: $PKG_NAME.deb"
