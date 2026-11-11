# CUI widget gaps

## Integration direction

Keep the GUI as thin as possible. CUI owns widget rendering, layout, animation,
input and interaction state. FPAS supplies the MVU model and messages; the
binding handles ABI conversion, borrowed data, stable identities, lifetime and
inspection. Compose existing CUI primitives before adding a native adapter.
Do not build a second GUI widget or layout engine. Keep unavoidable knowledge
of private CUI state and IDs inside the native bridge, not in C# or FPAS.

TUI development is paused until explicitly resumed by the user. Active work
targets the GUI only: widgets, layout, themes, examples and acceptance. Existing
TUI features, gallery and regression tests remain in place. Shared-code changes
must keep that implemented subset building and working. New GUI capabilities
must be rejected clearly by TUI preflight when unsupported; do not add terminal
implementations, parity work or gallery examples for them during this pause.
TUI annotations below describe existing coverage, not further implementation
requirements. TUI-specific proposals and experiments are deferred.

## Acceptance

Every CUI widget is in scope for FPAS GUI. New widgets target GUI only;
terminal support and suitability assessments are deferred. TUI coverage is
not an acceptance requirement. Use one canonical public constructor per widget
and keep existing shared APIs coherent. Do not retain alternate legacy widget
representations.

Use this inventory to rebuild the [CUI gallery][gallery] in FPAS. Check each
widget against its upstream demo and API. Accept it once its GUI API and behavior
are covered by tests, it is shown in the GUI gallery, and its relevant page is
captured. Only checked widgets have public FPAS widget implementations. Text,
layout and native primitives needed to compose accepted widgets remain
foundations; using a primitive does not accept its standalone widget API.

Show only checked widgets in the GUI gallery and use it for acceptance tests
of its visible pages and interactions. The GUI gallery is a clone of the pinned
native CUI gallery: match its pages, examples, labels,
icons, order, typography, colors, dimensions and spacing as closely as possible,
with pixel parity as the target at the same viewport, scale, theme and renderer.
Use native CUI widgets and layout rather than compensating with a second engine.
Implement that clone incrementally as widgets are accepted. Add supplementary
examples for every public CUI widget missing from the upstream gallery; absence
from that gallery does not remove a widget from scope. Capture the relevant
GUI pages when accepting a widget. The TUI gallery retains its implemented
subset; extending or capturing it is not part of new widget acceptance.
The sections group related work and leave implementation order to the gallery
work.

### Widget source layout

Give each new widget a dedicated source file for its FPAS interface and one
GUI implementation file in `Fpas.Ui.Gui/Internal/Widgets`. Match the unit name
to its location, such as `Fpas.Ui.Gui.Internal.Widgets.Sheet`. Add native code under
`src/Fpas.Ui.Gui.Cui/native/widgets` and managed interop only when existing
bindings cannot express the widget. A composition needs no dedicated C entry
point: theme switcher and theme toggle use the segment and icon-button
primitives. Shared dispatch, layout translation, focus and input stay with
their host. No new terminal implementation file is required.

## Reference

The catalog targets the [vendored CUI v1.4.0 release][vendor] at commit
[`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`][revision]. Compare each component
with that release before binding it.

The inventory combines the [core catalog][components], [extended
catalog][more], [public header][header], and both parts of the native gallery
([core][gallery], [extended][gallery-more]). It splits catalog pages that
describe multiple widgets, such as checkbox, switch, and radio. Options and
companion calls belong to their widget. Renderer, window,
font, and resource APIs are outside this inventory. Layout primitives,
axis formatting, and chart annotations are listed separately because the
gallery also needs them.

## Shared foundations

### Native window identity

- [ ] Expose the Linux application ID through GUI options, the managed window,
  and the native bridge into `cui_app_desc.app_id`. Match the installed
  `.desktop` filename and verify the resulting X11 `WM_CLASS`. The ID is read
  at window creation and is ignored on Windows. See [desktop identity][app-id].

### Theme compatibility

GUI and TUI already share [`Fpas.Ui.Themes`][fpas-themes]. The contract has
light and dark modes, eight CUI accents, semantic RGBA colors, custom palette
overrides, and a model-driven `ThemeFor` callback. GUI passes the palette to
native CUI. TUI derives its widget and focus styles from the same tokens and
composites alpha over the cell background. The [GUI gallery contract][gui-gallery-test]
checks theme switching with the native host. The [TUI contract][tui-contract]
checks semantic foregrounds, backgrounds and alpha composition.

