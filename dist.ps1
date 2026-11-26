# [Documentation](docs/build/distribution.md#build-and-deliver)
$ErrorActionPreference = 'Stop'

& (Join-Path $PSScriptRoot 'dist-small.ps1')

$artifacts = Join-Path $PSScriptRoot '.temp-data/dist/apps-win-x64'
$bin = Join-Path $PSScriptRoot 'bin'

function Publish-App([string]$Project, [string]$Name) {
    $publish = Join-Path $artifacts $Name
    dotnet publish $Project --configuration Release --runtime win-x64 -p:PublishAot=true `
        --artifacts-path (Join-Path $artifacts 'build') --output $publish --nologo
    if ($LASTEXITCODE -ne 0) { throw "$Name AOT publish failed (exit code $LASTEXITCODE)." }
    Copy-Item -LiteralPath (Join-Path $publish "$Name.exe") -Destination $bin -Force
    $licenses = Join-Path $publish 'licenses'
    if (Test-Path -LiteralPath $licenses) {
        Copy-Item -LiteralPath $licenses -Destination $bin -Recurse -Force
    }
}

Push-Location $PSScriptRoot
try {
    Publish-App 'examples/ui/gui/gallery/GuiWidgetGallery.fpasproj' 'GuiWidgetGallery'
    Publish-App 'examples/ui/tui/gallery/TuiWidgetGallery.fpasproj' 'TuiWidgetGallery'
    Publish-App 'apps/fpas-sweeper/src/FpasSweeper/FpasSweeper.fpasproj' 'FpasSweeper'

    & 'apps/fpas-amp/dist.ps1'
    Write-Host 'Deployed both widget galleries, FpasSweeper and FpasAmp in bin/.'
}
finally {
    Pop-Location
}
