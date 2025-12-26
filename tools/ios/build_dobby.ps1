param (
    [string]$SdkPath = $env:SDKROOT
)

if ([string]::IsNullOrWhiteSpace($SdkPath)) {
    $SdkPath = "C:\\iPhoneOS26.1.sdk"
    
}

$buildDir = "$PSScriptRoot/dobby_build"
$sourceDir = "$PSScriptRoot/dobby_src"

# 1. Clone Dobby if not exists
if (-not (Test-Path $sourceDir)) {
    Write-Host "Cloning Dobby..."
    git clone --depth 1 https://github.com/jmpews/Dobby.git $sourceDir
}

# 2. Configure CMake
Remove-Item $buildDir -Recurse -Force

New-Item -ItemType Directory -Path $buildDir | Out-Null

Push-Location $buildDir

try {
    # Add current dir (with ninja.exe) to PATH
    $env:PATH = "$PSScriptRoot;$env:PATH"

    if (Get-Command "ninja" -ErrorAction SilentlyContinue) {
       Write-Host "Found ninja in PATH"
       ninja --version
    } else {
       Write-Error "Ninja not found in PATH ($PSScriptRoot)"
       exit 1
    }

    $SdkPath = $SdkPath -replace "\\", "/"

    Write-Host "Configuring Dobby CMake with SDK: $SdkPath"

    $LLVM_AR = (Get-Command llvm-ar).Source
    $LLVM_RANLIB = (Get-Command llvm-ranlib).Source
    $LLVM_NM = (Get-Command llvm-nm).Source

    if (Get-Command llvm-install-name-tool -ErrorAction SilentlyContinue) {
        $LLVM_INSTALL_NAME_TOOL = (Get-Command llvm-install-name-tool).Source
    } else {
        Write-Host "llvm-install-name-tool not found, using dummy."
        $LLVM_INSTALL_NAME_TOOL = "true"
    }

    Write-Host "Using LLVM tools from: $LLVM_AR"

    cmake -G "Ninja" $sourceDir `
        "-DCMAKE_SYSTEM_NAME=iOS" `
        "-DCMAKE_OSX_SYSROOT=$SdkPath" `
        "-DCMAKE_OSX_ARCHITECTURES=arm64" `
        "-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0" `
        "-DCMAKE_C_COMPILER=clang" `
        "-DCMAKE_CXX_COMPILER=clang++" `
        "-DCMAKE_AR=$LLVM_AR" `
        "-DCMAKE_RANLIB=$LLVM_RANLIB" `
        "-DCMAKE_Nm=$LLVM_NM" `
        "-DCMAKE_INSTALL_NAME_TOOL=$LLVM_INSTALL_NAME_TOOL" `
        "-DCMAKE_C_COMPILER_TARGET=aarch64-apple-ios" `
        "-DCMAKE_CXX_COMPILER_TARGET=aarch64-apple-ios" `
        "-DCMAKE_ASM_COMPILER_TARGET=aarch64-apple-ios" `
        "-DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld" `
        "-DCMAKE_SHARED_LINKER_FLAGS=-fuse-ld=lld" `
        "-DCMAKE_SYSTEM_PROCESSOR=arm64" `
        "-DDOBBY_GENERATE_SHARED=OFF" `
        "-DDOBBY_BUILD_TEST=OFF"

    if ($LASTEXITCODE -ne 0) {
        Write-Error "CMake configure failed."
        exit 1
    }

    Write-Host "Building Dobby..."
    cmake --build . --config Release --parallel 1

    if ($LASTEXITCODE -eq 0) {
        Write-Host "Dobby built successfully at $buildDir"
        
        # Create dummy libstdc++.a
        $dummyC = "$buildDir/dummy.c"
        Set-Content -Path $dummyC -Value "void __dummy_stdc_shim() {}"
        
        $dummyObj = "$buildDir/dummy.o"
        & clang -c $dummyC -o $dummyObj "-target" "aarch64-apple-ios"
        
        $dummyLib = "$buildDir/libstdc++.a"
        & $LLVM_AR rc $dummyLib $dummyObj
        Write-Host "Created dummy libstdc++.a to satisfy linker."
    } else {
        Write-Error "Dobby build failed."
        exit 1
    }
} finally {
    Pop-Location
}
