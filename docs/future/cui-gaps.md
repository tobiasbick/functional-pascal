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
captured. Checkmarks record acceptance of the documented GUI API and examples.
Record any remaining native options with their widget. Public APIs and
composition primitives can exist while their gallery acceptance is open.

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
bindings cannot express the widget. Native widgets with identical begin/end
contracts may share their traversal adapter while keeping separate public units.
A composition needs no dedicated C entry
point: theme switcher and theme toggle use the segment and icon-button
primitives. Shared dispatch, layout translation, focus and input stay with
their host. No new terminal implementation file is required.

## Reference

The catalog targets the [vendored CUI snapshot][vendor] at commit
[`c8baf35008807d66efe8e766c8964122227a2f80`][revision]. Compare each component
with that snapshot before binding it.

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
distances in cells. Graphical rows support native wrapping via `Flow.Wrap`,
exercised by the chips in the [GUI gallery][gui-gallery-test]. Reader uses
`ScrollView` with native edge fades. Selecting a different page starts at the
top, including when returning to a previously scrolled page.

The [CUI layout reference][layout] and [native gallery][gallery] also use
features outside that shared subset. The following layout work targets the GUI;
terminal equivalents are deferred.

- [ ] Stack layout with layered children, independent width/height constraints
  and per-edge padding. Native percentages are available through
  `Layout.Extent.Percent` for main-axis sizing, button width and skeleton width.
- [ ] Flexible spacer and fixed gap parity.
- [ ] Horizontal scrolling in the shared FPAS API and CUI-equivalent viewport
  control. The low-level native scroll binding supports both axes; shared
  `ScrollView` is vertical. Virtual list and split view have separate entries.
- [ ] Reset the gallery content scroll position when reselecting the active
  destination. The native gallery also resets in this case.
- [ ] Reading direction and logical start/end layout for right-to-left content.

### Native keyboard limits

Use the input contract of each component in the pinned release. CUI's
[documented pointer requirements][input-limits] cover rating, histogram-slider
handles, swipe actions, reorderable lists, JSON folding, agent-plan steps,
blame, time histograms, and the choices inside open time and color pickers.
The picker fields support keyboard opening and dismissal; their internal
choices have no keyboard focus. Describe and test the pinned release's input
behavior in each GUI widget's contract.

See [CUI upstream bugs](cui-upstream-bugs.md) for open defects referenced
by the FPAS adapters.

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

- [ ] Text, including heading, label, secondary, muted, and inline code
  (`cui_text`, `cui_heading`, `cui_label`, `cui_secondary`, `cui_muted`, `cui_code`).
  `Elements.Text` and semantic styles already supply text, headings, labels
  and secondary and muted text in the gallery. Inline code and standalone
  text acceptance remain open.
