#!/usr/bin/env sh
set -eu
CDPATH= cd -- "$(dirname -- "$0")"

# Build and deploy matching Release CLI and project SDK artifacts.
sdk_project=sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj
sdk_version=$(dotnet msbuild "$sdk_project" -nologo -getProperty:PackageVersion)
case "$sdk_version" in ''|*[!0-9A-Za-z.+-]*) printf '%s\n' 'Invalid SDK package version.' >&2; exit 1;; esac
case "$sdk_version" in [0-9]*) ;; *) printf '%s\n' 'Invalid SDK package version.' >&2; exit 1;; esac
mkdir -p sdk/feed
dotnet build "$sdk_project" --configuration Release --artifacts-path .temp-data/dist/unix -p:UseAppHost=true -warnaserror
dotnet pack "$sdk_project" --no-build --no-restore --configuration Release \
    --artifacts-path .temp-data/dist/unix -p:UseAppHost=true --output sdk/feed
dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --no-build --no-restore --configuration Release --self-contained false \
    -p:UseAppHost=true --artifacts-path .temp-data/dist/unix --output bin -warnaserror
mv -f bin/Fpas.Cli bin/fpas
chmod +x bin/fpas
cache_info=$(dotnet nuget locals global-packages --list --force-english-output)
case "$cache_info" in 'global-packages: '*) cache_root=${cache_info#global-packages: };; *) printf '%s\n' 'Unable to locate the NuGet package cache.' >&2; exit 1;; esac
case "$cache_root" in /*) ;; *) printf '%s\n' 'NuGet package cache must be an absolute path.' >&2; exit 1;; esac
cache_version=$(printf '%s' "$sdk_version" | tr '[:upper:]' '[:lower:]')
rm -rf -- "$cache_root/fpas.net.sdk/$cache_version"
printf '%s\n' "Deployed bin/fpas and sdk/feed/Fpas.NET.Sdk.$sdk_version.nupkg (requires .NET 10)." \
    "Refreshed the local Fpas.NET.Sdk $sdk_version cache; consumers restore from the feed."
