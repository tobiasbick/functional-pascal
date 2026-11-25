# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> (`master` matches the pin) |
| Revision | [v1.5.0](https://github.com/yappologistic/CUI/releases/tag/v1.5.0) |
| Commit | [`f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3`](https://github.com/yappologistic/CUI/commit/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`tools/amalgamate.py` from the upstream tree at the pinned commit, with LF line
endings.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `2d730b79c279567b619431b929c76845c366118085b833de4eabc518fa2d7c7a` |
| `cui.h` | `4882d7d1c67edb53aa58eb9f842886154f6dfba55192b37e406846dab9247afc` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here. Open upstream defects and their FPAS
workarounds are tracked in [CUI upstream bugs](../../docs/future/cui-upstream-bugs.md).

## Verification

| Check | Windows x64 | Linux x64 under WSL |
| --- | --- | --- |
| Native FPAS tests | 58 passed | 55 passed, shared and static |
| CUI binding tests | 291 passed, 8 skipped | 291 passed, 8 skipped |

The Linux amalgamation is byte-identical to the vendored pair. Skipped binding
tests need the other platform's OS input, native dialogs or the publishing
matrices. The [binding documentation](../../docs/ui/gui/cui.md#source-build-and-verification)
describes the test commands and the Xvfb configuration for Linux.

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
   Fast-forward the fork's `master` to it.
2. Review source and license changes against the previous pin. Check public
   structs that the binding mirrors, such as `cui_input_event`.
3. Generate the pair with `python3 tools/amalgamate.py` on Linux or WSL so it
   uses LF endings. Copy `cui.c` and `cui.h` from `build/amalgamation/` and
   verify that their SHA-256 digests match the generated files.
4. Refresh the revision, commit and digests. Check whether the new upstream
   tree changes the license inventory or notices.
5. Check the semantic palettes and neutral-light defaults exported by CUI's theme builder
   against [`Themes.fpas`](../../src/Fpas.Ui/Widgets/Theming/Themes.fpas).
   These values supply both hosts without adding native dependencies to TUI.
   Keep token and accent order aligned with the header; the native bridge
   checks the token count at compile time.
6. Run the native, binding and FPAS-consumer checks, including shared-theme
   and layout parity in `GuiHostTests` and the terminal CUI contract fixture.
   Record the results under [Verification](#verification).
7. Verify the reproductions of the [open upstream bugs](../../docs/future/cui-upstream-bugs.md)
   and remove workarounds whose defects the new pin fixes.
