# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Release | [v1.3.1](https://github.com/yappologistic/CUI/releases/tag/v1.3.1) |
| Commit | `e8688b8fd85c7b07afbddbac5fc748a905c12be0` |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The two files are the release's amalgamated C source and public header. Both
match the release assets byte for byte and can be reconstructed from the tagged
source tree.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `6665f60ae2330b6166a4de9c0755dfef7fe1f2b8d2a624222a016dc5e8cfdd35` |
| `cui.h` | `96c3fabaa5c72145fcf10aa319f93f7d608d005a1f9a510ac73f5e9336c2eae8` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
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
5. Check the semantic palettes exported by CUI's theme builder against
   [`ThemePalettes.fpas`](../../src/Fpas.Ui/Internal/ThemePalettes.fpas) and the
   neutral-light defaults in [`Themes.fpas`](../../src/Fpas.Ui/Themes.fpas).
   These values supply both hosts without adding native dependencies to TUI.
   Keep token and accent order aligned with the header; the native bridge
   checks the token count at compile time.
6. Run the native, binding and FPAS-consumer checks, including shared-theme
   and layout parity in `GuiHostTests` and the terminal CUI contract fixture.
