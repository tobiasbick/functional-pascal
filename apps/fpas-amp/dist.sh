#!/usr/bin/env sh
# [Documentation](doc/desktop.md#linux-aot-build)
set -eu
CDPATH= cd -- "$(dirname -- "$0")"
repository=$(CDPATH= cd -- ../.. && pwd)
artifacts="$repository/.temp-data/fpas-amp-linux"
publish="$artifacts/publish"
bin="$repository/bin"

dotnet publish src/FpasAmp/FpasAmp.fpasproj -c Release -r linux-x64 -p:PublishAot=true \
    --artifacts-path "$artifacts/build" --output "$publish" --nologo

mkdir -p "$bin"
for asset in "$publish"/*; do
    case "${asset##*/}" in
        fpas-amp|*.dbg|*.pdb|*.xml) ;;
        *) cp -Rp "$asset" "$bin/" ;;
    esac
done
install -m 755 "$publish/fpas-amp" "$bin/fpas-amp"
printf '%s\n' 'Deployed Native AOT bin/fpas-amp.'
