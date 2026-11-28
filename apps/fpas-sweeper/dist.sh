#!/usr/bin/env sh
# [Documentation](doc/README.md#distribution)
set -eu
CDPATH= cd -- "$(dirname -- "$0")"
repository=$(CDPATH= cd -- ../.. && pwd)
artifacts="$repository/.temp-data/fpas-sweeper-linux"
publish="$artifacts/publish"
bin="$repository/bin"

dotnet publish src/FpasSweeper/FpasSweeper.fpasproj -c Release -r linux-x64 -p:PublishAot=true \
    --artifacts-path "$artifacts/build" --output "$publish" --nologo

mkdir -p "$bin"
cp -Rp "$publish/licenses" "$publish/share" "$bin/"
install -m 755 "$publish/fpas-sweeper" "$bin/fpas-sweeper"
printf '%s\n' 'Deployed Native AOT bin/fpas-sweeper.'