That covers the shared palette required by the gallery. CUI also exposes
typography, control sizing, corner radii, shadows, reduced motion, and
system-theme behavior in its [theming reference][theming]. The shared FPAS
`Theme` contains appearance, accent, and colors. Track any additional theme
properties with the gallery widget that needs them. Theme switcher and theme
toggle propose a model change through existing CUI primitives without changing
the native palette themselves. System appearance is resolved by an application
provider; there is no automatic OS or terminal theme detection.

### Layout compatibility

The GUI passes the [shared box-layout subset][fpas-layout] to CUI's native
layout engine. The TUI reproduces those rules in cells. The subset covers
rows and columns, content/fill/fixed sizing, weighted growth,
shrinkage, size bounds, cross-axis alignment, gaps, padding, borders, and
vertical scrolling. GUI uses CUI logical points. TUI converts 8 logical units
to one column and 16 to one row. Both hosts expose responsive size classes.
The [native layout tests][gui-layout-test] check weighted and fixed allocations.
The [TUI contract][tui-contract] covers shrinkage, rounding, and logical
distances in cells. Graphical rows also support native wrapping via
`Flow.Wrap`, exercised by the chips in the [GUI gallery][gui-gallery-test].

The [CUI layout reference][layout] and [native gallery][gallery] also use
features outside that shared subset. The following layout work targets the GUI;
terminal equivalents are deferred.

- [ ] Responsive grid with a maximum column count and minimum column width.
- [ ] Main-axis justification, including space between children.
- [ ] Stack layout with layered children, per-axis sizing including percentages,
  and per-edge padding.
- [ ] Flexible spacer and fixed gap parity.
- [ ] Horizontal scrolling in the shared FPAS API and CUI-equivalent viewport
  control. The low-level native scroll binding supports both axes; shared
  `ScrollView` is vertical. Virtual list and split view have separate entries.
- [ ] Reset the gallery content scroll position when selecting a different page.
- [ ] Reading direction and logical start/end layout for right-to-left content.

### Native keyboard limits

Use the input contract of each component in the pinned release. CUI's
[documented pointer requirements][input-limits] cover rating, histogram-slider
handles, swipe actions, reorderable lists, JSON folding, agent-plan steps,
blame, time histograms, and the choices inside open time and color pickers.
The picker fields support keyboard opening and dismissal; their internal
choices have no keyboard focus. Describe and test the pinned release's input
behavior in each GUI widget's contract.

See [CUI upstream bugs](cui-upstream-bugs.md) for the toggle group's
Right-arrow reproduction and pending upstream fix.

### Terminal icon experiment (deferred)

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

## Widget inventory

### Text, icons, and display

- [ ] Text, including heading, label, secondary, muted, and inline code (`cui_text`, `cui_heading`, `cui_label`, `cui_secondary`, `cui_muted`, `cui_code`)
- [ ] Link (`cui_link`)
- [ ] Icon (`cui_icon`)
- [ ] Emoji display (`cui_emoji`)
- [ ] Keycap (`cui_kbd`)
- [ ] Badge (`cui_badge`)
- [ ] Avatar (`cui_avatar`)
- [ ] Avatar group (`cui_avatar_group`)
- [ ] Card (`cui_card`)
- [ ] Metric (`cui_metric`)
- [ ] Empty state (`cui_empty_state`)
- [ ] Image (`cui_image`)
- [ ] Interactive canvas (`cui_canvas`)

### Actions and selection

- [x] Button (`cui_button`): [API][fpas-button]. TUI: implemented; explicit terminal
  glyphs, static busy mark, one- or two-row sizes.
- [x] Icon button (`cui_icon_button`): [API][fpas-icon-button]. TUI: implemented;
  explicit terminal glyph or tooltip text.
- [x] Button group (`cui_button_group`): [API][fpas-button-group]. TUI: implemented;
  connected cell outline and dividers, independent member focus and messages.
- [x] Copy button (`cui_copy_button`): [API][fpas-copy-button]. GUI uses CUI's
  clipboard provider. TUI: implemented; OSC 52 requires terminal clipboard permission.
- [x] Hold-to-confirm button (`cui_hold_button`): [API][fpas-hold-button]. GUI
  fires at completion through native pointer, Space or Enter holds. TUI: implemented;
  confirm on pointer release after the same duration. Terminal input supplies no
  key release events, so keyboard holds cannot be reproduced.
