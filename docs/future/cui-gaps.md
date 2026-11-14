# CUI gallery gaps

The [CUI widget inventory](../cui-widget-inventory.md) records accepted widgets
and current terminal coverage. Desktop host and publishing work is tracked in
[Desktop GUI application gaps](gui-app-gaps.md).

## Strict gallery pixel parity

- [ ] Remove the remaining single-channel antialiasing differences in wide page
  composition. At `86d9781`, Linux software captures at 1240x1700, scale 1,
  differ by one 8-bit channel step at 182/184 pixels in the Inputs picker card
  (dark/light), 487 in dark Feedback progress and one in dark Loading. The
  measured wide picker positions differ by about 0.00012 logical points from
  direct CUI. Ten of twelve reviewed Inputs regions and four of six reviewed
  Feedback regions match exactly. Comparisons exclude the reserved sample email,
  unsynchronized activity animations and the unbound skeleton group. The native
  widget and calendar tests compare at identical positions and times and pass
  without a pixel tolerance. Keep strict whole-page parity open until the
  composition differences are resolved.

- [ ] Match the supplementary App chrome composition to CUI's documentation
  captures. At 1100 and 550 logical points, scale one, Software and OpenGL show
  remaining differences in the editor code and status alignment, the Docking
  viewport illustration and frame, and the boundary pixels of Split view.
  The direct native widget comparison is exact. Keep the assembled examples
  open until their corresponding regions also match.

## Terminal icon experiment (deferred)

This experiment is paused with TUI development.

The [CUI icon set][icons] uses Lucide glyphs in a bundled font. FPAS currently
passes CUI glyphs to GUI widgets. TUI buttons use explicit terminal glyphs or
text fallbacks. Its sidebar omits icons while showing labels and badges.

- [ ] Try a small semantic icon mapping to Unicode symbols and optional emoji
  for the TUI. Share the icon identifier across hosts and supply a text or
  ASCII fallback. Keep labels or accessible names on actionable icons. Check
  display, cell width, truncation, clipping, redraws, and alignment in Windows
  Terminal and at least one Linux terminal. The current
  [grapheme width code][glyphs] handles common wide characters, flags, and emoji
  presentation selectors with heuristics. Verify the actual terminal widths
  before choosing emoji defaults. See
  [Unicode emoji presentation][emoji] and [East Asian Width][width].

[icons]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/icons.md
[emoji]: https://www.unicode.org/reports/tr51/
[width]: https://www.unicode.org/reports/tr11/
[glyphs]: ../../src/Fpas.Ui.Tui/Internal/Rendering/Glyphs.fpas
