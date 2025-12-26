param (
    [string]$SdkPath = $env:SDKROOT
)

if ([string]::IsNullOrWhiteSpace($SdkPath)) {
    Write-Error "SDK path not provided and SDKROOT env var is not set."
    exit 1
}

Write-Host "Fixing symlinks in SDK at: $SdkPath"

$files = Get-ChildItem -Path $SdkPath -Recurse -File

foreach ($file in $files) {
    # Git symlinks are usually small text files
    if ($file.Length -lt 1024) {
        try {
            $content = Get-Content -Path $file.FullName -Raw -ErrorAction Stop
            $content = $content.Trim()
            
            # Check if content looks like a relative path
            if (-not $content.Contains("`n") -and -not [string]::IsNullOrWhiteSpace($content)) {
                $targetPath = Join-Path -Path $file.DirectoryName -ChildPath $content
                
                if (Test-Path $targetPath -PathType Leaf) {
                    Write-Host "Fixing symlink: $($file.Name) -> $content"
                    
                    # Verify target is not a symlink itself (simple check: valid TBD start?)
                    # Or just copy it. If it's a chain, we might need multiple passes or recursive resolution.
                    # For now, simplistic copy.
                    
                    Copy-Item -Path $targetPath -Destination $file.FullName -Force
                }
            }
        }
        catch {
            # Ignore read errors
        }
    }
}

Write-Host "Done."