- [x] Split button (`cui_split_button`): [API][fpas-split-button]. GUI delegates
  popup navigation to CUI. TUI: implemented; fitted cell popup, wheel/page-key scrolling.
- [x] Confirm button (`cui_confirm_button`): [API][fpas-confirm-button]. TUI:
  implemented; remaining seconds replace animated countdown faces, with pointer,
  Enter and typed Space activation.
- [x] Status button (`cui_status_button`): [API][fpas-status-button]. TUI: implemented;
  static busy mark, result symbols and semantic status colors replace animations.
- [x] Theme switcher (`cui_theme_switcher`): [API][fpas-theme-switcher]. GUI
  composes native segments. TUI: implemented; joined text segments preserve the
  controlled selection, focus, keyboard and application-provider contract.
- [x] Theme toggle (`cui_theme_toggle`): [API][fpas-theme-toggle]. GUI composes
  the native icon button. TUI: implemented; target caption with sun/moon symbol.
- [ ] Reactions (`cui_reactions`)
- [ ] Speed dial (`cui_speed_dial`)
- [ ] User menu (`cui_user_menu`)
- [ ] Swipe actions (`cui_swipe_item_begin`)
- [x] Segmented control (`cui_segmented`): [API][fpas-segmented]. GUI uses native
  CUI; Buttons contains the upstream view and compact date-range examples.
  TUI: implemented; joined cell segments, explicit glyphs or labels, Tab and
  non-wrapping Left/Right selection, with the model owning acceptance.
- [x] Toggle group (`cui_toggle_group`): [API][fpas-toggle-group]. The GUI
  Actions page shows the original independent formatting and exclusive
  alignment examples. TUI: deferred; graphical profile required.
- [x] Checkbox (`cui_checkbox`): [API][fpas-selection]. Selection shows the
  original notification and disabled examples. TUI: graphical profile required.
- [x] Switch (`cui_switch`): [API][fpas-selection]. Selection shows Wi-Fi,
  Bluetooth and airplane mode. TUI: graphical profile required.
- [x] Radio button (`cui_radio`): [API][fpas-selection]. The native rows
  accept a controlled value, caption and description. TUI: graphical profile required.
- [x] Radio group (`cui_radio_group`): [API][fpas-selection]. Selection
  composes native radio rows for the original save-frequency choices.
  TUI: graphical profile required.
- [x] Radio cards (`cui_radio_cards`): [API][fpas-radio-cards]. Selection shows
  the original save destinations and tests pointer selection and arrow-key focus.
  TUI: graphical profile required.
- [x] Chip (`cui_chip`): [API][fpas-chips-rating]. Selection exercises six
  independent chips with pointer and keyboard input and native row wrapping.
  TUI: graphical profile required.
- [x] Rating (`cui_rating`): [API][fpas-chips-rating]. Selection shows five
  stars, starting at four, and tests clicking a star to select or clear it.
  TUI: graphical profile required.
- [ ] Control tile (`cui_control_tile`)

### Inputs and pickers

- [ ] Text field, including password mode, prefix, and suffix (`cui_input`)
- [ ] Search field, including prefix and suffix (`cui_search`)
- [ ] Text area, including prefix and suffix (`cui_textarea`)
- [ ] Number field, including suffix (`cui_number`)
- [ ] Inline edit (`cui_inline_edit`)
- [ ] Scrub field (`cui_scrub`)
- [ ] Money field (`cui_money_input`)
- [ ] Shortcut recorder and shortcut label (`cui_shortcut_recorder`, `cui_shortcut_label`)
- [ ] Tag input (`cui_tag_input`)
- [ ] One-time code input (`cui_otp_input`)
- [ ] Password strength meter (`cui_password_strength`)
- [ ] Select, including the searchable combobox (`cui_select`)
- [ ] Multi-select (`cui_multi_select`)
- [ ] Calendar (`cui_calendar`)
- [ ] Date picker (`cui_date_picker`)
- [ ] Date range calendar (`cui_range_calendar`)
- [ ] Date range picker (`cui_date_range_picker`)
- [ ] Time picker (`cui_time_picker`)
- [ ] Time dial (`cui_time_dial`)
- [ ] Wheel picker (`cui_wheel_picker`)
- [ ] Color picker (`cui_color_picker`)
- [ ] Emoji picker (`cui_emoji_picker`)
- [x] Slider (`cui_slider`): [API][fpas-sliders]. Selection shows continuous
  Volume and stepped Brightness with percentage display and keyboard adjustment.
  TUI: graphical profile required.