- [x] Link (`cui_link`). [API][fpas-hover-card]. Disclosure verifies pointer
  and keyboard activation of the original @jonas link and its visible focus
  ring. The bridge applies the workaround for [CUI-012](cui-upstream-bugs.md#cui-012).
  The graphical profile is required.
- [ ] Icon (`cui_icon`). GUI widgets accept CUI icon glyphs. Standalone icons
  still need a public API and gallery acceptance.
- [ ] Emoji display (`cui_emoji`)
- [ ] Keycap (`cui_kbd`)
- [x] Badge (`cui_badge`). [API][fpas-surfaces]. Overview shows the original
  team roles. Feedback shows neutral, tinted, dotted and solid badges.
  The graphical profile is required.
- [x] Avatar (`cui_avatar`). [API][fpas-hover-card]. The original Jonas
  Berg card shows initials and the online dot. The native test verifies image
  replacement and clearing. Avatar and Avatar upload share CUI image loading
  and texture lifetime. The graphical profile is required.
- [x] Avatar group (`cui_avatar_group`). [API][fpas-surfaces]. Feedback
  reproduces the seven-name group with four initials and a +3 overflow badge.
  The graphical profile is required.
- [x] Card (`cui_card`). [API][fpas-surfaces]. Overview and Feedback use native
  cards in responsive grids. The gallery checks equal row heights and reflow
  after resizing. Feedback uses CUI's Around justification for the Goals rings.
  The graphical profile is required.
- [x] Metric (`cui_metric`). [API][fpas-surfaces]. Overview shows the original
  three tiles, including the Frame time trend and sparkline. The graphical
  profile is required.
- [x] Empty state (`cui_empty_state`). [API][fpas-surfaces]. Feedback shows
  No projects yet. The gallery activates New project and verifies the original
  success toast. The graphical profile is required.
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
- [x] Reactions (`cui_reactions`): [API][fpas-reactions]. GUI delegates the
  count animation and interaction to CUI and returns the changed item to the model.
- [x] Speed dial (`cui_speed_dial`): [API][fpas-speed-dial]. Actions shows the
  original Create dial. CUI owns the floating layout and animation. Gallery
  tests cover opening, choosing an action, Escape and focus return.
- [x] User menu (`cui_user_menu`): [API][fpas-user-menu]. GUI uses CUI's native
  account trigger and popup with shared menu commands and focus handling.
  The original example uses initials. The native avatar image option is open.
  Reuse the image loading and lifetime handling from Avatar.
- [x] Swipe actions (`cui_swipe_item_begin`): [API][fpas-swipe-actions]. Actions
  shows the three original task rows and the resulting toasts. The tests
  exercise mouse dragging and the Windows horizontal wheel path used by
  touchpads inside a vertical scroll view.
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

- [x] Text field, including password mode, prefix, and suffix (`cui_input`):
  [API][fpas-text-field]. Inputs shows the responsive text fields. Smart inputs
  shows the original prefix and suffix examples. Windows Software and OpenGL
  cover Unicode editing, submission, read-only input and password reveal.
  TUI: deferred; graphical profile required.
- [x] Search field, including prefix and suffix (`cui_search`):
  [API][fpas-search-field]. Inputs shows the original project search beside
  Password. Windows Software and OpenGL cover query editing, the native clear
  button and focus restoration, and submission with displayed affixes.
  TUI: deferred; graphical profile required.
- [x] Text area, including prefix and suffix (`cui_textarea`):
  [API][fpas-text-area]. Inputs shows the full-width About field with its hint.
  Windows Software and OpenGL cover LF insertion, Unicode editing, visual-line
  movement, Ctrl+Enter submission with displayed affixes, and wheel routing
  inside an enclosing viewport. Wheel-routing workaround:
  [CUI-001](cui-upstream-bugs.md#cui-001). TUI: deferred; graphical profile required.
- [x] Number field, including suffix (`cui_number`): [API][fpas-number-field].
  Inputs shows Seats with model-owned bounds, native step buttons and editable
  numeric text. Draft and finite-value workaround:
  [CUI-005](cui-upstream-bugs.md#cui-005). TUI: deferred; graphical profile required.
- [ ] Inline edit (`cui_inline_edit`)
- [ ] Scrub field (`cui_scrub`)
- [ ] Money field (`cui_money_input`)
- [ ] Shortcut recorder and shortcut label (`cui_shortcut_recorder`, `cui_shortcut_label`)
- [x] Tag input (`cui_tag_input`): [API][fpas-tag-input]. Inputs shows Topics
  with native tag entry and removal. Batched tag-limit workaround:
  [CUI-002](cui-upstream-bugs.md#cui-002). TUI: deferred; graphical profile required.
- [x] One-time code input (`cui_otp_input`): [API][fpas-one-time-code]. Inputs
  shows Verification code with change and completion messages. Replacement
  completion workaround: [CUI-003](cui-upstream-bugs.md#cui-003).
  TUI: deferred; graphical profile required.
- [x] Password strength meter (`cui_password_strength`): [API][fpas-password-strength].
  Inputs shows CUI's native score and label for New password.
  TUI: deferred; graphical profile required.
- [x] Select, including searchable combobox (`cui_select`): [API][fpas-select].
  Inputs shows Country and searchable Language choices with stable option
  indices. Search-dismissal focus workaround:
  [CUI-004](cui-upstream-bugs.md#cui-004). TUI: deferred; graphical profile required.
- [x] Multi-select (`cui_multi_select`): [API][fpas-multi-select]. Pickers
  shows the original searchable Languages example. Gallery tests cover
  filtering, controlled selection, and dismissal with focus return.
- [x] Calendar (`cui_calendar`): [API][fpas-calendar]. The supplementary
  Inputs example shares the Due date selection. The gallery verifies
  day selection and browsing across a year boundary. Its Windows capture
  matches a standalone native CUI calendar.
- [x] Date picker (`cui_date_picker`): [API][fpas-calendar]. Inputs shows the
  original Due date field. Windows comparison covers the field and popup.
  The gallery checks selection, an empty value, dismissal and focus return.
  Focus ring workaround: [CUI-012](cui-upstream-bugs.md#cui-012).
- [x] Date range calendar (`cui_range_calendar`): [API][fpas-date-range].
  Pickers shows the original inline two-month calendar, linked to Stay dates.
  The gallery verifies month navigation and selection across a year boundary.
- [x] Date range picker (`cui_date_range_picker`): [API][fpas-date-range].
  Pickers shows the original fields with and without presets. The gallery
  verifies presets, shared partial selections, Escape, and focus return.
  The Pickers content matched the CUI reference on Windows in all three captures.
- [x] Time picker (`cui_time_picker`): [API][fpas-time-controls]. Pickers shows
  the original 24-hour and 12-hour examples. Windows captures match the
  reference with both fields closed and each popup open. The gallery checks
  minute steps, AM/PM, Escape and focus return.
- [x] Time dial (`cui_time_dial`): [API][fpas-time-controls]. The original
  clock face is captured in Pickers. The gallery verifies hour and minute
  selection and keyboard activation of the hour selector.
- [x] Wheel picker (`cui_wheel_picker`): [API][fpas-time-controls]. Pickers
  shows the original three joined date wheels. The gallery verifies keyboard
  wrapping, wheel input, dragging and visible captions on keyboard focus.
  Native workarounds: [CUI-009](cui-upstream-bugs.md#cui-009) and
  [CUI-010](cui-upstream-bugs.md#cui-010).
- [x] Color picker (`cui_color_picker`): [API][fpas-color-emoji]. Inputs shows
  the original Brand color field. The gallery verifies presets, hue editing
  after a model change, alpha preservation, and keyboard focus and dismissal.
  Windows captures match within one color level per channel. Native
  workarounds: [CUI-012](cui-upstream-bugs.md#cui-012) and
  [CUI-013](cui-upstream-bugs.md#cui-013).
- [x] Emoji picker (`cui_emoji_picker`): [API][fpas-color-emoji]. Smart inputs
  shows the original message field and popup. The gallery verifies group
  navigation, search, insertion, and persistence of recent emoji and skin
  tone. The accepted Smart inputs sections match the Windows reference
  with the popup closed and open after the glyphs finish loading.
- [x] Slider (`cui_slider`): [API][fpas-sliders]. Selection shows continuous
  Volume and stepped Brightness with percentage display and keyboard adjustment.
  TUI: graphical profile required.
- [x] Range slider (`cui_range_slider`): [API][fpas-sliders]. Selection shows
  Price range and tests pointer and keyboard adjustment of both endpoints.
  TUI: graphical profile required.
- [x] Ruler slider (`cui_ruler_slider`): [API][fpas-ruler-histogram]. Pickers
  shows the original Height and Temperature tapes. The gallery verifies
  dragging, both wheel axes, Shift+arrow and visible ticks with keyboard focus.
  Native workarounds: [CUI-009](cui-upstream-bugs.md#cui-009),
  [CUI-010](cui-upstream-bugs.md#cui-010) and [CUI-011](cui-upstream-bugs.md#cui-011).
- [x] Histogram slider (`cui_histogram_slider`): [API][fpas-ruler-histogram].
  Pickers shows the original 24-bin Price example and verifies both endpoints.
  Windows captures of the complete page at rest and after price adjustment
  match the CUI content within one level per color channel.
- [x] Knob (`cui_knob`): [API][fpas-audio]. Audio shows the original Cutoff,
  Resonance, Attack and Pan. The gallery verifies logarithmic reset and drag.
  TUI: graphical profile required.
- [x] Fader (`cui_fader`): [API][fpas-audio]. Audio shows the six original
  mixer strips with stereo peaks and the Kick scale. The gallery verifies
  the focused wheel and keyboard input. TUI: graphical profile required.
- [x] Level meter (`cui_level_meter`): [API][fpas-audio]. Audio shows the
  segmented stereo master, and the native test checks clip clearing.
  The accepted Audio content matches CUI pixel for pixel in Windows software
  captures at 1201 and 900 points with the same signal phase.
  TUI: graphical profile required.
- [x] Dropzone (`cui_dropzone`): [API][fpas-capture]. Capture shows the original
  tall and compact targets. Windows tests verify accepted and rejected shell
  drops and confirmation and cancellation in the native file dialog.
  Native workaround: [CUI-015](cui-upstream-bugs.md#cui-015).
  TUI: graphical profile required.
- [x] Avatar upload (`cui_avatar_upload`): [API][fpas-capture]. Capture shows the
  original profile photo control. Windows verifies Unicode file paths and
  replacement of the rendered image. Native workarounds:
  [CUI-014](cui-upstream-bugs.md#cui-014) and [CUI-015](cui-upstream-bugs.md#cui-015).
  TUI: graphical profile required.
- [x] Signature pad (`cui_signature_pad`): [API][fpas-capture]. Capture shows
  the original 520 by 180 pad and verifies native ink, stroke boundaries,
  and clearing. In the Windows software comparison at 1000 by 1000, five
  pixels in the Capture content differ by at most one level per color channel.
  TUI: graphical profile required.

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
- [x] Table of contents and anchors (`cui_toc`, `cui_toc_anchor`). [API][fpas-reader].
  Reader shows the original article and its indented outline. The gallery
  exercises pointer and Enter jumps, wheel-driven selection, and the painted
  active marker in Windows Software and OpenGL. The Windows software Reader
  area matches CUI at rest, after a jump and after scrolling at 1100 × 850.
  The graphical profile is required.
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
- [x] Dialog and action footer (`cui_dialog`, `cui_dialog_footer`):
  [API][fpas-dialog]. Overlays shows the original Edit profile example.
  Buttons and Overlays share the original 400-point project deletion dialog.
  The gallery verifies both launchers, cancellation, confirmation and Undo.
  The dialog interior matches the Windows software reference.
  Windows Software and OpenGL cover footer alignment, focus return, Escape,
  scrim and close dismissal, and the Save changes toast. TUI: deferred;
  graphical profile required.
- [x] Sheet/Drawer (`cui_sheet`): [API][fpas-sheet]. TUI: implemented; cell placement
  replaces CUI's slide transition, retaining modal focus, scrolling and dismissal.
  The Notifications sheet shows the original three switches. Their values are
  shared with Selection and Edit profile. The header and switches match the
  Windows software reference. The Recent list depends on List item.
- [ ] Command palette (`cui_command_palette`)
- [x] Accordion (`cui_accordion`): [API][fpas-accordion]. Lists shows the original
  three questions. Native headers propose model-owned expansion; pointer,
  Enter/Space, disabled input, closing bodies and nested dialogs are covered.
- [x] Hover card (`cui_hover_card`). [API][fpas-hover-card]. Disclosure verifies
  hover delay, moving onto the original @jonas card, leaving it and dismissal
  with Escape. The open and closed examples match the Windows software reference
  pixel for pixel. The bridge applies the workaround for [CUI-016](cui-upstream-bugs.md#cui-016).
  The graphical profile is required.
- [x] Confirm popover (`cui_confirm_popover`): [API][fpas-popovers].
  Disclosure shows the original Delete branch question. Windows Software and
  OpenGL cover separate open and confirm actions and cancellation. TUI:
  deferred; graphical profile required.
- [x] Expandable text (`cui_expandable_text`): [API][fpas-expandable-text].
  Disclosure shows the original clamped About text. CUI owns expansion, wrapping,
  fade and the optional link. Removing the widget releases local state.
  Native focus loss after toggling is documented as [CUI-006][cui-006]; no
  integration workaround or upstream contribution is planned.
- [x] Expandable card (`cui_expandable_card`): [API][fpas-accordion]. Disclosure
  shows Billing and Security with their native descriptions, icons and badge.
  CUI owns layout, chevrons and height springs. Software/OpenGL host tests cover
  model proposals, keyboard/pointer input, closing content and responsive views.
  Native pixel comparisons cover all three widgets in light/dark and narrow/wide
  configurations. Gallery captures cover Lists and Disclosure.
- [ ] Guided tour (`cui_tour`)

### Feedback and progress

- [x] Toast, including timed undo toast (`cui_toast`, `cui_toast_action_clicked`):
  [API][fpas-toast]. Overlays shows the four native examples and a timed Undo.
  Disclosure shows the original Archive 3 conversations trigger and draining
  Undo timer. The gallery exercises the action and captures both states.
  Navigation uses toasts for breadcrumb activation and Finish. The gallery
  checks stacking, hover pause, expiry, dismissal and action delivery.
  TUI: graphical host command required.
- [x] Alert (`cui_alert`): [API][fpas-feedback]. Feedback shows the four original
  alerts. CUI owns dismissal; pointer, Enter/Space, disabled input, stable literal
  titles, once-only notification, removal and reinsertion are tested.
- [x] Progress bar (`cui_progress`): [API][fpas-feedback]. Feedback shows the
  labeled upload percentage and indeterminate indexing bar. CUI owns animation;
  validation, native dimensions, message mapping and resizing are tested.
- [x] Meter (`cui_meter`): [API][fpas-meter]. Feedback shows the original
  Storage example with 18.4 / 20 GB. CUI owns formatting, interpolation and warning
  colors. Tests cover zero/exceeded capacity, literal text, value validation,
  message mapping, resizing and release of native interpolation on removal.
  Direct native pixel comparisons cover 64 threshold, capacity, label/unit,
  light/dark and narrow/wide combinations.
- [x] Spinner (`cui_spinner`): [API][fpas-feedback]. Feedback shows the original
  16/22/28-point sizes. Native rendering is checked alongside alerts and progress
  against direct CUI calls in 152 light/dark, narrow/wide and option combinations.
  Feedback captures cover both renderers in light, dark and narrow layouts.
- [x] Skeleton (`cui_skeleton`): [API][fpas-skeleton]. Feedback reproduces
  the original Loading avatar, percentage text lines and body placeholder.
  CUI owns shimmer, animation scheduling, rounding and reduced-motion behavior.
  Software/OpenGL tests cover fill/fixed/percentage widths, default height,
  resizing, typed mapping, passive inspection and GUI-only preflight. Direct
  native pixel comparisons cover 144 width, height, theme, motion and viewport
  combinations, including native percentage button widths. Feedback captures
  show Loading in light, dark and narrow views.
- [x] Progress ring (`cui_progress_ring`): [API][fpas-progress-ring]. Feedback
  shows the original Onboarding and Quota examples at 104 points. CUI owns
  percentage formatting, painting and interpolation. Software/OpenGL tests
  cover literal captions, native diameters, resizing, typed mapping and GUI-only
  preflight. Direct native pixel comparisons cover 192 value, size, caption,
  theme, width and state-retirement combinations. Feedback captures show the
  rings in light, dark and narrow layouts.
- [x] Timeline item (`cui_timeline_item`): [API][fpas-timeline]. Feedback
  reproduces the original deployment, review and failed-build Activity entries.
  CUI owns wrapping, title/time alignment, tone colors, icons and connectors.
  Software/OpenGL tests cover literal text, final-item spacing, resizing,
  typed mapping, inspection and GUI-only preflight. Direct native pixel
  comparisons cover 576 tone, optional-field, last-item, theme and width
  combinations. Feedback captures show Activity in light, dark and narrow views.
- [x] Banner (`cui_banner`). [API][fpas-disclosure-feedback]. Disclosure
  shows the original release announcement and verifies its action, dismissal
  and reopening. The banner matches the Windows software reference pixel
  for pixel. The graphical profile is required.
- [x] Countdown (`cui_countdown`). [API][fpas-disclosure-feedback]. Disclosure
  shows the original standard and compact countdowns, including hidden days.
  The gallery verifies live updates. The Windows software capture matches
  the original at the same remaining time. The graphical profile is required.
- [x] Onboarding checklist (`cui_checklist`). [API][fpas-disclosure-feedback].
  Disclosure shows the original workspace steps and verifies focus, keyboard
  toggling, action completion and advancement. Six pixels in the Windows
  software comparison differ by one level per color channel at most.
  The bridge applies the workaround for [CUI-010](cui-upstream-bugs.md#cui-010).
  The graphical profile is required.

### Lists, tables, and files

- [ ] List item (`cui_list_item`). Use it for the original Recent rows in the
  Notifications sheet as well as the Data display page.
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

[revision]: https://github.com/yappologistic/CUI/tree/c8baf35008807d66efe8e766c8964122227a2f80
[components]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/docs/components.md
[more]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/docs/components-more.md
[header]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/include/cui/cui.h
[gallery]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/examples/gallery_ui.c
[gallery-more]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/examples/gallery_more.c
[theming]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/docs/theming.md
[layout]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/docs/layout.md
[icons]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/docs/icons.md
[app-id]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/docs/getting-started.md#your-apps-name-and-icon-on-linux
[input-limits]: https://github.com/yappologistic/CUI/blob/c8baf35008807d66efe8e766c8964122227a2f80/README.md#limitations
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
[fpas-ruler-histogram]: ../../src/Fpas.Ui/README.md#ruler-and-histogram-sliders
[fpas-audio]: ../../src/Fpas.Ui/README.md#audio-controls
[fpas-capture]: ../../src/Fpas.Ui/README.md#capture-controls
[fpas-surfaces]: ../../src/Fpas.Ui/README.md#native-grids-and-surfaces
[fpas-reader]: ../../src/Fpas.Ui/README.md#table-of-contents-and-scroll-anchors
[fpas-hover-card]: ../../src/Fpas.Ui/README.md#link-avatar-and-hover-card
[fpas-disclosure-feedback]: ../../src/Fpas.Ui/README.md#banner-countdown-and-checklist
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
[fpas-reactions]: ../../src/Fpas.Ui/README.md#reactions
[fpas-speed-dial]: ../../src/Fpas.Ui/README.md#speed-dial
[fpas-swipe-actions]: ../../src/Fpas.Ui/README.md#swipe-actions
[fpas-user-menu]: ../../src/Fpas.Ui/README.md#user-menu
[fpas-context-menu]: ../../src/Fpas.Ui/README.md#context-menu
[fpas-popovers]: ../../src/Fpas.Ui/README.md#popover-and-confirmation-popover
[fpas-text-field]: ../../src/Fpas.Ui/README.md#text-field
[fpas-search-field]: ../../src/Fpas.Ui/README.md#search-field
[fpas-text-area]: ../../src/Fpas.Ui/README.md#text-area
[fpas-number-field]: ../../src/Fpas.Ui/README.md#number-field
[fpas-tag-input]: ../../src/Fpas.Ui/README.md#tag-input
[fpas-one-time-code]: ../../src/Fpas.Ui/README.md#one-time-code-input
[fpas-password-strength]: ../../src/Fpas.Ui/README.md#password-strength-meter
[fpas-select]: ../../src/Fpas.Ui/README.md#select
[fpas-multi-select]: ../../src/Fpas.Ui/README.md#multi-select
[fpas-date-range]: ../../src/Fpas.Ui/README.md#date-range
[fpas-calendar]: ../../src/Fpas.Ui/README.md#calendar-and-date-picker
[fpas-color-emoji]: ../../src/Fpas.Ui/README.md#color-and-emoji-pickers
[fpas-time-controls]: ../../src/Fpas.Ui/README.md#time-picker-wheel-picker-and-time-dial
[fpas-dialog]: ../../src/Fpas.Ui/README.md#dialog-and-action-footer
[fpas-accordion]: ../../src/Fpas.Ui/README.md#accordion-and-expandable-card
[fpas-expandable-text]: ../../src/Fpas.Ui/README.md#expandable-text
[fpas-feedback]: ../../src/Fpas.Ui/README.md#alerts-progress-and-spinners
[fpas-meter]: ../../src/Fpas.Ui/README.md#meter
[fpas-progress-ring]: ../../src/Fpas.Ui/README.md#progress-ring
[fpas-timeline]: ../../src/Fpas.Ui/README.md#timeline-item
[fpas-skeleton]: ../../src/Fpas.Ui/README.md#skeleton
[cui-006]: cui-upstream-bugs.md#cui-006
[fpas-tooltip]: ../../src/Fpas.Ui/README.md#tooltip
[glyphs]: ../../src/Fpas.Ui.Tui/Internal/Rendering/Glyphs.fpas
[gui-gallery-test]: ../../tests/Fixtures/gui-host/PublishedGallery.fpas
[gui-layout-test]: ../../tests/Fpas.Ui.Gui.Cui.Tests/CuiHostLayoutTests.cs
[tui-gallery-test]: ../../tests/Fpas.IntegrationTests/Tui/TuiGalleryTests.cs
[tui-contract]: ../../tests/Fixtures/tui/CuiContract.fpas
