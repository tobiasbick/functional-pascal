# FpasAmp.Imaging

Application-local MagicScaler image decoding/resizing and thumbnail storage.
FPAS owns album identity, source selection, asynchronous lifetime and UI state.
This small CLR boundary handles MagicScaler's `Span<byte>` pixel-copy API and
BGRA-to-RGBA conversion. Drawing remains in CUI.

`ThumbnailStore` accepts encoded pictures on background callers and returns
immutable-by-contract straight-alpha RGBA `Bitmap` values. Pictures are fitted
into a transparent 384 × 384 square. Inputs are bounded to sixteen MiB and
64 megapixels. No music/source file is modified and no URL is fetched.

Encoded image SHA-256 keys share pixel blobs across albums. Per-album references
include the observed audio/sidecar revision; unchanged cached pixels work offline.
Corrupt/missing cache files are ignored and rebuilt. Disk blobs use a 256 MiB
budget with access recency eviction; decoded cache entries use 32 MiB. The GUI
separately retains only its realized viewport. Writes replace temporary files
atomically. Source reads and decoding occur outside the GUI thread.

MagicScaler and its libjpeg/libpng codec plugins come from NuGet. Both plugins
are registered once and used on Windows and Linux, including the native assets
supplied for x64 and ARM64. The selected codec packages are upstream preview
releases. MagicScaler fits pictures without cropping, normalizes orientation/color,
and supplies straight-alpha pixels for the transparent square. MIT licenses and
native codec notices are copied beside application outputs.
See [PhotoSauce](https://github.com/saucecontrol/PhotoSauce).
