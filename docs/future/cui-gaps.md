# CUI widget gaps

Every CUI widget should be available in FPAS GUI and TUI. Use the same public
FPAS API and match CUI's behavior in both hosts wherever the terminal permits
it. Match options, model state, events, keyboard navigation, focus, and pointer
interaction where applicable. The TUI can use cell-based equivalents for
graphics and motion. If a widget has no practical terminal equivalent,
record the reason for omitting it from TUI on its entry and keep it in the GUI
scope.

Breaking changes throughout the TUI are allowed, including its public API,
rendering, input, layout, and examples. A complete TUI rewrite is allowed if
that is the clearest way to reach parity. Prefer a shared widget contract and
backend-specific rendering and input handling.

Use this inventory to rebuild the [CUI gallery][gallery] in FPAS. Check each
widget against its upstream demo and API. Accept it once the API and behavior
are covered, it is shown in both FPAS galleries, and any TUI omission is
documented. A checkbox stays open even if FPAS already has an implementation.
The sections group related work and leave implementation order to the gallery
work.

### Widget source layout

Give each widget a dedicated source file for its shared FPAS interface and
one implementation file in each host's `Internal/Widgets` directory. Match
the unit name to its location, such as `Fpas.Ui.Gui.Internal.Widgets.Sheet`
and `Fpas.Ui.Tui.Internal.Widgets.Sheet`. Keep the native CUI adapters in
separate files under `src/Fpas.Ui.Gui.Cui/native/widgets` with matching
managed interop partials. Shared dispatch, layout, focus and input stay
with their host. Pass widget-specific settings and behavior through the
widget's module.

## Reference and counting

The catalog targets CUI commit
[`629b6fcb991d71527f2d67b25c51429c3dcdb498`][revision]. FPAS currently builds
against the [vendored CUI v1.3.1 release][vendor]. Compare the component with
that release before binding it. Track any required upstream upgrade with the
widget that needs it.

The inventory combines the [core catalog][components], [extended
catalog][more], [public header][header], and both parts of the native gallery
([core][gallery], [extended][gallery-more]). It splits catalog pages that
describe multiple widgets, such as checkbox, switch, and radio. The 213 widget
entries count options and companion calls with their widget. Renderer, window,
font, and resource APIs are outside the widget count. Layout primitives and
chart annotations are listed separately because the gallery also needs them.

## Shared foundations

### Theme compatibility

GUI and TUI already share [`Fpas.Ui.Themes`][fpas-themes]. The contract has
light and dark modes, eight CUI accents, semantic RGBA colors, custom palette
overrides, and a model-driven `ThemeFor` callback. GUI passes the palette to
native CUI. TUI derives its widget and focus styles from the same tokens and
composites alpha over the cell background. CUI's
[theme implementation][theme-source] is byte-identical to the vendored tag's
source at the catalog revision. The [portable contract][portable]
checks both modes and all accents, native palette transfer, a custom palette,
and terminal background colors. The [TUI contract][tui-contract] also checks
semantic foregrounds and alpha composition.

That covers the shared palette required by the gallery. CUI also exposes
typography, control sizing, corner radii, shadows, reduced motion, and
system-theme behavior in its [theming reference][theming]. The shared FPAS
`Theme` contains appearance, accent, and colors. Track any additional theme
properties with the gallery widget that needs them. Theme switcher and theme
toggle are widgets and have their own open entries.

### Layout compatibility

Both hosts implement the [shared box-layout subset][fpas-layout] of CUI.
It covers rows and columns, content/fill/fixed sizing, weighted growth,
shrinkage, size bounds, cross-axis alignment, gaps, padding, borders, and
vertical scrolling. GUI uses CUI logical points. TUI converts 8 logical units
to one column and 16 to one row. Both hosts expose responsive size classes.
The [portable contract][portable] checks weighted and fixed allocations in
both hosts. The [TUI contract][tui-contract] covers shrinkage, rounding, and
logical distances in cells.

The [CUI layout reference][layout] and [native gallery][gallery] also use
features outside that shared subset. Open layout work is tracked below.

- [ ] Responsive grid with a maximum column count and minimum column width.
- [ ] Wrapping rows and main-axis justification, including space between children.
- [ ] Stack layout with layered children, per-axis sizing including percentages,
  and per-edge padding.
- [ ] Flexible spacer, fixed gap, and orientation-aware separator parity.
- [ ] Horizontal scrolling and CUI-equivalent viewport control. Virtual list and
  split view have separate widget entries.
- [ ] Reset the gallery content scroll position when selecting a different page.
- [ ] Reading direction and logical start/end layout for right-to-left content.

### Terminal icon experiment

The [CUI icon set][icons] uses Lucide glyphs in a bundled font. FPAS currently
passes the CUI glyph to the GUI sidebar. Its TUI sidebar accepts the shared
icon option and omits the glyph while showing labels and badges.

- [ ] Try a small semantic icon mapping to Unicode symbols and optional emoji
  for the TUI. Share the icon identifier across hosts and supply a text or
  ASCII fallback. Keep labels or accessible names on actionable icons. Check
  display, cell width, truncation, clipping, redraws, and alignment in Windows
  Terminal and at least one Linux terminal. The current
  [grapheme width code][glyphs] handles common wide characters, flags, and emoji
  presentation selectors with heuristics. The [headless test][tui-headless]
  checks two-cell allocation and clipping for a flag emoji. Verify the actual
  terminal widths before choosing emoji defaults. See
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

- [x] Button (`cui_button`). Both galleries exercise the shared
  [button API][fpas-button] with six CUI variants, three sizes, icons on
  either side, busy and disabled input, selection, foreground color,
  tooltip and shortcut hint, and fixed or full width. TUI uses terminal
  glyphs, a static busy mark, and one- or two-row button sizes.
