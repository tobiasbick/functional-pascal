# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> |
| Revision | Upstream snapshot after [v1.4.0](https://github.com/yappologistic/CUI/releases/tag/v1.4.0) |
| Commit | [`a8f71c05be29eb441bf3ff72dba67b279e68384f`](https://github.com/yappologistic/CUI/commit/a8f71c05be29eb441bf3ff72dba67b279e68384f) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`python3 tools/amalgamate.py` on Linux in the source fork at the pinned commit.
That commit is identical to upstream `master` at the time of this update.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `3a3650864335abf6a4cfc8609532df903a4b270e473846794bac408db1dff924` |
| `cui.h` | `e725933e9022064317af4d53a16eeb536c5875a157c0bfe767de60e631521fa9` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## Snapshot verification

The copied pair matches the generated files byte for byte. This revision
updates select keyboard handling, number-field reconciliation, and input
method positioning. The source changes leave license inputs, theme tokens,
and palette definitions unchanged.

The FPAS native CTest suite passed in Debug and Release on Windows x64 and
Linux x64. Both distribution scripts produced a working launcher and embedded
SDK. The packaged CUI probe ran in all five publish modes on both hosts.
The packaged FPAS gallery ran all five modes with both renderers on Windows
and Linux. The X11 gallery input test passed under Xvfb.

See [CUI upstream bugs](../../docs/future/cui-upstream-bugs.md) for remaining defects.

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

1. Choose an upstream commit. Resolve the tag when updating to a release.
2. Review source and license changes against the previous pin. Generate the
   pair with `python3 tools/amalgamate.py` on Linux or WSL so it uses LF endings.
3. Copy `cui.c` and `cui.h` from `build/amalgamation/` and verify that their
   SHA-256 digests match the generated files. For a release, compare the assets.
4. Refresh the revision, commit and digests. Check whether the new upstream
   tree changes the license inventory or notices.
5. Check the semantic palettes exported by CUI's theme builder against
   [`ThemePalettes.fpas`](../../src/Fpas.Ui/Internal/Theming/ThemePalettes.fpas) and the
   neutral-light defaults in [`Themes.fpas`](../../src/Fpas.Ui/Widgets/Theming/Themes.fpas).
   These values supply both hosts without adding native dependencies to TUI.
   Keep token and accent order aligned with the header; the native bridge
   checks the token count at compile time.
6. Run the native, binding and FPAS-consumer checks, including shared-theme
   and layout parity in `GuiHostTests` and the terminal CUI contract fixture.