- [x] Range slider (`cui_range_slider`): [API][fpas-sliders]. Selection shows
  Price range and tests dragging both endpoints. Pinned CUI adjusts by pointer.
  TUI: graphical profile required.
- [ ] Ruler slider (`cui_ruler_slider`)
- [ ] Histogram slider (`cui_histogram_slider`)
- [ ] Knob (`cui_knob`)
- [ ] Fader (`cui_fader`)
- [ ] Level meter (`cui_level_meter`)
- [ ] Dropzone (`cui_dropzone`)
- [ ] Avatar upload (`cui_avatar_upload`)
- [ ] Signature pad (`cui_signature_pad`)

### Navigation and app chrome

- [x] Sidebar navigation (`cui_nav_item`, `cui_nav_section`): [API][sidebar].
  TUI: implemented; top-aligned cell rows, labels and badges. Icons are omitted.
  [GUI][gui-gallery-test] and [TUI][tui-gallery-test] gallery tests
  cover destinations and the responsive drawer.
- [x] Tabs (`cui_tabs`): [API][fpas-navigation-controls]. Navigation shows the
  original underline and icon pill strips with model-selected content.
  TUI: graphical profile required.
- [x] Breadcrumb (`cui_breadcrumb`): [API][fpas-navigation-controls]. Navigation
  exercises ancestor activation and shows the selected caption in a native toast.
  TUI: graphical profile required.
- [x] Pagination (`cui_pagination`): [API][fpas-navigation-controls]. Navigation
  checks the original twelve-page pager, page boundaries and compact layout.
  TUI: graphical profile required.
- [x] Stepper (`cui_stepper`): [API][fpas-stepper]. Navigation shows the native
  workflow in desktop and compact layouts and exercises Back, Next and Finish.
  Finish shows the native completion toast. TUI: graphical profile required.
- [ ] Table of contents and anchors (`cui_toc`, `cui_toc_anchor`)
- [ ] Dock and dock items (`cui_dock`, `cui_dock_item`, `cui_dock_separator`)
- [ ] Bottom tab bar (`cui_tab_bar`)
- [ ] Menu bar and menus (`cui_menu_bar`, `cui_menu_bar_menu`)
- [ ] Toolbar and toolbar buttons (`cui_toolbar`, `cui_toolbar_button`, `cui_toolbar_spacer`, `cui_toolbar_separator`)
- [ ] Document tabs (`cui_doc_tabs`)
- [ ] Status bar and its items (`cui_status_bar`, `cui_status_bar_text`, `cui_status_bar_item`, `cui_status_bar_progress`, `cui_status_bar_spacer`)
- [ ] Docking workspace and panels (`cui_dockspace`, `cui_dock_panel`)
- [ ] Split view (`cui_split`)

### Overlays and disclosure

- [x] Dropdown menu and its items (`cui_menu`, `cui_menu_item`, `cui_menu_label`,
  `cui_menu_separator`): [API][fpas-menu]. Both galleries show the original
  Options and More actions examples. GUI uses native CUI. TUI: implemented;
  shared split-button popup, fitted cell placement, keyboard selection,
  command states, shortcut hints, sections and separators.
- [x] Context menu (`cui_context_menu`): [API][fpas-context-menu]. GUI attaches
  native CUI menus to existing target components and shares dropdown commands,
  conversion and inspection. The Overlays gallery shows Cut/Copy/Paste/Delete.
  TUI: deferred; graphical profile required.
- [x] Popover (`cui_popover`): [API][fpas-popovers]. Overlays shows the
  original Filters panel. Windows Software and OpenGL cover placement, input,
  retained values and dismissal. TUI: deferred; graphical profile required.
- [x] Tooltip (`cui_tooltip`): [API][fpas-tooltip]. Overlays reproduces the
  original icon toolbar and its shortcut hints. Windows Software and OpenGL
  cover cold hover, warm handoff and anchor input. TUI: deferred; graphical
  profile required.
- [ ] Dialog and action footer (`cui_dialog`, `cui_dialog_footer`)
- [x] Sheet/Drawer (`cui_sheet`): [API][fpas-sheet]. TUI: implemented; cell placement
  replaces CUI's slide transition, retaining modal focus, scrolling and dismissal.
