# Build release binaries and bundled standard-library sources in bin/.
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    cargo build --release -p fpas-cli
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed (exit code $LASTEXITCODE)."
    }
    cargo run --release -p fpas-build --example precompile_stdlib -- target\release\lib bin\lib
    if ($LASTEXITCODE -ne 0) {
        throw "Standard-library staging failed (exit code $LASTEXITCODE)."
    }
    New-Item -ItemType Directory -Path bin -Force | Out-Null
    Copy-Item target\release\fpas.exe bin\fpas.exe -Force
    Copy-Item target\release\fpas-runner.exe bin\fpas-runner.exe -Force
    Write-Host "Built: bin\fpas.exe, bin\fpas-runner.exe, and bin\lib"
}
finally {
    Pop-Location
}
