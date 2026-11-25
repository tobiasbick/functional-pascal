# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> |
| Revision | Upstream `master` after [v1.4.0](https://github.com/yappologistic/CUI/releases/tag/v1.4.0) |
| Commit | [`e2a0ad2a493e22eb1d96beb4991f50dc2c4ac435`](https://github.com/yappologistic/CUI/commit/e2a0ad2a493e22eb1d96beb4991f50dc2c4ac435) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`python3 tools/amalgamate.py` on Linux from the upstream tree at the pinned
commit. No combined fork branch is needed.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `fe7444e631d67c27499fb0e93ae72fd03575b5bb1fd14845b71710923ea8ff07` |
| `cui.h` | `8057c2fdfb6e4d4797b4f3abb26b91387474e76d3173fb6294cbf783c3b62bc1` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## Changes from `e0124ea` to `e2a0ad2`

- CUI-046. The Windows Save dialog keeps the suggested directory even when
  file creation there is denied. Open and Save restore the process working
  directory. [Issue #121](https://github.com/yappologistic/CUI/issues/121) ·
  [PR #125](https://github.com/yappologistic/CUI/pull/125)
- CUI-047. CUI runs the Windows folder chooser on an STA thread when the host
  uses MTA. The binding now delegates both Save and Choose Folder to CUI.
  [Issue #122](https://github.com/yappologistic/CUI/issues/122) ·
  [PR #126](https://github.com/yappologistic/CUI/pull/126)
- CUI-049. The frame allocator already clears its result. Removing a redundant
  clear in `cui__em_button` lets the native build enable GCC's bounds and
  string-overflow diagnostics again.
  [Issue #123](https://github.com/yappologistic/CUI/issues/123) ·
  [PR #127](https://github.com/yappologistic/CUI/pull/127)
- CUI-050. With the pointer on a different day, Enter activates the
  keyboard-selected day and Space preserves the selection. The month-calendar
  adapter passes input directly to CUI. [Issue #124](https://github.com/yappologistic/CUI/issues/124) ·
  [PR #128](https://github.com/yappologistic/CUI/pull/128)

The public header adds optional chart X-axis bounds and annotation colors,
tree-table drag and drop, and `cui_tree_move`. The existing adapters use the
default values for the new options. The theme palettes, fonts and license
inputs match the previous pin.

| Release verification | Windows x64 | Linux x64 under WSL |
| --- | --- | --- |
| Native FPAS tests | 58 passed | 55 passed |
| CUI binding tests | 282 passed, 7 skipped | 288 passed, 6 skipped |
| Terminal CUI contract and dependency check | 2 passed | 2 passed |

The bindings exercised the FPAS gallery and the month-calendar keyboard and
pointer checks with both the software and OpenGL renderers. On Windows the
upstream core, Save, MTA and STA tests passed. Under WSL the upstream core
suite passed, GCC 15.2 compiled the amalgamation at `-O3` with
`-Werror=array-bounds -Werror=stringop-overflow`, and Xvfb ran the Linux
bindings, including all six Zenity selection and cancellation cases.

An additional Xvfb run with Openbox failed 15 size or image checks and the
gallery voice-orb interaction. All 16 failures also reproduced at the previous
pin, `e0124ea`. The Linux results above use the
[documented Xvfb configuration](../../docs/ui/gui/cui.md).

See [CUI upstream bugs](../../docs/future/cui-upstream-bugs.md) for remaining
open defects (`CUI-019`).

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
5. Check the semantic palettes and neutral-light defaults exported by CUI's theme builder
   against [`Themes.fpas`](../../src/Fpas.Ui/Widgets/Theming/Themes.fpas).
   These values supply both hosts without adding native dependencies to TUI.
   Keep token and accent order aligned with the header; the native bridge
   checks the token count at compile time.
6. Run the native, binding and FPAS-consumer checks, including shared-theme
   and layout parity in `GuiHostTests` and the terminal CUI contract fixture.
