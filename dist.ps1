# Build release binaries and bundled standard-library sources in bin/.
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    cargo build --locked --release -p fpas-cli -p fpas-lsp -p fpas-runner
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed (exit code $LASTEXITCODE)."
    }
    & .\target\release\fpas.exe stdlib stage --source target\release\lib --output bin\lib
    if ($LASTEXITCODE -ne 0) {
        throw "Standard-library staging failed (exit code $LASTEXITCODE)."
    }
    New-Item -ItemType Directory -Path bin -Force | Out-Null
    Copy-Item target\release\fpas.exe bin\fpas.exe -Force
    Copy-Item target\release\fpas-runner.exe bin\fpas-runner.exe -Force
    Copy-Item target\release\fpas-lsp.exe bin\fpas-lsp.exe -Force
    Write-Host "Built: bin\fpas.exe, bin\fpas-runner.exe, bin\fpas-lsp.exe, and bin\lib"
}
finally {
    Pop-Location
}
