# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Release | [v1.3.0](https://github.com/yappologistic/CUI/releases/tag/v1.3.0) |
| Commit | `f70a1d3d095cafd67814b23bdd82c2dfba28efe0` |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The two files are the release's amalgamated C source and public header. Both
match the release assets byte for byte and can be reconstructed from the tagged
source tree.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `07f2be6fec062fc5a44fe4cec430f27282151a04bf5f8f4715de50bd7b9098c6` |
| `cui.h` | `81431e25aa4d2373777d1e6510a6f5d95f33e7c4e1a679841048539d418b7840` |

Project-owned integration belongs under `src/Fpas.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## License inventory

The paths in the middle column refer to the pinned upstream repository.
Standalone notices accompany binary distributions.

| Local notice | Upstream source | Terms |
| --- | --- | --- |
| [LICENSE-CUI.txt](LICENSE-CUI.txt) | `LICENSE` | MIT |
| [LICENSE-stb.txt](LICENSE-stb.txt) | License blocks in `third_party/stb_image.h`, `stb_image_write.h`, `stb_rect_pack.h`, `stb_truetype.h` | MIT or public domain, at the recipient's option |
| [LICENSE-Unicode.txt](LICENSE-Unicode.txt) | `third_party/LICENSE-Unicode.txt` | Unicode License V3 |
| [LICENSE-Inter.txt](LICENSE-Inter.txt) | `assets/fonts/LICENSE-Inter.txt` | SIL Open Font License 1.1 |
| [LICENSE-Geist.txt](LICENSE-Geist.txt) | `assets/fonts/LICENSE-Geist.txt` | SIL Open Font License 1.1 |
| [LICENSE-Lucide.txt](LICENSE-Lucide.txt) | `assets/fonts/LICENSE-Lucide.txt` | ISC, including Feather-derived icons under MIT |

The CUI, Unicode and font/icon notice files are copied unchanged. The stb notice
contains the dual-license text from the stb code in `cui.c`.

Keep these notices when redistributing the dependency, including native,
single-file and NativeAOT application distributions.

## Updating the dependency

1. Choose an upstream release deliberately and resolve its tag to a commit.
2. Download `cui.c` and `cui.h` from the release. Review source and license changes
   against the pinned release.
3. Verify the SHA-256 digests and check that the files match an amalgamation of
   the tagged sources. Copy the assets without changing their bytes.
4. Refresh the version, commit and digests. Use the tagged sources to check
   whether the license inventory or notice files need updating.
5. Run the native, binding and FPAS-consumer checks.
