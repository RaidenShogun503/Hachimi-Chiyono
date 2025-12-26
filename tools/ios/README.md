# iOS Dev Tools

These scripts are intended for development on macOS (required for iOS cross-compilation and signing).

- `build.sh`: Builds the Hachimi dylib for iOS (`aarch64-apple-ios`).
- `package.sh`: Packages the built dylib into a `.deb` file for storage/distribution on jailbroken devices.

## Requirements

- macOS with Xcode installed.
- Rust with `aarch64-apple-ios` target: `rustup target add aarch64-apple-ios`
- `ldid` for signing (install via Homebrew: `brew install ldid`).
- `dpkg-deb` for packaging (install via Homebrew: `brew install dpkg`).
