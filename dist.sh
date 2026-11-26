#!/usr/bin/env sh
# [Documentation](docs/build/distribution.md#build-and-deliver)
set -eu
CDPATH= cd -- "$(dirname -- "$0")"

sh ./dist-small.sh

artifacts="$PWD/.temp-data/dist/apps-linux-x64"

publish() {
    dotnet publish "$1" --configuration Release --runtime linux-x64 -p:PublishAot=true \
        --artifacts-path "$artifacts/build" --output "$artifacts/$2" --nologo
}

publish_app() {
    name=$2
    publish "$1" "$name"
    install -m 755 "$artifacts/$name/$name" "bin/$name"
    if [ -d "$artifacts/$name/licenses" ]; then
        cp -R "$artifacts/$name/licenses" bin/
    fi
}

publish_app examples/ui/gui/gallery/GuiWidgetGallery.fpasproj GuiWidgetGallery
publish_app examples/ui/tui/gallery/TuiWidgetGallery.fpasproj TuiWidgetGallery
publish_app apps/fpas-sweeper/src/FpasSweeper/FpasSweeper.fpasproj FpasSweeper

publish apps/fpas-amp/src/FpasAmp/FpasAmp.fpasproj FpasAmp
for asset in "$artifacts/FpasAmp/"*; do
    case "$asset" in
        */FpasAmp|*.dbg|*.pdb|*.xml) ;;
        *) cp -R "$asset" bin/ ;;
    esac
done
install -m 755 "$artifacts/FpasAmp/FpasAmp" bin/FpasAmp
printf '%s\n' 'Deployed both widget galleries, FpasSweeper and FpasAmp in bin/.'
