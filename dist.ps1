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
    if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') { throw 'FPAS distributions support Windows x64 and Linux x64.' }
    $artifacts = Join-Path $PSScriptRoot '.temp-data/dist/windows'
    $package = Join-Path $PSScriptRoot "sdk/feed/Fpas.NET.Sdk.$sdkVersion.nupkg"
    dotnet build $sdkProject --configuration Release --artifacts-path $artifacts -p:UseAppHost=true -p:DebugType=portable "-p:FpasSdkVersion=$sdkVersion" -warnaserror
    if ($LASTEXITCODE -ne 0) { throw "Release SDK build failed (exit code $LASTEXITCODE)." }
    dotnet pack $sdkProject --no-build --no-restore --configuration Release `
        --artifacts-path $artifacts -p:UseAppHost=true -p:DebugType=portable "-p:FpasSdkVersion=$sdkVersion" --output sdk/feed
    if ($LASTEXITCODE -ne 0) { throw "Release SDK pack failed (exit code $LASTEXITCODE)." }
    dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --configuration Release --runtime win-x64 `
        -p:FpasDistribution=true -p:DebugType=portable "-p:FpasSdkPackage=$package" "-p:FpasSdkVersion=$sdkVersion" `
        --artifacts-path $artifacts --output "$artifacts/publish" -warnaserror
    if ($LASTEXITCODE -ne 0) {
        throw "Release publish failed (exit code $LASTEXITCODE)."
    }
    New-Item -ItemType Directory -Force -Path bin | Out-Null
    Copy-Item -LiteralPath "$artifacts/publish/Fpas.Cli.exe" -Destination bin/fpas.exe -Force

    $cacheInfo = (dotnet nuget locals global-packages --list --force-english-output).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $cacheInfo.StartsWith('global-packages: ')) {
        throw 'Unable to locate the NuGet package cache.'
    }
    $cachePath = $cacheInfo.Substring('global-packages: '.Length)
    if (-not [System.IO.Path]::IsPathRooted($cachePath)) { throw 'NuGet package cache must be an absolute path.' }
    $cacheRoot = [System.IO.Path]::GetFullPath($cachePath)
    $sdkCache = [System.IO.Path]::GetFullPath((Join-Path $cacheRoot 'fpas.net.sdk'))
    $versionCache = [System.IO.Path]::GetFullPath((Join-Path $sdkCache $sdkVersion.ToLowerInvariant()))
    if ([System.IO.Path]::GetDirectoryName($versionCache) -ne $sdkCache) { throw 'Invalid SDK cache target.' }
    if (Test-Path -LiteralPath $versionCache) { Remove-Item -LiteralPath $versionCache -Recurse -Force }
    Write-Host "Deployed self-contained single-file bin/fpas.exe with embedded Fpas.NET.Sdk $sdkVersion."
    Write-Host "Refreshed the local Fpas.NET.Sdk $sdkVersion cache; consumers restore from the feed."
}
finally {
    Pop-Location
}
