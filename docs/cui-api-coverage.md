# CUI API coverage

The [widget inventory](cui-widget-inventory.md) owns catalog acceptance. This
reference classifies the other public APIs in the [vendored CUI header](../third-party/cui/upstream/cui.h).
The binding exposes useful application capabilities, rather than mirroring every
C function or adding another spelling for an existing operation.

## Application capabilities

| Native capability | FPAS entry point |
| --- | --- |
| Window, placement, minimum/fixed dimensions, native title and size | `Gui.Options`, `Gui.Start`/`Run`, `Windows` commands |
| Native close decision, redraw and delayed wakeup | `OnCloseRequested`, `Closing.Cancel`, `Mvu.Exit`, `Frames.Redraw` |
| System appearance, palette, typography, geometry, elevation and motion | `StartThemed`/`RunThemed`, `Themes.Theme`, `ThemeMetrics.Options` |
| Font copying, role replacement, four direct fallback faces, emoji loading/support | `Fonts.Apply` and its typed operations |
| Native calendar/file-browser caption table | `Localization.Set`; native storage owns a copy |
| OKLCH conversion, OKLab mixing and alpha multiplication | `Fpas.Ui.Gui.Colors` |
| Drawing primitives, text measurement, image caching, clipping and native frame clock | `Canvases.Canvas` with callback-scoped `Painting.Surface` |
| Canvas click, hover/press changes and optional two-axis wheel ownership | `Canvas` callbacks; unchanged pointer states emit no message |
| Application shortcuts and editor navigation/find/selection | `BindingsFor`, `EditorCommands` |
| Native open/save/folder dialogs, clipboard, file drops and URL activation | GUI commands and existing file/link widgets |
| Deferred axis labels, native trace links/hover/geometry, avatars and captions | Existing widget options and `TraceGeometry` |

See the [GUI host](ui/gui/host.md) and
[shared widget reference](ui/widgets.md) for contracts. Font resources,
painting and native caption tables require the GUI host. The terminal host
retains its semantic palette and cell-based widget subset.

## Functions without a second application API

| Header family | Classification |
| --- | --- |
| `cui_create/destroy`, renderer/platform installation, viewport and frame begin/end, redraw scheduling, frame count | Host/backend lifecycle. CUI's app loop and the GUI session own it. Custom painting can read native time/delta; `Frames.Redraw` schedules work. |
| `cui_input_*`, dropped-file counts/drag state, cursor, mouse/key levels and `cui_wants_*` | Platform input and native ownership. Applications consume typed widget proposals, drop zones and application bindings. Test injection remains a low-level binding service. |
| `cui_make_id`, ID stack, last-ID/rect/hover queries, raw retained state and state destructors | Binding implementation. Stable application IDs and session snapshots provide the application contract. Raw native pointers are not exposed. |
| `cui_set_theme`, `cui_get_theme_mode`, `cui_set_accent`, `cui_get_accent`, token/color resolution and accent names | Covered by the model's theme, semantic colors and accent enum. No second imperative application theme state. |
| `cui_get_strings` | Native borrowed-pointer query. Applications retain their immutable caption data; no borrowed table escapes the binding. |
| `cui_font_load_file`, `cui_emoji_font_load_file` | File-loading conveniences. Use CLR file APIs and the single owned-memory loading path; emoji loading additionally accepts the platform system font. |
| `cui_icon_button_ex`, `cui_radio_group`, native theme switcher/toggle, dialog/sheet begin/end, grid cell editing/depth and flyout markers | Existing accepted widget/composition contracts. Native component composition and host scopes supply these operations; no parallel widget API. |
| `cui_popover_open/close/toggle/is_open`, toast dismissal, signature clear, log append, Markdown task application | Model-owned open state, collections, signatures and edited text already express these changes. Keep Update authoritative. |
| `cui_money_input_ex` | Its native amount is `double`. The single FPAS `MoneyFields.MoneyField` uses CUI text input and exact CLR `decimal` parsing/formatting. No binary amount crosses the native boundary. |
| `cui_textf`, `cui_table_cell_textf`, `cui_log_appendv`, `cui_terminal_printf` and C compound-literal convenience macros | C formatting/calling conveniences. FPAS supplies literal strings, typed options and model-owned data. Validated widget count formats retain only their documented integer slots. |
| `cui_tree_sort`, `cui_trace_sort`, `cui_flame_sort` | Model preparation. Use CLR collection sorting and the widget's documented ordering contract. |
| `cui_fs_list`, date conversion/today, key names, HTTP names, URL/path helpers and `cui_app_open_url` | Native widget helpers or ordinary CLR services. Existing file, date, shortcut, request and link widgets retain native behavior. No general-purpose duplicate standard library. |
| `cui_diff_compute/words/view_rows`, `cui_flame_span_rect`, `cui_curve_eval`, `cui_gradient_eval`, query matching/operator names, cron evaluation/description and native regex search | Widget algorithm/query helpers with no additional application consumer in this binding. Widgets use CUI's algorithms internally and return typed model data, including the schedule's next run. Standalone domain processing is outside the GUI API contract. |
| `cui_animate`, `cui_animate_spring` | CUI widget motion. Application model changes use MVU; custom drawing uses native frame time and explicit redraw requests. No parallel retained animation store. |
| Raw image/texture handles, render data, software/OpenGL renderer creation, upload, threads, damage, readback and native window handles | Renderer/backend plumbing. Widgets and custom painting own cached image sources; screenshots and renderer selection have host operations. The custom-paint API uses full-image UVs and uniform corner radii. |
| `cui_app_run`, software/backend status and raw screenshots/PNG writing | The existing GUI owner loop, renderer status and `CapturePng` provide these operations. |

## Numeric precision

Money uses CLR `decimal` for both model values and proposals, with zero through
twenty-eight fractional digits. CUI's text field supplies native editing and
presentation; amount parsing/formatting stays in the CLR and rejects implicit
rounding. Model values must fit the selected precision. No amount is converted
through `double`; application currency, rounding and storage policies remain
explicit. See the [money contract](ui/gui/smart-inputs/money-field.md).
