param (
    [string]$SdkPath = $env:SDKROOT
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($SdkPath)) {
    # Default path based on user's environment in previous manual steps
    $SdkPath = "C:\iPhoneOS26.1.sdk"
    Write-Warning "SDKROOT not set, defaulting to $SdkPath"
} else {
    Write-Host "Using SDKROOT: $SdkPath"
}

if (-not (Test-Path $SdkPath)) {
    Write-Error "iOS SDK not found at $SdkPath. Please set SDKROOT or download the SDK."
    exit 1
}

# 1. Build Dobby Dependencies (SKIPPED - MIGRATED TO ELLEKIT)
# Write-Host "Building Dobby dependencies..."
# $dobbyScript = Join-Path $PSScriptRoot "build_dobby.ps1"
# & $dobbyScript -SdkPath $SdkPath
# if ($LASTEXITCODE -ne 0) { exit 1 }

# 2. Setup Environment for Cross-Compilation
Write-Host "Configuring cross-compilation environment..."
$env:SDKROOT = $SdkPath
$env:CC_aarch64_apple_ios = "clang"
$env:CXX_aarch64_apple_ios = "clang++"
$env:AR_aarch64_apple_ios = "llvm-ar"
$env:RUSTFLAGS = "-C link-arg=-Wl,-install_name,@rpath/libhachimi.dylib -C link-arg=-fuse-ld=lld"
# Important: double quotes for paths with spaces, though SDK path here likely doesn't have them
$env:CFLAGS_aarch64_apple_ios = "-isysroot $SdkPath -target aarch64-apple-ios"
$env:CXXFLAGS_aarch64_apple_ios = "-isysroot $SdkPath -target aarch64-apple-ios"

# Fix "linker cc not found"
$env:CARGO_TARGET_AARCH64_APPLE_IOS_LINKER = "clang"

# Check if clang is available
if (-not (Get-Command "clang" -ErrorAction SilentlyContinue)) {
    Write-Error "Clang not found. Please install LLVM for Windows and add to PATH."
    exit 1
}

# 3. Cargo Build
Write-Host "Starting Cargo Build (Release)..."
# We act as if we are in the root of the repo if this script is in tools/ios
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Push-Location $RepoRoot

try {
    cargo build --release --target aarch64-apple-ios
} finally {
    Pop-Location
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build Success!"
    $TargetPath = Join-Path $RepoRoot "target\aarch64-apple-ios\release\libhachimi.dylib"
    if (Test-Path $TargetPath) {
        Write-Host "Artifact: $TargetPath"
        
        # Optional: Attempt signing if ldid exists
        if (Get-Command "ldid" -ErrorAction SilentlyContinue) {
            Write-Host "Signing with ldid..."
            ldid -S $TargetPath
        } else {
            Write-Warning "ldid not found. The binary is unsigned."
        }
    }
} else {
    Write-Error "Build Failed."
    exit 1
}
