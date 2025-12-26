# Building for iOS on Windows

Building for iOS natively on Windows is effectively impossible because it requires the **Apple iOS SDK** and macOS-specific tools like `codesign` and `ldid` which are not available on Windows.

## The Solution: GitHub Actions

We have provided a **GitHub Actions** workflow that allows you to build the iOS package automatically in the cloud using a macOS runner.

### How to use:

1.  **Push your code** to a GitHub repository.
2.  Go to the **Actions** tab in your repository.
3.  Select the **Build iOS** workflow on the left.
4.  Click **Run workflow** (or just push to `main`/`master` to trigger it).
5.  Once finished, click on the workflow run.
6.  Scroll down to **Artifacts** and download `hachimi-ios-deb`.
7.  Extract the zip to find your `.deb` file.

7.  Extract the zip to find your `.deb` file.

## Method 2: Local Build (Advanced)

If you have a **jailbroken iOS device** and are comfortable with advanced usage, you can cross-compile on Windows using `rust-lld` and Clang.

### Prerequisites
1.  **Clang/LLVM**: Install LLVM for Windows.
    - **Method A (Winget)**: Run `winget install LLVM.LLVM` in PowerShell, then **restart your terminal**.
    - **Method B (Manual)**: Download from [releases.llvm.org](https://releases.llvm.org/) and add to PATH.
2.  **iOS SDK**: You must obtain a copy of the iOS SDK (e.g. iPhoneOS.sdk). You can find these on GitHub (e.g., search for `xybp888/iOS-SDKs`).
3.  **Rust Target**: `rustup target add aarch64-apple-ios`
4.  **SDKROOT**: Set the `SDKROOT` environment variable to the path of your SDK.

### How to Build
1.  Open PowerShell.
2.  Set SDKROOT: `$env:SDKROOT = "C:\path\to\iPhoneOS.sdk"`
3.  Run the build script:
    ```powershell
    powershell -ExecutionPolicy Bypass -File .\tools\ios\build_windows.ps1
    ```
    *(Note: The `-ExecutionPolicy Bypass` flag is needed if your system blocks running scripts)*
4.  The output file will be in `target/aarch64-apple-ios/release/libhachimi.dylib`.

### Signing
**Important**: The locally built file is **unsigned**. You must sign it on your device using `ldid` or before installing.
- Using `ldid` on device: `ldid -S libhachimi.dylib`
- Using `rcodesign` (included in build script): The script attempts to adhere-sign automatically if `rcodesign` is installed.

## Debugging / Viewing Logs

To view Hachimi's real-time logs (`[Hachimi] ...`) on Windows:

### Option 1: libimobiledevice (Command Line)
1.  Install **libimobiledevice** for Windows via Scoop:
    ```powershell
    scoop bucket add extras
    scoop install libimobiledevice
    ```
2.  Connect your device via USB.
3.  Run:
    ```powershell
    idevicesyslog | findstr "Hachimi"
    ```

### Option 2: 3uTools (GUI)
1.  Open **3uTools**.
2.  Go to **Toolbox** > **Realtime Log**.
3.  Click **Filter** and enter `Hachimi`.
4.  Launch the game to see logs appear.

