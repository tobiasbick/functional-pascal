# [Documentation](doc/desktop.md#windows-portable-build)
$ErrorActionPreference = 'Stop'
$repository = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$artifacts = Join-Path $repository '.temp-data/fpas-amp-portable'
Push-Location $PSScriptRoot
try {
    $payload = Join-Path $artifacts 'publish-win-x64'
    dotnet publish src/FpasAmp/FpasAmp.fpasproj -c Release -r win-x64 -p:PublishAot=true `
        --artifacts-path (Join-Path $artifacts 'build') --output $payload --nologo
    if ($LASTEXITCODE -ne 0) { throw "FPas-Amp AOT publish failed (exit code $LASTEXITCODE)." }

    & 'packaging/windows/Dependencies.ps1' -Payload $payload -Cache (Join-Path $artifacts 'ffmpeg')
    & (Join-Path $repository 'src/Fpas.Deployment.WindowsPortable/Publish.ps1') -PayloadDirectory $payload `
        -EntryPoint 'fpas-amp.exe' -Output (Join-Path $repository 'bin/fpas-amp.exe') `
        -Icon 'packaging/windows/FpasAmp.ico' -Title 'FPas-Amp' -ArtifactsPath (Join-Path $artifacts 'portable')
    Write-Host 'Deployed portable bin/fpas-amp.exe.'
}
finally {
    Pop-Location
}
