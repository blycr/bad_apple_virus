$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location -LiteralPath $projectRoot
try {
    $metadataJson = cargo metadata --locked --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' }
    $metadata = $metadataJson | ConvertFrom-Json
    $package = $metadata.packages | Where-Object { $_.name -eq 'bad_apple' }
    $version = $package.version
    $binary = Join-Path $metadata.target_directory 'release\bad_apple.exe'
    if (-not (Test-Path -LiteralPath $binary)) { throw 'Run cargo build --locked --release first' }
    $dist = Join-Path $metadata.target_directory 'dist'
    $staging = Join-Path $dist "bad-apple-windows-v$version"
    New-Item -ItemType Directory -Path $staging -Force | Out-Null
    Copy-Item -LiteralPath $binary -Destination (Join-Path $staging 'bad_apple.exe') -Force
    foreach ($file in @('Play.cmd', 'Play-Windows.cmd', 'README.md', 'LICENSE')) {
        Copy-Item -LiteralPath (Join-Path $projectRoot $file) -Destination $staging -Force
    }
    $binaryHash = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
    "$binaryHash  bad_apple.exe" | Set-Content -LiteralPath (Join-Path $staging 'SHA256SUMS') -Encoding ascii
    $zipName = "bad-apple-windows-v$version.zip"
    $zipPath = Join-Path $dist $zipName
    Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zipPath -Force
    $zipHash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
    @("$binaryHash  bad_apple.exe", "$zipHash  $zipName") |
        Set-Content -LiteralPath (Join-Path $dist 'SHA256SUMS') -Encoding ascii
    Write-Output "Packaged $zipPath"
    Get-Content -LiteralPath (Join-Path $dist 'SHA256SUMS')
} finally {
    Pop-Location
}
