#!/usr/bin/env sh
set -eu

# Build an installable Debian package from the current Git working tree.
# FPAS_DEB_OUTPUT_DIR overrides bin/; FPAS_DEB_MAINTAINER overrides the
# generic package maintainer. The build uses dist.sh's release profile.
CDPATH= cd -- "$(dirname -- "$0")"
root=$(pwd -P)

for tool in cargo date dpkg dpkg-deb dpkg-shlibdeps git awk mktemp find xargs; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Required tool not found: $tool" >&2
        exit 1
    fi
done

base_version=$(awk '
    { sub(/\r$/, "") }
    /^\[workspace.package\]$/ { section = 1; next }
    section && /^\[/ { exit }
    section && /^version[[:space:]]*=/ {
        gsub(/^[^"]*"|".*/, "")
        print
        exit
    }
' Cargo.toml)
if [ -z "$base_version" ]; then
    echo "Cannot determine the workspace version from Cargo.toml" >&2
    exit 1
fi
commit=$(git rev-parse --short=12 HEAD)
timestamp=$(date -u +%Y%m%d.%H%M%S.%N)
case "$timestamp" in
    *[!0-9.]*) echo 'GNU date with nanosecond support is required' >&2; exit 1 ;;
esac
version="$base_version~dev$timestamp+g$commit"
if [ -n "$(GIT_OPTIONAL_LOCKS=0 git status --porcelain --untracked-files=normal)" ]; then
    version="$version+dirty"
fi
version="$version-1"
dpkg --validate-version "$version"
architecture=$(dpkg --print-architecture)
output_dir=$(printenv FPAS_DEB_OUTPUT_DIR || true)
if [ -z "$output_dir" ]; then output_dir="$root/bin"; fi
mkdir -p -- "$output_dir"
output_dir=$(CDPATH= cd -- "$output_dir" && pwd -P)
package="$output_dir/fpas_${version}_${architecture}.deb"
if [ -e "$package" ]; then
    echo "Package already exists: $package; retry to generate a new timestamp" >&2
    exit 1
fi

maintainer=$(printenv FPAS_DEB_MAINTAINER || true)
if [ -z "$maintainer" ]; then
    maintainer='Functional Pascal contributors <noreply@github.com>'
fi
case "$maintainer" in
    *'
'*) echo 'FPAS_DEB_MAINTAINER must be a single line' >&2; exit 1 ;;
esac

work=$(mktemp -d "${TMPDIR:-/tmp}/fpas-deb.XXXXXXXX")
temporary_package=
cleanup() {
    if [ -n "$temporary_package" ]; then rm -f -- "$temporary_package"; fi
    rm -rf -- "$work"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
stage="$work/debian/fpas"
mkdir -p "$stage/DEBIAN" "$stage/opt/fpas" "$stage/usr/bin" \
    "$stage/usr/share/doc/fpas" "$work/debian"

sh "$root/dist.sh"
install -m 0755 "$root/bin/fpas" "$root/bin/fpas-runner" \
    "$root/bin/fpas-lsp" "$stage/opt/fpas/"
cp -a "$root/bin/lib" "$stage/opt/fpas/lib"
cp -a "$root/docs" "$stage/usr/share/doc/fpas/docs"
(cd "$root" && git ls-files -z --cached --others --exclude-standard -- examples |
    xargs -0 -r cp -a --parents -t "$stage/usr/share/doc/fpas")
# Documentation also links to source, tests, tools, applications and skills.
# Include their tracked files so those relative links remain available offline.
(cd "$root" && git ls-files -z --cached -- crates tests apps tools scripts editors \
    .agents AGENTS.md CONTRIBUTING.md |
    xargs -0 -r cp -a --parents -t "$stage/usr/share/doc/fpas")
install -m 0644 "$root/README.md" "$root/LICENSE" "$stage/usr/share/doc/fpas/"
install -m 0644 "$root/LICENSE" "$stage/usr/share/doc/fpas/copyright"
mkdir -p "$stage/usr/share/doc/fpas/lib"
install -m 0644 "$root/lib/Cargo.toml" "$stage/usr/share/doc/fpas/lib/Cargo.toml"
for entry in "$stage/opt/fpas/lib"/*; do
    if [ ! -e "$entry" ] && [ ! -L "$entry" ]; then continue; fi
    name=${entry##*/}
    destination="$stage/usr/share/doc/fpas/lib/$name"
    if [ ! -e "$destination" ] && [ ! -L "$destination" ]; then
        ln -s "/opt/fpas/lib/$name" "$destination"
    fi
done
for tool in fpas fpas-runner fpas-lsp; do
    ln -s "/opt/fpas/$tool" "$stage/usr/bin/$tool"
done
find "$stage/opt/fpas/lib" "$stage/usr/share/doc/fpas" -type d -exec chmod 0755 {} +
find "$stage/opt/fpas/lib" "$stage/usr/share/doc/fpas" -type f -exec chmod 0644 {} +

cat > "$work/debian/control" <<EOF
Source: fpas
Section: devel
Priority: optional
Maintainer: $maintainer
Standards-Version: 4.7.0

Package: fpas
Architecture: any
Description: Functional Pascal compiler and toolchain
EOF
dependencies=$(cd "$work" && dpkg-shlibdeps -O \
    "$stage/opt/fpas/fpas" "$stage/opt/fpas/fpas-runner" \
    "$stage/opt/fpas/fpas-lsp")
case "$dependencies" in
    shlibs:Depends=*) dependencies=${dependencies#shlibs:Depends=} ;;
    *) echo 'Unable to determine Debian runtime dependencies' >&2; exit 1 ;;
esac

cat > "$stage/DEBIAN/control" <<EOF
Package: fpas
Version: $version
Architecture: $architecture
Maintainer: $maintainer
Section: devel
Priority: optional
Depends: $dependencies
Homepage: https://github.com/tobiasbick/functional-pascal
Description: Functional Pascal compiler and toolchain
 Functional Pascal compiler, runner, language server, standard library,
 examples, and complete project documentation.
EOF
temporary_package=$(mktemp "$output_dir/.fpas-deb.XXXXXXXX")
dpkg-deb --root-owner-group --build "$stage" "$temporary_package"
chmod 0644 "$temporary_package"
if ! ln -- "$temporary_package" "$package"; then
    echo "Cannot publish package without overwriting an existing file: $package" >&2
    echo 'The output filesystem must support hard links.' >&2
    exit 1
fi
rm -f -- "$temporary_package"
temporary_package=
printf 'Built Debian package: %s\n' "$package"
