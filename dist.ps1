# Build and deploy the Release compiler and its dependencies to bin/.
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --configuration Release --self-contained false `
        -p:UseAppHost=true --artifacts-path .temp-data/dist/windows --output bin -warnaserror
    if ($LASTEXITCODE -ne 0) {
        throw "Release publish failed (exit code $LASTEXITCODE)."
    }
    Move-Item -LiteralPath bin/Fpas.Cli.exe -Destination bin/fpas.exe -Force
    Write-Host 'Deployed Release compiler to bin/fpas.exe (requires .NET 10).'
}
finally {
    Pop-Location
}
