# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Release | [v1.2.0](https://github.com/yappologistic/CUI/releases/tag/v1.2.0) |
| Commit | `7bb08ff7c56eb912fe7133985f8b7fa65098ea2e` |
| Build inputs | [`upstream/`](upstream/) |
| Local source modifications | None |

The vendored CMake file, sources, public headers and bundled third-party headers
retain their upstream paths and bytes. The following Git object IDs identify the
copied paths in the pinned commit.

| Path under `upstream/` | Upstream Git object ID |
| --- | --- |
| `CMakeLists.txt` | `cbe9d6f02522064fd26a432bce97133404bf25f0` |
| `include/` | `e0b69bb531220662ac26d2fb3dc5d37102a5c86f` |
| `src/` | `1344119548fb37563d77281db85ae0440e2cd282` |
| `third_party/` | `82705a8c2982bc60838a5804c7508a12b9fd4a32` |

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
contains the dual-license text from the bundled stb headers, which retain their
original license blocks.

Keep these notices when redistributing the dependency, including native,
single-file and NativeAOT application distributions.

## Updating the dependency

1. Choose an upstream release deliberately and resolve its tag to a commit.
2. Export the build paths listed above and obtain the license notices from the
   same commit. Review changes in source, build configuration and licensing.
3. Compare `git hash-object --no-filters <vendored-file>` for each copied file
   with its blob ID from `git ls-tree -r <commit>`. Preserve line endings during
   export and checkout.
4. Replace the vendored files and notices together. Check every bundled component
   for license changes and refresh the version, commit, object IDs and inventory.
5. Run the native, binding and FPAS-consumer checks.
