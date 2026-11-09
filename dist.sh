#!/usr/bin/env sh
set -eu
CDPATH= cd -- "$(dirname -- "$0")"

# Build and deploy matching Release CLI and project SDK artifacts.
sdk_project=sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj
sdk_version=$(dotnet msbuild "$sdk_project" -nologo -getProperty:PackageVersion)
case "$sdk_version" in ''|*[!0-9A-Za-z.+-]*) printf '%s\n' 'Invalid SDK package version.' >&2; exit 1;; esac
case "$sdk_version" in [0-9]*) ;; *) printf '%s\n' 'Invalid SDK package version.' >&2; exit 1;; esac
mkdir -p sdk/feed
case "$(uname -s)-$(uname -m)" in Linux-x86_64) rid=linux-x64;; *) printf '%s\n' 'FPAS distributions support Windows x64 and Linux x64.' >&2; exit 1;; esac
artifacts="$PWD/.temp-data/dist/unix"
package="$PWD/sdk/feed/Fpas.NET.Sdk.$sdk_version.nupkg"
dotnet build "$sdk_project" --configuration Release --artifacts-path "$artifacts" -p:UseAppHost=true -p:DebugType=portable "-p:FpasSdkVersion=$sdk_version" "-p:FpasCuiDistributionRid=$rid" -warnaserror
dotnet pack "$sdk_project" --no-build --no-restore --configuration Release \
    --artifacts-path "$artifacts" -p:UseAppHost=true -p:DebugType=portable "-p:FpasSdkVersion=$sdk_version" "-p:FpasCuiDistributionRid=$rid" --output sdk/feed
dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --configuration Release --runtime "$rid" \
    -p:FpasDistribution=true -p:DebugType=portable "-p:FpasSdkPackage=$package" "-p:FpasSdkVersion=$sdk_version" \
    --artifacts-path "$artifacts" --output "$artifacts/publish" -warnaserror
mkdir -p bin
cp -f "$artifacts/publish/Fpas.Cli" bin/fpas
chmod +x bin/fpas
cache_info=$(dotnet nuget locals global-packages --list --force-english-output)
case "$cache_info" in 'global-packages: '*) cache_root=${cache_info#global-packages: };; *) printf '%s\n' 'Unable to locate the NuGet package cache.' >&2; exit 1;; esac
case "$cache_root" in /*) ;; *) printf '%s\n' 'NuGet package cache must be an absolute path.' >&2; exit 1;; esac
cache_version=$(printf '%s' "$sdk_version" | tr '[:upper:]' '[:lower:]')
rm -rf -- "$cache_root/fpas.net.sdk/$cache_version"
printf '%s\n' "Deployed self-contained single-file bin/fpas with embedded Fpas.NET.Sdk $sdk_version." \
    "Refreshed the local Fpas.NET.Sdk $sdk_version cache; consumers restore from the feed."
