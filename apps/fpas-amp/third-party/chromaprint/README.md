# Chromaprint fpcalc

FPas-Amp runs the unmodified upstream Chromaprint 1.6.2 executable as a separate
process on Windows x64 and Linux x64. It reads local audio with the bundled
FFmpeg 8.0 decoder. The default analysis window is the first 120 seconds.

The build downloads pinned archives and checks their SHA-256 digests. It copies
the executable to `tools/chromaprint` and installs the license notices and
corresponding source archives under `licenses/chromaprint`. The Chromaprint
archive contains the `package/build.sh` release recipe. The ffmpeg-build
archive contains the matching decoder configuration and platform build scripts.
The player can run offline after the build.

Chromaprint's own code uses the MIT license. Upstream distributes the combined
tool under LGPL 2.1. See `Chromaprint-LICENSE.md` and `LGPL-2.1.txt`.

- [Chromaprint 1.6.2 release](https://github.com/acoustid/chromaprint/releases/tag/v1.6.2)
- [FFmpeg 8.0 sources](https://ffmpeg.org/releases/ffmpeg-8.0.tar.xz)
- [AcoustID's FFmpeg build scripts](https://github.com/acoustid/ffmpeg-build/tree/v8.0-1)
