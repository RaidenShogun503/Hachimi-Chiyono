$sdk = $env:SDKROOT
if ([string]::IsNullOrWhiteSpace($sdk)) {
    $sdk = "C:\iPhoneOS26.1.sdk"
}

Write-Host "Fixing critical symlinks in $sdk"

# Helpers
function Fix-Symlink {
    param($LinkName, $TargetName)
    $linkPath = Join-Path "$sdk\usr\lib" $LinkName
    $targetPath = Join-Path "$sdk\usr\lib" $TargetName
    
    if (Test-Path $targetPath) {
        Write-Host "Copying $TargetName to $LinkName"
        Copy-Item -Path $targetPath -Destination $linkPath -Force
    } else {
        Write-Error "Target $targetPath does not exist!"
    }
}

# libSystem chain
Fix-Symlink "libSystem.tbd" "libSystem.B.tbd"
Fix-Symlink "libc.tbd" "libSystem.B.tbd" 
Fix-Symlink "libm.tbd" "libSystem.B.tbd"

# libobjc
Fix-Symlink "libobjc.tbd" "libobjc.A.tbd"

# libiconv (assuming libiconv.2.tbd or similar exists? Need to check. Usually libiconv.2.4.0.dylib or similar)
# Let's check listing first or just blindly try common ones if needed.
# For now, fix the known ones.

Write-Host "Done."
