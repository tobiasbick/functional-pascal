# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> |
| Revision | Upstream `master` after [v1.4.0](https://github.com/yappologistic/CUI/releases/tag/v1.4.0), mirrored in the source fork |
| Commit | [`71c383fd3c20cfaf06adb2faf7dbbbdaa7074441`](https://github.com/yappologistic/CUI/commit/71c383fd3c20cfaf06adb2faf7dbbbdaa7074441) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`python3 tools/amalgamate.py` on Linux in the source fork at the pinned commit.
This upstream snapshot includes window state and placement from
[PR #95](https://github.com/yappologistic/CUI/pull/95), and the file-picker
drive sidebar, filter-height, selection and double-click fixes from
[PR #97](https://github.com/yappologistic/CUI/pull/97). No combined fork branch
is needed.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `0f0bf605d82ade1209d206d6f0cd5b9d4c366f02ee36033a61c904a71ae009c3` |
| `cui.h` | `63e6ecd1de3ff994f423f79385a59294c5ef580efdfbe8b434b8951e92e425e7` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## Snapshot verification

The copied pair matches the generated files byte for byte. License inputs and
the theme builder are unchanged from the previous pin; all 16 generated palettes
match the prior reference. The upstream snapshot includes the GLFW normal-size
follow-up to #95 and the container-mount exclusion follow-up to #97.

The release-frame calendar fix from
[PR #93](https://github.com/yappologistic/CUI/pull/93), applied as
[`5830981`](https://github.com/yappologistic/CUI/commit/5830981), replaces the
CUI-032 workaround. Both the direct-CUI and FPAS-adapter regression checks now
require a drag ending in the release frame to return no activation.

The native integration suite passed all 57 tests on Windows x64 with MSVC and
all 55 tests on Linux x64 under WSL with GCC. The planning checks exercise
both direct CUI and the adapter, distinguishing movement before release from
movement after a completed click.

The upstream unit, file-place, no-home and window-state tests passed on
Windows. The first three passed under WSL, and the window-state test passed
with X11/Openbox under Xvfb with both software and OpenGL rendering.

The chart regression still reproduces CUI-019. Removing the CUI-033 guards in
an isolated copy fails all six interrupted-drag cases, so those guards remain.

The managed binding suite passed 271 tests on Windows and 270 on Linux under
WSL, with two and three platform or availability skips respectively. Both
FPAS-consumer tests passed on each platform, including the gallery with the
software and OpenGL renderers. The Windows build reports the existing upstream
Gantt shadowing warning C4456.

Publishing, NativeAOT, macOS, and SDL2/GLFW runtime checks were not run for
this update.

See [CUI upstream bugs](../../docs/future/cui-upstream-bugs.md) for remaining defects.

## Changes from the previous snapshot

The seven commits after `a5c9ef5` leave the generated public header byte-identical.
The public API and widget inventory are unchanged.

- The week calendar measures movement between the recorded press and release
  positions. Motion after release preserves a completed click. Focus loss
  records a release position only while the left button is held.
- GLFW restoration before the first frame cancels a queued initial maximize.
  SDL2 retains a maximize or restore command until the next frame confirms it.
- File-place tests now cover a missing home directory and disabled native
  filesystem support.

The planning regression uses these event-order semantics. Existing bindings
continue to use the same public declarations.

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