- [ ] Command palette (`cui_command_palette`)
- [ ] Accordion (`cui_accordion`)
- [ ] Hover card (`cui_hover_card`)
- [x] Confirm popover (`cui_confirm_popover`): [API][fpas-popovers].
  Disclosure shows the original Delete branch question. Windows Software and
  OpenGL cover separate open and confirm actions and cancellation. TUI:
  deferred; graphical profile required.
- [ ] Expandable text (`cui_expandable_text`)
- [ ] Expandable card (`cui_expandable_card`)
- [ ] Guided tour (`cui_tour`)

### Feedback and progress

- [x] Toast, including timed undo toast (`cui_toast`, `cui_toast_action_clicked`):
  [API][fpas-toast]. Overlays shows the four native examples and a timed Undo.
  Navigation uses toasts for breadcrumb activation and Finish. The gallery
  checks stacking, hover pause, expiry, dismissal and action delivery.
  TUI: graphical host command required.
- [ ] Alert (`cui_alert`)
- [ ] Progress bar (`cui_progress`)
- [ ] Meter (`cui_meter`)
- [ ] Spinner (`cui_spinner`)
- [ ] Skeleton (`cui_skeleton`)
- [ ] Progress ring (`cui_progress_ring`)
- [ ] Timeline item (`cui_timeline_item`)
- [ ] Banner (`cui_banner`)
- [ ] Countdown (`cui_countdown`)
- [ ] Onboarding checklist (`cui_checklist`)

### Lists, tables, and files

- [ ] List item (`cui_list_item`)
- [ ] Tree (`cui_tree_begin`, `cui_tree_clicked`)
- [ ] Virtual list (`cui_virtual_list`)
- [ ] Reorderable list and items (`cui_reorder_begin`, `cui_reorder_item`)
- [ ] Table, rows, and cells (`cui_table`, `cui_table_row`, `cui_table_cell`, `cui_table_cell_text`)
- [ ] Data grid, including editable cells (`cui_data_grid_begin`, `cui_data_grid_row`, `cui_data_grid_cell`, `cui_data_grid_cell_text`, `cui_data_grid_edit`, `cui_data_grid_cell_edit`)
- [ ] Filter bar (`cui_filter_bar`)
- [ ] Tree table (`cui_tree_table_begin`, `cui_tree_table_row`, `cui_tree_table_name`, `cui_tree_cell`)
- [ ] Path bar and its dropdown (`cui_path_bar`, `cui_path_bar_menu_begin`)
- [ ] File browser (`cui_file_browser`)
- [ ] File picker (`cui_file_picker`)

### Code and tool panels

- [ ] Code block (`cui_code_block`)
- [ ] JSON viewer (`cui_json_view`)
- [ ] Markdown view (`cui_markdown`)
- [ ] Code editor (`cui_code_editor`)
- [ ] Diff viewer (`cui_diff_view`)
- [ ] Property inspector and editors for float, integer, text, boolean, enum,
  color, and vectors (`cui_inspector`, `cui_inspector_group`, `cui_prop_float`,
  `cui_prop_int`, `cui_prop_text`, `cui_prop_bool`, `cui_prop_enum`,
  `cui_prop_color`, `cui_prop_vec2`, `cui_prop_vec3`, `cui_prop_vec4`)
- [ ] Log console (`cui_log_view`)
- [ ] Terminal widget (`cui_terminal`)
- [ ] Hex viewer (`cui_hex_view`)
- [ ] Flame graph (`cui_flame_graph`)
- [ ] Curve editor (`cui_curve_editor`)
- [ ] Gradient editor (`cui_gradient_editor`)
- [ ] Timeline editor (`cui_timeline_editor`)

### Developer tools

