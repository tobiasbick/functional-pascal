# [Documentation](doc/README.md#distribution)
$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$artifacts = Join-Path $repository '.temp-data/fpas-sweeper-windows'
$publish = Join-Path $artifacts 'publish'
$bin = Join-Path $repository 'bin'
Push-Location $PSScriptRoot
try {
    dotnet publish src/FpasSweeper/FpasSweeper.fpasproj -c Release -r win-x64 -p:PublishAot=true `
        --artifacts-path (Join-Path $artifacts 'build') --output $publish --nologo
    if ($LASTEXITCODE -ne 0) { throw "FPas-Sweeper AOT publish failed (exit code $LASTEXITCODE)." }

    New-Item -ItemType Directory -Force -Path $bin | Out-Null
    Copy-Item -LiteralPath (Join-Path $publish 'licenses') -Destination $bin -Recurse -Force
    Copy-Item -LiteralPath (Join-Path $publish 'fpas-sweeper.exe') -Destination $bin -Force
    Write-Host 'Deployed Native AOT bin/fpas-sweeper.exe.'
}
finally {
    Pop-Location
}
