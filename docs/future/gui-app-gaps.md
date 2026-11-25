# Desktop GUI application gaps

Use the [GUI handbook](../ui/gui/desktop.md) for the supported API and the
[CUI widget inventory](../cui-widget-inventory.md) for gallery acceptance. Keep CUI responsible
for native rendering and input. Use FPAS MVU for application state, commands
and typed messages. The acceptance targets are Windows x64 and Linux x64.

See CUI's [Building an app][apps], [Recipes][recipes],
[Getting started][getting-started] and [Theming][theming] at the
[vendored revision](../../third-party/cui/UPSTREAM.md).

## Appearance and accessibility

Use the [font and native caption commands](../ui/gui/host.md#fonts-and-native-captions)
and shared theme metrics for platform acceptance.
Platform acceptance must cover a running animation and visible CJK glyphs with
an explicit fallback. Linux live appearance needs change notifications for an
idle window; CUI's X11 backend currently supplies a dark `GTK_THEME` hint.
Font coverage and bidirectional editing are separate concerns.
[CUI app options][app-options] · [CUI typography and motion][theming]

- [ ] Establish a screen-reader plan before treating the GUI as accessible for
  public apps. CUI has no accessibility tree. FPAS snapshots expose bounds,
  roles and values for inspection. A screen reader also needs platform
  semantics, focus events and actions. Start by validating a form, a modal and
  a list with a real screen reader. Track IME and complex-script editing
  separately. [CUI limitations][limitations] ·
  [FPAS platform limits](../ui/gui/desktop.md#publishing-and-platform-limits)

## Deferred window chrome

The [optional title bar](cui-titlebar.md) arrives upstream in two pull
requests. The vendored CUI revision lacks that API until both are merged.

[apps]: https://github.com/yappologistic/CUI/blob/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3/docs/apps.md
[recipes]: https://github.com/yappologistic/CUI/blob/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3/docs/recipes.md
[getting-started]: https://github.com/yappologistic/CUI/blob/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3/docs/getting-started.md
[theming]: https://github.com/yappologistic/CUI/blob/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3/docs/theming.md
[app-options]: https://github.com/yappologistic/CUI/blob/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3/docs/apps.md#two-ways-to-run
[limitations]: https://github.com/yappologistic/CUI/blob/f9597a30d3bba4f7ef3e0f918e31c28bdcd4d2a3/docs/faq.md#what-can-it-not-do