- [ ] Key-value editor (`cui_kv_editor`)
- [ ] Problems panel (`cui_problems`)
- [ ] Pipeline (`cui_pipeline`)
- [ ] Query builder (`cui_query_builder`)
- [ ] Schedule picker (`cui_schedule_picker`)
- [ ] Keymap (`cui_keymap`)
- [ ] Commit graph (`cui_commit_graph`)
- [ ] Dependency graph (`cui_graph_view`)
- [ ] Network waterfall (`cui_waterfall`)
- [ ] Trace viewer (`cui_trace_view`)
- [ ] Regex tester (`cui_regex_tester`)
- [ ] Test explorer (`cui_test_explorer`)
- [ ] Stack trace (`cui_stack_trace`)
- [ ] HTTP request bar (`cui_request_bar`)
- [ ] HTTP response status (`cui_response_status`)
- [ ] Uptime bars (`cui_uptime`)
- [ ] Time histogram (`cui_time_histogram`)
- [ ] Coverage (`cui_coverage`)
- [ ] Blame (`cui_blame`)
- [ ] Feature flags (`cui_feature_flags`)
- [ ] API explorer (`cui_api_explorer`)

### Charts and instruments

- [ ] Line chart, including area mode (`cui_line_chart`)
- [ ] Bar chart, including stacked mode (`cui_bar_chart`)
- [ ] Donut chart (`cui_donut_chart`)
- [ ] Sparkline (`cui_sparkline`)
- [ ] Gauge (`cui_gauge`)
- [ ] Heatmap (`cui_heatmap`)
- [ ] Calendar heatmap (`cui_calendar_heatmap`)
- [ ] Treemap (`cui_treemap`)
- [ ] Radar chart (`cui_radar_chart`)
- [ ] Funnel (`cui_funnel`)
- [ ] Split bar (`cui_split_bar`)
- [ ] Waffle (`cui_waffle`)
- [ ] Sankey (`cui_sankey`)
- [ ] Bar list (`cui_bar_list`)
- [ ] Activity rings (`cui_activity_rings`)
- [ ] Scatter chart, including bubble mode (`cui_scatter_chart`)
- [ ] Radial gauge (`cui_radial_gauge`)
- [ ] Candlestick chart (`cui_candlestick`)
- [ ] Dot-matrix chart (`cui_dot_chart`)
- [ ] Event track (`cui_event_track`)
- [ ] Readout (`cui_readout`)
- [ ] Dot-matrix display (`cui_dot_display`)

Chart annotations (`cui_chart_annotation`) are an open feature of the dot-matrix,
line, and bar charts. Verify thresholds, points, labels, and label collision
handling with those demos.

Include axis formatting through `cui_axis_units`, including `CUI_AXIS_HOURS`:
12.25 reads 12:15 and 30 reads 06:00 on the next day. Match these semantics
in the GUI chart bindings. For right-to-left views, readout and dot-display
placement follow reading direction while chart and event-track time axes remain left to
right. Arabic-script labels require CUI's optional static-text shaping support;
track that native build option with the GUI RTL layout work.

### Conversation and media

- [ ] Chat message (`cui_chat_message`)
- [ ] Chat divider (`cui_chat_divider`)
- [ ] Typing indicator (`cui_typing_indicator`)
- [ ] Comment (`cui_comment`)
- [ ] Presence (`cui_presence`)
- [ ] Counter (`cui_counter`)
- [ ] Ticker (`cui_ticker`)
- [ ] Carousel and slides (`cui_carousel`, `cui_carousel_slide`)
- [ ] Image compare (`cui_image_compare`)
- [ ] Card stack and items (`cui_card_stack_begin`, `cui_card_stack_item_begin`)
- [ ] Image viewer (`cui_image_viewer`)
- [ ] Image cropper (`cui_cropper`)
- [ ] Now playing (`cui_now_playing`)
- [ ] Waveform (`cui_waveform`)

### Planning and diagram editors

- [ ] Kanban board, columns, and cards (`cui_kanban_begin`, `cui_kanban_column_begin`, `cui_kanban_card_begin`)
- [ ] Week calendar (`cui_week_calendar`)
- [ ] Month calendar (`cui_month_calendar`)
- [ ] Gantt chart (`cui_gantt`)
- [ ] Node editor with pins, links, and node content (`cui_node_editor_begin`,
  `cui_node_begin`, `cui_node_input`, `cui_node_output`,
  `cui_node_content_begin`, `cui_node_link`)

### Motion

- [ ] Shimmer text (`cui_shimmer_text`)
- [ ] Typewriter text (`cui_typewriter`)
- [ ] Text reveal (`cui_text_reveal`)
- [ ] Rotating words (`cui_word_rotate`)
- [ ] Rolling number (`cui_number_roll`)
- [ ] Gradient text (`cui_gradient_text`)
- [ ] Spotlight card (`cui_spotlight`)
- [ ] Loader and its motion variants (`cui_loader`)

