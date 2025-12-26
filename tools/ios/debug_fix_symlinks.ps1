param (
    [string]$SdkPath = $env:SDKROOT
)

if ([string]::IsNullOrWhiteSpace($SdkPath)) {
    $SdkPath = "C:\iPhoneOS26.1.sdk"
}

Write-Host "Checking symlinks in SDK at: $SdkPath"

$files = Get-ChildItem -Path $SdkPath -Recurse -File

foreach ($file in $files) {
    if ($file.Length -lt 1024) {
        $content = Get-Content -Path $file.FullName -Raw
        if ($null -eq $content) { continue }
        
        $trimmed = $content.Trim()
        
        # Check specific files we know are problematic to debug logic
        if ($file.Name -eq "libSystem.tbd") {
            Write-Host "DEBUG: Found libSystem.tbd. Content: '$($content)' Trimmed: '$($trimmed)'"
        }

        # Heuristic: no newlines, not empty, target exists
        if (-not $trimmed.Contains("`n") -and -not [string]::IsNullOrWhiteSpace($trimmed)) {
            $targetPath = Join-Path -Path $file.DirectoryName -ChildPath $trimmed
            
            if (Test-Path $targetPath -PathType Leaf) {
                Write-Host "Fixing: $($file.Name) -> $($trimmed)"
                try {
                    Copy-Item -Path $targetPath -Destination $file.FullName -Force -ErrorAction Stop
                } catch {
                    Write-Error "Failed to copy $targetPath to $($file.FullName): $_"
                }
            } else {
                 if ($file.Name -eq "libSystem.tbd") {
                    Write-Host "DEBUG: Target '$targetPath' does not exist."
                 }
            }
        } else {
             if ($file.Name -eq "libSystem.tbd") {
                Write-Host "DEBUG: Content failed heuristic."
             }
        }
    }
}
