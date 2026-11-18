# FpasAmp.Imaging

FPas-Amp uses [Fpas.Media.Images](../../../../src/Fpas.Media.Images/README.md)
for decoding and owns the album thumbnail cache here.

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