### AI interfaces

- [ ] Reasoning panel (`cui_reasoning_begin`)
- [ ] Tool call (`cui_tool_call`)
- [ ] Agent plan (`cui_agent_plan`)
- [ ] Agent status (`cui_agent_status`)
- [ ] Source list (`cui_source_list`)
- [ ] Citation (`cui_citation`)
- [ ] Composer (`cui_chat_composer`)
- [ ] Model picker (`cui_model_picker`)
- [ ] Prompt suggestions (`cui_prompt_suggestions`)
- [ ] Response actions (`cui_response_actions`)
- [ ] Context meter (`cui_context_meter`)
- [ ] Voice orb (`cui_voice_orb`)

[revision]: https://github.com/yappologistic/CUI/tree/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5
[components]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/docs/components.md
[more]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/docs/components-more.md
[header]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/include/cui/cui.h
[gallery]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/examples/gallery_ui.c
[gallery-more]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/examples/gallery_more.c
[theming]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/docs/theming.md
[layout]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/docs/layout.md
[icons]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/docs/icons.md
[app-id]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/docs/getting-started.md#your-apps-name-and-icon-on-linux
[input-limits]: https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/README.md#limitations
[emoji]: https://www.unicode.org/reports/tr51/
[width]: https://www.unicode.org/reports/tr11/
[vendor]: ../../third-party/cui/UPSTREAM.md
[fpas-themes]: ../../src/Fpas.Ui/Themes.fpas
[fpas-layout]: ../../src/Fpas.Ui/README.md#layout-and-style
[sidebar]: ../../src/Fpas.Ui/README.md#sidebar-navigation
[fpas-sheet]: ../../src/Fpas.Ui/README.md#sheet-and-drawer
[fpas-icon-button]: ../../src/Fpas.Ui/README.md#icon-button
[fpas-button]: ../../src/Fpas.Ui/README.md#button
[fpas-button-group]: ../../src/Fpas.Ui/README.md#button-group
[fpas-segmented]: ../../src/Fpas.Ui/README.md#segmented-control
[fpas-toggle-group]: ../../src/Fpas.Ui/README.md#toggle-group
[fpas-selection]: ../../src/Fpas.Ui/README.md#checkbox-switch-and-radio-controls
[fpas-radio-cards]: ../../src/Fpas.Ui/README.md#radio-cards
[fpas-sliders]: ../../src/Fpas.Ui/README.md#slider-and-range-slider
[fpas-chips-rating]: ../../src/Fpas.Ui/README.md#chip-and-rating
[fpas-navigation-controls]: ../../src/Fpas.Ui/README.md#tabs-breadcrumb-and-pagination
[fpas-stepper]: ../../src/Fpas.Ui/README.md#stepper
[fpas-toast]: ../../src/Fpas.Ui.Gui/README.md#toasts
[fpas-copy-button]: ../../src/Fpas.Ui/README.md#copy-button
[fpas-hold-button]: ../../src/Fpas.Ui/README.md#hold-to-confirm
[fpas-confirm-button]: ../../src/Fpas.Ui/README.md#confirm-button
[fpas-status-button]: ../../src/Fpas.Ui/README.md#status-button
[fpas-theme-switcher]: ../../src/Fpas.Ui/README.md#theme-switcher
[fpas-theme-toggle]: ../../src/Fpas.Ui/README.md#theme-toggle
[fpas-split-button]: ../../src/Fpas.Ui/README.md#split-button
[fpas-menu]: ../../src/Fpas.Ui/README.md#dropdown-menu
[fpas-context-menu]: ../../src/Fpas.Ui/README.md#context-menu
[fpas-popovers]: ../../src/Fpas.Ui/README.md#popover-and-confirmation-popover
[fpas-tooltip]: ../../src/Fpas.Ui/README.md#tooltip
[glyphs]: ../../src/Fpas.Ui.Tui/Internal/Rendering/Glyphs.fpas
[gui-gallery-test]: ../../tests/Fixtures/gui-host/PublishedGallery.fpas
[gui-layout-test]: ../../tests/Fpas.Ui.Gui.Cui.Tests/CuiHostLayoutTests.cs
[tui-gallery-test]: ../../tests/Fpas.IntegrationTests/Tui/TuiGalleryTests.cs
[tui-contract]: ../../tests/Fixtures/tui/CuiContract.fpas
