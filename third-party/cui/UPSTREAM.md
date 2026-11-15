# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> |
| Revision | Source fork branch `window-state`, proposed upstream in [pull request #95](https://github.com/yappologistic/CUI/pull/95), on upstream [`6e6b353`](https://github.com/yappologistic/CUI/commit/6e6b35326f655405822fcf863fac0d3bb35bb787) after [v1.4.0](https://github.com/yappologistic/CUI/releases/tag/v1.4.0) |
| Commit | [`a18f6616ba466a95ec6e9d64f4fef859d51930fd`](https://github.com/tobiasbick/CUI/commit/a18f6616ba466a95ec6e9d64f4fef859d51930fd) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`python3 tools/amalgamate.py` on Linux in the source fork at the pinned commit.
That commit adds window state and placement to upstream `master` at `6e6b353`
and is not part of upstream yet. Once upstream contains the change, pin the
upstream commit again.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `58478ed4de7346a210fdb4128744f96afd5e4578d7799637f2435b39400d83fe` |
| `cui.h` | `c9c3ff679e0b574380dcc26163a3741f0c7d9d642cc4555097867f1f11a17585` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## Snapshot verification

The copied pair matches the generated files byte for byte. This snapshot
includes the seven groups of fixes for collection widgets, chart interactions,
read-only widgets, numeric editors, focus and popups, pickers, and disabled
widgets, plus the window state and placement API of pull request #95. The
upstream license inputs and theme builder are unchanged.

The native integration suite passed 37 tests on Windows and 35 on Linux under
WSL. Five upstream test suites built against the amalgamation passed on both
platforms. The 25 selected .NET GUI tests passed on both platforms, including
the gallery with software and OpenGL renderers.

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
