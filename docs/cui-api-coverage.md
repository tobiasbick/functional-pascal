# CUI API coverage

The [widget inventory](cui-widget-inventory.md) owns catalog acceptance. This
reference classifies the other public APIs in the [vendored CUI header](../third-party/cui/upstream/cui.h).
The binding exposes useful application capabilities, rather than mirroring every
C function or adding another spelling for an existing operation.

## Application capabilities

| Native capability | FPAS entry point |
| --- | --- |
| Window, placement, minimum/fixed dimensions, native title and size | [`Gui.Options`, `Gui.Start`/`Run` and `Windows` commands](ui/gui/app-chrome/window.md) |
| `cui_dpi_scale`, `cui_window_focused`, `cui_file_drag_active`, `cui_dropped_file_count` | [`Session.InspectWindow()`](ui/gui/app-chrome/window.md#window-observation) reports native DPI, focus, file drag and unclaimed drop state. |
| Native close decision, `cui_request_redraw`, `cui_request_wakeup` | `OnCloseRequested`, `Closing.Cancel`, `Mvu.Exit`, [`Frames.Redraw`](ui/gui/host.md#invalidation-and-inspection) |
| System appearance, palette, typography, geometry, elevation and motion | `StartThemed`/`RunThemed`, `Themes.Theme`, `ThemeMetrics.Options` |
| Font copying, role replacement, four direct fallback faces, emoji loading and support queries | [`Fonts.Apply`](ui/gui/host.md#fonts-and-native-captions) loads emoji fonts from bytes, files, or the system. |
| Native calendar/file-browser caption table | `Localization.Set`; native storage owns a copy |
| OKLCH conversion, OKLab mixing and alpha multiplication | `Fpas.Ui.Gui.Colors` |
| Drawing primitives, independent corner radii, texture coordinates, text measurement, clipping and frame clock | [`Canvases.Canvas`](ui/gui/text-and-display/canvas.md) with callback-scoped [`Painting.Surface`](ui/gui/canvas-and-bars/painting.md) |
| `cui_animate`, `cui_animate_spring` | [`Painting.Surface.Animate` and `AnimateSpring`](ui/gui/canvas-and-bars/painting.md#spring-animation) sample a canvas-local value and request redraws. |
| Focusable canvas, ordered key/text/window-focus dispatch, hover cursor and visible focus | `Canvases.Options.Focusable`, `HoverCursor`, `OnInput`, `Mvu.Focus` and `Painting.Surface.FocusVisible()` |
| Pane-scoped Tab and Shift+Tab | [`Layout.Flow.Scope`](ui/gui/layout/focus-scopes.md) on graphical rows, columns and layers |
| `cui_disabled_scope` | [`Elements.DisabledScope`](ui/gui/layout/container-options.md#disabled-subtrees) inherits native input restrictions across an element tree. |
| `cui_box`, `cui_box_begin` and `cui_box_end` | [Containers](ui/gui/layout/container-options.md), [layered stacks](ui/gui/layout/stack.md) and [responsive grids](ui/gui/feedback/grid.md) share `Layout.Flow` and `Elements.Size`. |
| `cui_column`, `cui_row` | [`Elements.Column`](ui/gui/layout/column.md) and [`Elements.Row`](ui/gui/layout/row.md) arrange children. |
| `cui_gap`, `cui_spacer` | [`Elements.Gap` and `Elements.Spacer`](ui/gui/layout/spacing.md) add fixed or flexible space. |
| `cui_separator` | [`Elements.Divider()`](ui/gui/layout/divider.md) uses CUI's direction and theme border color. |
| `cui_viewport_size` | `CuiWindow.ViewportSize` supplies the logical drawable size for the graphical `View` and [`Snapshot().Size`](ui/gui/layout/layout-inspection.md#read-the-bounds). |
| `cui_split` | [`SplitViews.SplitView`](ui/gui/app-chrome/split-view.md) exposes the orientation, minimum pane sizes and divider proposals. |
| `cui_last_rect` and `cui_get_rect` | [`CuiWindow.LastRectangle`](ui/gui/cui.md#layout-and-control-inspection) reads the previous native layout. Find a control by its stable ID in [`Snapshot().Controls`](ui/gui/layout/layout-inspection.md#read-the-bounds) to read its finalized bounds. |
| `cui_scroll`, `cui_scroll_to`, `cui_scroll_reveal` | [`Elements.ScrollView`](ui/gui/layout/scroll-view.md) uses native viewport extents, padding, gap, edge fades and wheel routing. `Layout.ScrollPosition` controls the model-owned position. [`Scrolls.To`](ui/gui/layout/scroll-view.md#request-a-scroll-offset) moves a host-owned viewport. |
| `cui_table`, `cui_table_row`, `cui_table_cell`, `cui_table_cell_text` | [`Tables.Table`](ui/gui/data-display/table.md) renders sortable headings, selectable rows and literal or composed cells. |
| `cui_tree_begin`, `cui_tree_clicked`, `cui_tree_end` | [`Trees.Branch` and `Trees.Leaf`](ui/gui/data-display/tree.md) represent the hierarchy, expansion and activation. |
| `cui_reorder_begin`, `cui_reorder_item`, `cui_reorder_end` | [`ReorderableLists.ReorderableList`](ui/gui/data-display/reorderable-list.md) renders each row's children and proposes a permutation on drop. |
| `cui_accordion_begin`, `cui_accordion_end` | [`Accordions.Accordion`](ui/gui/data-display/accordion.md) renders each entry with model-owned expansion and a body. |
| `cui_virtual_list` | [`VirtualLists.VirtualList`](ui/gui/data-display/virtual-list.md) builds rows in CUI's requested viewport range. |
| `cui_line_chart`, `cui_bar_chart`, `cui_donut_chart`, `cui_pattern_meter`, `cui_gauge` | The [chart widgets](ui/gui/charts/README.md) render native charts and expose typed options. |
| Window and text reading direction | [`Desktop.Options`, `Layout.Flow`, and `Texts.Options`](ui/gui/layout/reading-direction.md) configure inheritance and overrides. |
| PNG and JPEG loading, rectangular texture updates | [`MediaImages.Source.File` and `MediaImages.Update`](ui/gui/text-and-display/image.md#usage) supply cached sources and immutable RGBA updates. |
| Canvas click, hover, press, and two-axis wheel ownership | `Canvas` callbacks. Use `Surface.AddWheelRegion` and `Surface.SetCursor` for custom interaction. [Canvas painting](ui/gui/canvas-and-bars/painting.md) |
| Application shortcuts and editor navigation/find/selection | [`BindingsFor`](ui/gui/host.md#application-shortcuts), [`EditorCommands`](ui/gui/host.md#code-editor-commands) |
| Native open/save/folder dialogs, clipboard, file drops and URL activation | GUI commands and existing file/link widgets |
| Inspect a popover, mark a flyout trigger, dismiss a toast or clear a signature during direct drawing | `CuiWindow.IsPopoverOpen`, `MarkFlyout`, `DismissToast` and `ClearSignature`. [Popover](ui/gui/overlays/popover.md) · [Toolbar](ui/gui/canvas-and-bars/toolbar.md) · [Toast](ui/gui/overlays/toast.md) · [Signature pad](ui/gui/capture/signature-pad.md) |
| Deferred axis labels, native trace links/hover/geometry, avatars and captions | Existing widget options and `TraceGeometry` |
| `cui_diff_view_rows`, `cui_flame_span_rect` | [`DiffGeometry.ReadRows`](ui/gui/code/diff-viewer.md#inspect-displayed-rows) and [`FlameGeometry.ReadSpanRectangle`](ui/gui/editing-tools/flame-graph.md#inspect-span-bounds) read the last presented view. |
| `cui_curve_eval`, `cui_gradient_eval`, `cui_query_match` | [`Curves.Evaluate`](ui/gui/editing-tools/curve-editor.md#sample-a-curve), [`Gradients.Evaluate`](ui/gui/editing-tools/gradient-editor.md#sample-a-gradient), and [`QueryBuilders.Matches`](ui/gui/developer-tools/query-builder.md#match-a-record) evaluate model data with CUI. |

See the [GUI host](ui/gui/host.md) and
[shared widget reference](ui/widgets.md) for contracts. Font resources,
painting and native caption tables require the GUI host. The terminal host
retains its semantic palette and cell-based widget subset.

## Functions without a second application API

| Header family | Classification |
| --- | --- |
| `cui_create/destroy`, renderer/platform installation, viewport setup and frame begin/end, redraw scheduling, frame count | Host/backend lifecycle. CUI's app loop and the GUI session own it. Custom painting can read native time/delta; `Frames.Redraw` schedules work. |
| `cui_time`, `cui_delta_time` | CUI supplies frame timing to [custom painting](ui/gui/canvas-and-bars/painting.md). Use the MVU session clock for background work such as the [Feedback upload](ui/gui/feedback/status-spinner.md#interaction). |
| `cui_input_*`, cursor, mouse/key levels and `cui_wants_*` | Platform input and native ownership. Applications consume typed widget proposals, drop zones and application bindings. Test injection remains a low-level binding service. |
| `cui_make_id`, ID stack, last-ID/rect/hover queries, raw retained state and state destructors | Binding implementation. Stable application IDs and session snapshots provide the application contract. Raw native pointers are not exposed. |
| `cui_set_theme`, `cui_get_theme_mode`, `cui_set_accent`, `cui_get_accent`, token/color resolution and accent names | Covered by the model's theme, semantic colors and accent enum. No second imperative application theme state. |
| `cui_get_strings` | Native borrowed-pointer query. Applications retain their immutable caption data; no borrowed table escapes the binding. |
| `cui_font_load_file`, borrowed font memory | Use CLR file APIs and the owned-memory font operation. Emoji file loading preserves CUI's on-demand bitmap reads. |
| `cui_icon_button_ex`, `cui_radio_group`, native theme switcher/toggle, dialog/sheet begin/end and grid cell editing/depth | Existing widget and composition contracts. Native component composition and host scopes supply these operations. |
| `cui_popover_open/close/toggle`, log append and Markdown task application | Update owns open state, log entries and edited text. |
| `cui_money_input_ex` | Its native amount is `double`. The single FPAS `MoneyFields.MoneyField` uses CUI text input and exact CLR `decimal` parsing/formatting. No binary amount crosses the native boundary. |
| `cui_textf`, `cui_table_cell_textf`, `cui_log_appendv`, `cui_terminal_printf` and C compound-literal convenience macros | C formatting/calling conveniences. FPAS supplies literal strings, typed options and model-owned data. Validated widget count formats retain only their documented integer slots. |
| `cui_tree_sort`, `cui_trace_sort`, `cui_flame_sort` | Model preparation. Use CLR collection sorting and the widget's documented ordering contract. |
| `cui_fs_list`, date conversion/today, key names, HTTP names, URL/path helpers and `cui_app_open_url` | Native widget helpers or ordinary CLR services. Existing file, date, shortcut, request and link widgets retain native behavior. No general-purpose duplicate standard library. |
| `cui_diff_compute`, `cui_diff_words`, query operator names, cron evaluation/description and native regex search | Widgets use CUI's algorithms internally and return typed model data, including the schedule's next run. |
| Raw image/texture handles, render data, software/OpenGL renderer creation, upload, threads, damage, readback and native window handles | Renderer/backend plumbing. Widgets and custom painting own cached image sources. The host exposes screenshots and renderer selection. |
| `cui_app_run`, software/backend status and raw screenshots/PNG writing | The existing GUI owner loop, renderer status and `CapturePng` provide these operations. |
| `cui_app_desc.custom_title_bar`, `cui_platform_iface.window_frame`/`.window_button`, `cui_title_bar_hit_test` | Deferred. Without the `cui_title_bar` component such a window has no caption, so the GUI host keeps the system frame. See the [CUI title bar](future/cui-titlebar.md). |

## Numeric precision

Money uses CLR `decimal` for both model values and proposals, with zero through
twenty-eight fractional digits. CUI's text field supplies native editing and
presentation; amount parsing/formatting stays in the CLR and rejects implicit
rounding. Model values must fit the selected precision. No amount is converted
through `double`; application currency, rounding and storage policies remain
explicit. See the [money contract](ui/gui/smart-inputs/money-field.md).
