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
    dotnet build $sdkProject --configuration Release --artifacts-path .temp-data/dist/windows -p:UseAppHost=true -warnaserror
    if ($LASTEXITCODE -ne 0) { throw "Release SDK build failed (exit code $LASTEXITCODE)." }
    dotnet pack $sdkProject --no-build --no-restore --configuration Release `
        --artifacts-path .temp-data/dist/windows -p:UseAppHost=true --output sdk/feed
    if ($LASTEXITCODE -ne 0) { throw "Release SDK pack failed (exit code $LASTEXITCODE)." }
    dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --no-build --no-restore --configuration Release --self-contained false `
        -p:UseAppHost=true --artifacts-path .temp-data/dist/windows --output bin -warnaserror
    if ($LASTEXITCODE -ne 0) {
        throw "Release publish failed (exit code $LASTEXITCODE)."
    }
    Move-Item -LiteralPath bin/Fpas.Cli.exe -Destination bin/fpas.exe -Force

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
    Write-Host "Deployed bin/fpas.exe and sdk/feed/Fpas.NET.Sdk.$sdkVersion.nupkg (requires .NET 10)."
    Write-Host "Refreshed the local Fpas.NET.Sdk $sdkVersion cache; consumers restore from the feed."
}
finally {
    Pop-Location
}
