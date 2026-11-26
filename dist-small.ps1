# [Documentation](docs/build/distribution.md#build-and-deliver)
# Build and deploy matching Release CLI and project SDK artifacts.
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    $sdkProject = 'sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj'
    $sdkVersion = (dotnet msbuild $sdkProject -nologo -getProperty:PackageVersion).Trim()
    if ($LASTEXITCODE -ne 0 -or $sdkVersion -notmatch '^[0-9][0-9A-Za-z.+-]*$') {
        throw 'Unable to determine the project SDK package version.'
    }
    New-Item -ItemType Directory -Force -Path sdk/feed | Out-Null
    # PSReadLine can shadow the framework type in Windows PowerShell.
    if ([System.Runtime.InteropServices.RuntimeInformation, mscorlib]::OSArchitecture -ne 'X64') { throw 'FPAS distributions support Windows x64 and Linux x64.' }
    $artifacts = Join-Path $PSScriptRoot '.temp-data/dist/windows'
    $package = Join-Path $PSScriptRoot "sdk/feed/Fpas.NET.Sdk.$sdkVersion.nupkg"
    dotnet build $sdkProject --configuration Release --artifacts-path $artifacts -p:UseAppHost=true -p:DebugType=portable "-p:FpasSdkVersion=$sdkVersion" -p:FpasCuiDistributionRid=win-x64 -warnaserror
    if ($LASTEXITCODE -ne 0) { throw "Release SDK build failed (exit code $LASTEXITCODE)." }
    dotnet pack $sdkProject --no-build --no-restore --configuration Release `
        --artifacts-path $artifacts -p:UseAppHost=true -p:DebugType=portable "-p:FpasSdkVersion=$sdkVersion" -p:FpasCuiDistributionRid=win-x64 --output sdk/feed
    if ($LASTEXITCODE -ne 0) { throw "Release SDK pack failed (exit code $LASTEXITCODE)." }
    dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --configuration Release --runtime win-x64 `
        -p:FpasDistribution=true -p:DebugType=portable "-p:FpasSdkPackage=$package" "-p:FpasSdkVersion=$sdkVersion" `
        --artifacts-path $artifacts --output "$artifacts/publish" -warnaserror
    if ($LASTEXITCODE -ne 0) {
        throw "Release publish failed (exit code $LASTEXITCODE)."
    }
    & 'src/Fpas.Deployment.WindowsPortable/Publish.ps1' -PayloadDirectory "$artifacts/publish" `
        -EntryPoint 'Fpas.Cli.exe' -Output 'bin/fpas.exe' -ArtifactsPath "$artifacts/portable"

    $cacheInfo = (dotnet nuget locals global-packages --list --force-english-output).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $cacheInfo.StartsWith('global-packages: ')) {
        throw 'Unable to locate the NuGet package cache.'
    }
    $cachePath = $cacheInfo.Substring('global-packages: '.Length)
    if (-not [System.IO.Path]::IsPathRooted($cachePath)) { throw 'NuGet package cache must be an absolute path.' }
    $cacheRoot = [System.IO.Path]::GetFullPath($cachePath)
    # Reused MSBuild nodes keep the cached build tasks loaded: Windows refuses to delete them,
    # and the nodes would go on running the old tasks after the refresh.
    dotnet build-server shutdown | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Stopping the .NET build servers failed (exit code $LASTEXITCODE)." }
    foreach ($packageId in @('fpas.net.sdk', 'fpas.ui.gui.cui')) {
        $packageCache = [System.IO.Path]::GetFullPath((Join-Path $cacheRoot $packageId))
        $versionCache = [System.IO.Path]::GetFullPath((Join-Path $packageCache $sdkVersion.ToLowerInvariant()))
        if ([System.IO.Path]::GetDirectoryName($versionCache) -ne $packageCache) { throw 'Invalid package cache target.' }
        if (Test-Path -LiteralPath $versionCache) {
            try { Remove-Item -LiteralPath $versionCache -Recurse -Force }
            catch { throw "Another process still uses $versionCache, such as an editor's build server. Close it and run dist-small.ps1 again. $_" }
        }
    }
    Write-Host "Deployed portable bin/fpas.exe with embedded Fpas.NET.Sdk $sdkVersion."
    Write-Host "Refreshed the local Fpas.NET.Sdk and Fpas.Ui.Gui.Cui $sdkVersion caches; consumers restore from the feed."
}
finally {
    Pop-Location
}