- [x] Icon button (`cui_icon_button`). Both galleries use the shared
  [icon button][fpas-icon-button] to open the compact drawer. The button
  supports a tooltip, shortcut hint, size, style, selected state, and
  disabled and busy states. TUI uses an explicit glyph or the tooltip text.
- [x] Button group (`cui_button_group`). Both galleries use the shared
  [button group][fpas-button-group] for independent formatting actions. GUI
  uses CUI's connected buttons and TUI draws a shared cell outline with
  dividers. Each enabled member retains its own focus stop and message.
- [x] Copy button (`cui_copy_button`). Both galleries copy their displayed
  command with the shared [copy button][fpas-copy-button]. The confirmation
  lasts 1.6 seconds and keeps the same focus identity. GUI uses CUI's clipboard
  provider and TUI sends OSC 52 to a terminal that allows clipboard writes.
- [ ] Hold-to-confirm button (`cui_hold_button`)
- [ ] Split button (`cui_split_button`)
- [ ] Confirm button (`cui_confirm_button`)
- [ ] Status button (`cui_status_button`)
- [ ] Theme switcher (`cui_theme_switcher`)
- [ ] Theme toggle (`cui_theme_toggle`)
- [ ] Reactions (`cui_reactions`)
- [ ] Speed dial (`cui_speed_dial`)
- [ ] User menu (`cui_user_menu`)
- [ ] Swipe actions (`cui_swipe_item_begin`)
- [ ] Segmented control (`cui_segmented`)
- [ ] Toggle group (`cui_toggle_group`)
- [ ] Checkbox (`cui_checkbox`)
- [ ] Switch (`cui_switch`)
- [ ] Radio button (`cui_radio`)
- [ ] Radio group (`cui_radio_group`)
- [ ] Radio cards (`cui_radio_cards`)
- [ ] Chip (`cui_chip`)
- [ ] Rating (`cui_rating`)
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
- [ ] Slider (`cui_slider`)
- [ ] Range slider (`cui_range_slider`)
- [ ] Ruler slider (`cui_ruler_slider`)
- [ ] Histogram slider (`cui_histogram_slider`)
- [ ] Knob (`cui_knob`)
- [ ] Fader (`cui_fader`)
- [ ] Level meter (`cui_level_meter`)
- [ ] Dropzone (`cui_dropzone`)
- [ ] Avatar upload (`cui_avatar_upload`)
- [ ] Signature pad (`cui_signature_pad`)

### Navigation and app chrome

- [x] Sidebar navigation (`cui_nav_item`, `cui_nav_section`). Both galleries
  use [`Fpas.Ui.Navigation.Sidebar`][sidebar]. The shared API covers sections,
  stable keys, model-owned active selection, badges, disabled entries, and
  focus independent of selection. The GUI uses native CUI rows and a gliding
  highlight. The terminal uses cell styles. The Windows tests cover gallery
  navigation in Software and OpenGL and compare the activation trace across
  the two hosts. Terminal icons are tracked under the icon experiment.
- [ ] Tabs (`cui_tabs`)
- [ ] Breadcrumb (`cui_breadcrumb`)
- [ ] Pagination (`cui_pagination`)
- [ ] Stepper (`cui_stepper`)
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

- [ ] Dropdown menu and its items (`cui_menu`, `cui_menu_item`, `cui_menu_label`, `cui_menu_separator`)
- [ ] Context menu (`cui_context_menu`)
- [ ] Popover (`cui_popover`)
- [ ] Tooltip (`cui_tooltip`)
- [ ] Dialog and action footer (`cui_dialog`, `cui_dialog_footer`)
- [x] Sheet/Drawer (`cui_sheet`). Both galleries use the shared
  [sheet widget][fpas-sheet] for sidebar navigation at narrow widths.
  Four edges, the pinned header, scrolling, modal focus, and dismissal are
  covered. GUI uses CUI's slide transition. TUI uses cell placement.
- [ ] Command palette (`cui_command_palette`)
- [ ] Accordion (`cui_accordion`)
- [ ] Hover card (`cui_hover_card`)
- [ ] Confirm popover (`cui_confirm_popover`)
- [ ] Expandable text (`cui_expandable_text`)
- [ ] Expandable card (`cui_expandable_card`)
- [ ] Guided tour (`cui_tour`)

### Feedback and progress

- [ ] Toast, including timed undo toast (`cui_toast`, `cui_toast_action_clicked`)
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

Chart annotations are an open feature of the dot-matrix, line, and bar charts.
Verify thresholds, points, labels, and label collision handling with those demos.

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

[revision]: https://github.com/yappologistic/CUI/tree/629b6fcb991d71527f2d67b25c51429c3dcdb498
[components]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/docs/components.md
[more]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/docs/components-more.md
[header]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/include/cui/cui.h
[gallery]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/examples/gallery_ui.c
[gallery-more]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/examples/gallery_more.c
[theming]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/docs/theming.md
[theme-source]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/src/cui_theme.c
[layout]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/docs/layout.md
[icons]: https://github.com/yappologistic/CUI/blob/629b6fcb991d71527f2d67b25c51429c3dcdb498/docs/icons.md
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
[fpas-copy-button]: ../../src/Fpas.Ui/README.md#copy-button
[glyphs]: ../../src/Fpas.Ui.Tui/Internal/Rendering/Glyphs.fpas
[portable]: ../../tests/Fixtures/gui-host/PortableContract.fpas
[tui-contract]: ../../tests/Fixtures/tui/CuiContract.fpas
[tui-headless]: ../../tests/Fixtures/tui/HeadlessContract.fpas
