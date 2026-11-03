#!/usr/bin/env sh
set -eu
CDPATH= cd -- "$(dirname -- "$0")"

# Build and deploy the Release compiler and its dependencies to bin/.
dotnet publish src/Fpas.Cli/Fpas.Cli.csproj --configuration Release --self-contained false \
    -p:UseAppHost=true --artifacts-path .temp-data/dist/unix --output bin -warnaserror
mv -f bin/Fpas.Cli bin/fpas
chmod +x bin/fpas
printf '%s\n' 'Deployed Release compiler to bin/fpas (requires .NET 10).'
