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

Validate widget batches with builds of the affected libraries/gallery, focused
native and binding tests, and strict gallery-content pixel comparisons at the
same viewport, scale, theme, renderer and animation state. Integration tests,
publishing, deployment and Native AOT runs are excluded unless the user requests
them. Record confirmed CUI defects with a stable local ID, a direct reproduction
and a small tested workaround in the FPAS bridge.

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

Give each widget a dedicated FPAS unit in `src/Fpas.Ui/Widgets/<Topic>`
and a GUI implementation in `src/Fpas.Ui.Gui/Internal/Widgets/<Topic>`.
Use the directory hierarchy in the unit name, for example
`Fpas.Ui.Gui.Internal.Widgets.Overlays.Sheet`. Put native C adapters in
`src/Fpas.Ui.Gui.Cui/native/widgets/<topic>` and group the managed binding
and interop by the same topic. Add bindings when the widget needs behavior
outside the existing primitives. Adapters with identical begin/end contracts
can share traversal while keeping separate public units. Theme switcher and
theme toggle compose native segment and icon-button primitives.
Shared dispatch, layout translation, focus and input stay with
their host. No new terminal implementation file is required.

## Reference

The inventory targets the [vendored CUI snapshot][vendor] at
[`86d9781bbc3c35c264ec46287ad89c1d86a4c4de`][revision]. Each checkmark covers
the API and gallery behavior described in its entry.

The inventory combines the [core catalog][components], [extended
catalog][more], [public header][header], and both parts of the native gallery
([core][gallery], [extended][gallery-more]). It splits catalog pages that
describe multiple widgets, such as checkbox, switch, and radio. Options and
companion calls belong to their widget. Renderer, window,
font, and resource APIs are outside this inventory. Layout primitives,
axis formatting, and chart annotations are listed separately because the
gallery also needs them.

## Shared foundations

### Strict gallery pixel parity

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

### Native window identity

- [ ] Expose the Linux application ID through GUI options, the managed window,
  and the native bridge into `cui_app_desc.app_id`. Match the installed
  `.desktop` filename and verify the resulting X11 `WM_CLASS`. The ID is read
  at window creation and is ignored on Windows. See [desktop identity][app-id].

### Theme compatibility

GUI and TUI already share [`Fpas.Ui.Widgets.Theming.Themes`][fpas-themes]. The contract has
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

Beyond that shared subset, GUI exposes native layered stacks, independent width/height constraints,
per-edge padding, flexible and fixed gaps, and inherited LTR/RTL layout.
`ScrollView` optionally scrolls horizontally as well as vertically, with native
clipping and edge fades. Its model-owned position remains vertical, matching
CUI's scroll-to API. The gallery resets its viewport for every destination
selection, including the active destination. The supplementary Layout page
demonstrates these primitives in light, dark and narrow captures.
See the [graphical layout API][fpas-native-layout] and [CUI layout reference][layout] for CUI's directional
limits; mirrored layout does not add text shaping or bidirectional editing.

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

- [x] Text, including heading, label, secondary, muted, and inline code
  (`cui_text`, `cui_heading`, `cui_label`, `cui_secondary`, `cui_muted`, `cui_code`).
  `Fpas.Ui.Widgets.Display.Texts` exposes the native roles used on Text and display.
  `Elements.Text` remains available in the portable profile.
- [x] Link (`cui_link`). [API][fpas-hover-card]. Disclosure verifies pointer
  and keyboard activation of the original @jonas link and its visible focus
  ring. The bridge applies the workaround for [CUI-012](cui-upstream-bugs.md#cui-012).
  The graphical profile is required.
- [x] Icon (`cui_icon`): standalone file icon from the CUI icon specimen.
- [x] Emoji display (`cui_emoji`): the original three emoji and counts.
- [x] Keycap (`cui_kbd`): native Ctrl Shift P keycaps.
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
- [x] Image (`cui_image`): the original illustration with native rounding.
  The API accepts file and RGBA sources and reuses native texture ownership.
- [x] Interactive canvas (`cui_canvas`): native rectangle, circle and line
  commands and local click coordinates. The gallery reuses the original
  sidebar swatch drawings in a standalone selection example. The native test
  checks the click coordinates. Custom paint callbacks and hover/press
  notifications remain unexposed.

Text and display matches its native CUI reference pixel for pixel on Windows
in Software and OpenGL at 1100 × 1700 and 550 × 1700, scale 1. The gallery
contract checks the keycap and inline code text and clicks a swatch.

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
- [x] Control tile (`cui_control_tile`): [API][fpas-motion]. Motion reproduces
  the five original settings, including the wide appearance tile. The Windows
  test toggles Wi-Fi and checks the model proposal.

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
  inside an enclosing viewport. The native adapter regression also covers mixed
  wheel axes and edge handoff using CUI's own routing.
  TUI: deferred; graphical profile required.
- [x] Number field, including suffix (`cui_number`): [API][fpas-number-field].
  Inputs shows Seats with model-owned bounds, native step buttons and editable
  numeric text. CUI validates finite drafts and refreshes committed or blurred
  text; native adapter regressions cover clamping, NaN and incomplete numbers.
  TUI: deferred; graphical profile required.
- [x] Inline edit (`cui_inline_edit`): [API][fpas-smart-inputs]. Smart inputs
  shows the original roadmap title and owner. CUI retains the draft; Enter or
  blur proposes a model commit and Escape cancels. Native state, undo offsets,
  focus and borrowed text survive frames and reconcile model replacements.
  The public faces are Medium, Display and Mono; custom loaded fonts remain
  part of the separate native font/resource API.
- [x] Scrub field (`cui_scrub`): [API][fpas-smart-inputs]. Smart inputs shows
  Opacity, Radius and Blur with their original widths, suffixes and steps.
  CUI handles dragging, Shift precision, arrow keys and click-to-type. Invalid
  commits are protected by [CUI-017](cui-upstream-bugs.md#cui-017).
  Arbitrary native printf formats remain unexposed; the API uses DecimalPlaces.
- [x] Money field (`cui_money_input`): [API][fpas-smart-inputs]. Smart inputs
  shows the original USD amount and whole-euro salary with grouped resting
  values and plain editing digits. The model uses double, with zero through
  fifteen fractional digits. Greater native precision is unexposed.
  Oversized proposals are protected by [CUI-018](cui-upstream-bugs.md#cui-018).
- [x] Shortcut recorder and shortcut label (`cui_shortcut_recorder`,
  `cui_shortcut_label`): [API][fpas-smart-inputs]. Smart inputs shows the original
  palette and unbound search shortcuts. CUI owns recording, cancellation, clear,
  modifier order and platform key names; passive labels use native keycaps.

These four Smart inputs widgets were reviewed against `86d9781`. Targeted
software/OpenGL binding tests cover draft retention, model proposals, rejection,
focus, keys, pointer dragging and removal. Native tests compare 224 images
exactly with direct CUI calls. The Smart inputs page content matches the pinned
native gallery pixel for pixel in light/dark, wide/narrow software and OpenGL
captures; the existing application branding and navigation are outside that
page-content comparison.
- [x] Tag input (`cui_tag_input`): [API][fpas-tag-input]. Inputs shows Topics
  with native tag entry, removal and per-event limit enforcement.
  TUI: deferred; graphical profile required.
- [x] One-time code input (`cui_otp_input`): [API][fpas-one-time-code]. Inputs
  shows Verification code with change and native completion messages. The adapter
  regression covers replacing a complete code and repeating the same paste.
  TUI: deferred; graphical profile required.
- [x] Password strength meter (`cui_password_strength`): [API][fpas-password-strength].
  Inputs shows CUI's native score and label for New password.
  TUI: deferred; graphical profile required.
- [x] Select, including searchable combobox (`cui_select`): [API][fpas-select].
  Inputs shows Country and searchable Language choices with stable option
  indices. CUI restores trigger focus after dismissing its search; native adapter
  regressions cover Escape followed by Enter to reopen.
  TUI: deferred; graphical profile required.
- [x] Multi-select (`cui_multi_select`): [API][fpas-multi-select]. Pickers
  shows the original searchable Languages example. Gallery tests cover
  filtering, controlled selection, and dismissal with focus return.
- [x] Calendar (`cui_calendar`): [API][fpas-calendar]. The supplementary
  Inputs example shares the Due date selection. The gallery verifies
  day selection and browsing across a year boundary. Its Windows capture
  matches a standalone native CUI calendar. The binding tracks the title and
  month/year choices in the current snapshot; native keyboard navigation and
  exact pixel comparisons cover all three views in light and dark themes.
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

### Creative controls

- [x] Status tag (`cui_status_tag`): [API][fpas-creative]. Creative controls
  shows all seven presets and the compact removable row. The Windows test
  removes the entire compact row and restores it.
- [x] Ratio slider (`cui_ratio_slider`): [API][fpas-creative]. The Coffee/Milk
  and Train/Test examples use native pointer and keyboard input. The test
  changes the share with the pointer and checks Shift+Right.
- [x] Gradient slider (`cui_gradient_slider`): [API][fpas-creative]. The original
  hue, white balance and opacity controls include the swatch and eyedropper.
  Custom stops and numeric formatting are exposed. The tests exercise snapped
  input, the tool action and the CUI-031 endpoint workaround.
- [x] Step slider (`cui_step_slider`): [API][fpas-creative]. Variations snaps to
  the four original stops and Generate updates the displayed count.
- [x] Palette strip (`cui_palette_strip`): [API][fpas-creative]. The original
  five-swatch palette preserves locked colors during seeded shuffle.
- [x] Aspect ratio picker (`cui_aspect_picker`): [API][fpas-creative]. The gallery
  uses the six native shapes and Custom. Tests cover pointer selection and
  one-step keyboard navigation.
- [x] Option field with action (`cui_option_action`): [API][fpas-creative].
  Background and Save as reproduce the original segmented wells. The gallery
  test selects WebP and verifies that Download uses the selected format.
- [x] Seed field (`cui_seed_field`): [API][fpas-creative]. The original dice and
  lock update the seed and mode. The test verifies that a locked seed stays put.

Creative controls requires the graphical profile. The original content is
pixel-identical on Windows with Software and OpenGL at 1100 × 1700 and
550 × 1700, scale 1, light theme and reduced motion. The compact test
checks that the slider wells stack in one column.

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
- [x] Dock and dock items (`cui_dock`, `cui_dock_item`, `cui_dock_separator`):
  [API][fpas-motion]. Motion shows the original eight icons with running dots,
  badge and separator. Files emits a toast.
- [x] Bottom tab bar (`cui_tab_bar`): [API][fpas-motion]. Motion shows the four
  original tabs and Inbox badge. The Windows keyboard test checks that Right
  moves one tab and transfers focus. FPAS works around CUI-029.
- [x] Menu bar and menus (`cui_menu_bar`, `cui_menu_bar_menu`): App chrome uses
  the CUI documentation's File, Edit, View, Go and Help menus. The gallery
  checks Alt+V and Enter, and the native test exercises F10 and Down.
- [x] Toolbar and toolbar buttons (`cui_toolbar`, `cui_toolbar_button`,
  `cui_toolbar_spacer`, `cui_toolbar_separator`). Include grouped and floating
  styles, overflow menus, groups, radio sets, primary actions, cycle pickers,
  and flyout markers (`cui_toolbar_group`, `cui_toolbar_radio`,
  `cui_toolbar_primary`, `cui_toolbar_cycle`, `cui_flyout_mark`). Canvas and
  bars shows the grouped and floating examples. The native test advances a
  cycle control. [API](../../src/Fpas.Ui/README.md#canvas-and-bars)
- [x] Document tabs (`cui_doc_tabs`): the App chrome editor shows the native
  strip with dirty markers and tooltips. The Windows gallery checks selecting,
  moving and closing the same document with the keyboard.
- [x] Status bar and its items (`cui_status_bar`, `cui_status_bar_text`,
  `cui_status_bar_item`, `cui_status_bar_progress`, `cui_status_bar_spacer`):
  the editor displays the branch, indexing progress and clickable status items.
  The Windows gallery switches encoding through the keyed native item.
- [x] Docking workspace and panels (`cui_dockspace`, `cui_dock_panel`): App chrome
  shows the reference panel arrangement, panel toggles and layout reset. Tests
  cover native drag, close, reopen, layout serialization and invalid restore.
- [x] Split view (`cui_split`): App chrome includes a resizable two-pane example.
  The Windows tests drag the divider and verify [CUI-040](cui-upstream-bugs.md#cui-040)
  when pane minimums exceed the allocated width.

The [desktop chrome APIs](../../src/Fpas.Ui/README.md#desktop-chrome) require
the graphical profile. The native Windows test compares all five widgets with
direct CUI at 740 and 360 points, in light and dark, with zero differing pixels.
The Windows gallery contract exercises keyboard and pointer input in Software
and OpenGL. The supplementary page uses CUI's documentation examples as its
reference. Whole-example composition differences are tracked under strict
gallery pixel parity.

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
  shared with Selection and Edit profile. The sheet also shows the two original
  Recent rows. Its interior matches the Windows software reference.
- [x] Command palette (`cui_command_palette`): [API][fpas-command-palette].
  Overview and Overlays launch the original grouped command list.
  Windows Software and OpenGL checks cover native search, Enter activation,
  Escape with focus return and Ctrl+K. The visible panel interior matches the
  pinned reference in both renderers. The adapter guards keyboard activation
  during the exit animation for [CUI-024](cui-upstream-bugs.md#cui-024).
  The graphical profile is required.
- [x] Accordion (`cui_accordion`): [API][fpas-accordion]. Data display shows the original
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
  configurations. Gallery captures cover Data display and Disclosure.
- [x] Guided tour (`cui_tour`): [API][fpas-tour]. Audio shows the original
  three-step tour of the Cutoff knob, master meter and Bass fader. CUI owns
  the spotlight, card, positioning, animation and input. Software/OpenGL tests
  cover Next, Back, Done, Escape, closing, reopening, focus, model proposals
  and state retirement. Native pixel comparisons cover light/dark themes,
  centered/targeted cards, all three steps, widths and spotlight options.
  Custom captions and step-count formatting remain unexposed native options.
- [x] Veil (`cui_veil`): Disclosure reproduces the picture, sample identifier
  and spoiler. Windows Software and OpenGL tests cover pointer reveal and
  re-cover, the Reveal/Hide button and Enter on the focused picture.
  The covered and revealed cards were compared at both widths, using the
  same synthetic sample identifier in the reference. Subpixel placement
  differences remain on Disclosure. [CUI-038](cui-upstream-bugs.md#cui-038)
  records the tested workaround for a disabled veil intercepting input.

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
  labeled upload percentage, indeterminate indexing bar and native dithered
  export/sync examples. CUI owns animation;
  validation, native dimensions, message mapping and resizing are tested.
- [x] Meter (`cui_meter`): [API][fpas-meter]. Feedback shows the original
  Storage example with 18.4 / 20 GB. CUI owns formatting, interpolation and warning
  colors. Tests cover zero/exceeded capacity, literal text, value validation,
  message mapping, resizing and release of native interpolation on removal.
  Direct native pixel comparisons cover 64 threshold, capacity, label/unit,
  light/dark and narrow/wide combinations.
- [x] Spinner (`cui_spinner`): [API][fpas-feedback]. Feedback shows the original
  16/22/28-point sizes. Native rendering is checked alongside alerts and progress
  against direct CUI calls in 224 light/dark, narrow/wide and option combinations,
  including determinate and indeterminate dither progress.
  Feedback captures cover both renderers in light, dark and narrow layouts.
  Feedback also shows the original dither styles and inline examples.
  The API exposes native ink and track colors.
- [x] Status spinner (`cui_status_spinner`): the original Feedback Upload
  example shows Busy and alternates Success and Error. Targeted checks cover
  the transitions, result expiry and retained results.
- [x] Skeleton (`cui_skeleton`): [API][fpas-skeleton]. Feedback reproduces
  the original Loading avatar, percentage text lines and body placeholder.
  CUI owns shimmer, animation scheduling, rounding and reduced-motion behavior.
  Software/OpenGL tests cover fill/fixed/percentage widths, default height,
  resizing, typed mapping, passive inspection and GUI-only preflight. Direct
  native pixel comparisons cover 144 width, height, theme, motion and viewport
  combinations, including native percentage button widths. Feedback captures
  show Loading in light, dark and narrow views.
- [x] Skeleton group (`cui_skeleton_group`): the original Loading example
  shares CUI's dither sweep across nested placeholder rows.
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

- [x] List item (`cui_list_item`): [API][fpas-data-display]. Data display
  shows the original file list. The Notifications sheet shows its Recent rows.
- [x] Tree (`cui_tree_begin`, `cui_tree_clicked`): [API][fpas-data-display].
  Data display uses the original source tree. Windows Software and OpenGL
  checks cover expansion, collapse and selection.
- [x] Virtual list (`cui_virtual_list`): [API][fpas-data-display]. The
  original 10,000 invoices are realized on demand. Windows checks cover wheel
  scrolling and the immediate range after shrinking, with a workaround for
  [CUI-023](cui-upstream-bugs.md#cui-023).
- [x] Reorderable list and items (`cui_reorder_begin`, `cui_reorder_item`):
  [API][fpas-data-display]. The original Tasks example saves the order
  proposed by native drag input and shows its toast.
- [x] Table, rows, and cells (`cui_table`, `cui_table_row`, `cui_table_cell`,
  `cui_table_cell_text`): [API][fpas-data-display]. Data display shows the
  original People example. Windows checks cover reversing the sort, selecting a
  row and the native focus ring supplied for [CUI-022](cui-upstream-bugs.md#cui-022).

Data display content and the Notifications sheet interior match the pinned
Windows reference in Software and OpenGL at 1100×1800 and scale one. The
reference uses the same reserved example domain for the People addresses.
These widgets require the graphical profile.

- [x] Data grid, including editable cells (`cui_data_grid_begin`,
  `cui_data_grid_row`, `cui_data_grid_cell`, `cui_data_grid_cell_text`,
  `cui_data_grid_edit`, `cui_data_grid_cell_edit`):
  [API](../../src/Fpas.Ui/README.md#data-grid-filter-bar-and-tree-table).
  The original People demo realizes rows on demand from 2,000 records.
  Windows checks cover sorting, column resizing, range selection, editing and
  scrolling. [CUI-025](cui-upstream-bugs.md#cui-025) covers shrinking while scrolled.
- [x] Filter bar (`cui_filter_bar`): the original Team, Status and Role filters
  and Search field. Windows checks cover search, clearing and adding a filter.
  The adapter opens the newly added chip's value menu for
  [CUI-026](cui-upstream-bugs.md#cui-026).
- [x] Tree table (`cui_tree_table_begin`, `cui_tree_table_row`,
  `cui_tree_table_name`, `cui_tree_cell`): the original source-tree cells in a
  table and a supplementary virtual tree table with native selection, activation
  and lazy children. Windows checks cover both forms.

The original Data grid page content matches the Windows Software and OpenGL
reference at 1100×1550 and scale one when both galleries use the same reserved
example domain and a stable tie-break for equal sort keys. The pinned CUI demo's
default `qsort` can order tied rows differently. A direct CUI rendering of the
supplementary example also matches in both renderers. The new widgets require
the graphical profile.

- [x] Path bar and its dropdown (`cui_path_bar`, `cui_path_bar_menu_begin`):
  native segment navigation, editable path and child menus. Files shows
  the original browser path and a standalone example.
- [x] File browser (`cui_file_browser`): original Files browser with a listing
  provider, native folder history and keyboard navigation.
- [x] File picker (`cui_file_picker`): the original Open and Save as examples
  with native list navigation, filename entry and cancellation.

The original Files browser, button row and the Open and Save as dialog
panels match pixel for pixel in Windows Software and OpenGL at 1100×2200
and scale one with identical fixture files. The separate Terminal example
is tracked under code and tool panels. [Files API][fpas-files-planning]

### Code and tool panels

- [x] Code block (`cui_code_block`): the Code page reproduces the C, Python
  and shell examples and adds a native SQL sample. The focused native test
  verifies clipboard contents and focus after the copy button changes glyph.
- [x] JSON viewer (`cui_json_view`): the Code page reproduces the expandable
  document and native parse error. Windows checks cover folding and invalid
  text, including [CUI-027](cui-upstream-bugs.md#cui-027).
- [x] Markdown view (`cui_markdown`): original release notes, task checkboxes,
  links and streaming replay. The native test verifies keyboard activation and
  UTF-8 task offsets.
- [x] Code editor (`cui_code_editor`): original editable C tile cache and an
  editable SQL example. Native tests cover UTF-8 edits, capacity refusal and
  undo reset after external replacement. Programmatic caret and find requests
  and custom marks remain unexposed.
- [x] Diff viewer (`cui_diff_view`): original config loader comparison with
  split/inline selection, change navigation and folding. The native test
  verifies keyboard navigation and responsive inline fallback.
- [x] Property inspector and editors for float, integer, text, boolean, enum,
  color, and vectors (`cui_inspector`, `cui_inspector_group`, `cui_prop_float`,
  `cui_prop_int`, `cui_prop_text`, `cui_prop_bool`, `cui_prop_enum`,
  `cui_prop_color`, `cui_prop_vec2`, `cui_prop_vec3`, `cui_prop_vec4`): the original crate transform,
  material and shadows. Native tests toggle a property group and a boolean row.
- [x] Log console (`cui_log_view`): original timestamped build log with native
  filters, copy, follow and clear. The native test verifies timestamped
  clipboard contents and the clear proposal.
- [x] Terminal widget (`cui_terminal`): original Files shell with built-in
  commands, ANSI output, native input and history. The native test verifies
  submission, history and Ctrl+C; the gallery contract verifies command
  routing. CUI keeps the echo and history while the widget is mounted.
- [x] Hex viewer (`cui_hex_view`): original editable PNG specimen with marked
  ranges, selection, overwrite and Go to offset. The native test checks the
  selected address and edited byte.
- [x] Flame graph (`cui_flame_graph`): the original two-frame profiler capture.
  Native tests select a child and zoom to it. The disabled-input workaround
  is tracked in [CUI-039](cui-upstream-bugs.md#cui-039).
- [x] Curve editor (`cui_curve_editor`): the original easing curve and the
  selected timeline track's curve. Native tests nudge and delete a key.
- [x] Gradient editor (`cui_gradient_editor`): the original flame ramp.
  Native tests nudge a stop, delete it and preserve the transparent endpoint.
- [x] Timeline editor (`cui_timeline_editor`): the original hero, camera and
  sound tracks, range, playhead and track curve. Tests cover exact frame steps,
  keyframe moves and deletion, playback requests and the shared gallery model.
  See [CUI-039](cui-upstream-bugs.md#cui-039) for disabled wheel input.

The five CUI component specimens are grouped under Editing tools. Windows
Software and OpenGL captures match the native reference at 1100 × 1400 and
550 × 1400, scale 1, across three scroll positions.

The Code page's new sections and the original Files content through Terminal
match their CUI references pixel for pixel on Windows at widths 1100 and 550,
scale 1, using Software and OpenGL. The Code comparison covers four scroll
positions per width. The supplementary Files path bar is outside the comparison.

### Developer tools

- [x] Key-value editor (`cui_kv_editor`)
- [x] Problems panel (`cui_problems`)
- [x] Pipeline (`cui_pipeline`)
- [x] Query builder (`cui_query_builder`)
- [x] Schedule picker (`cui_schedule_picker`)
- [x] Keymap (`cui_keymap`)
- [x] Commit graph (`cui_commit_graph`)
- [x] Dependency graph (`cui_graph_view`)
- [x] Network waterfall (`cui_waterfall`)
- [x] Trace viewer (`cui_trace_view`)
- [x] Regex tester (`cui_regex_tester`)
- [x] Test explorer (`cui_test_explorer`)
- [x] Stack trace (`cui_stack_trace`)
- [x] HTTP request bar (`cui_request_bar`)
- [x] HTTP response status (`cui_response_status`)
- [x] Uptime bars (`cui_uptime`)
- [x] Time histogram (`cui_time_histogram`)
- [x] Coverage (`cui_coverage`)
- [x] Blame (`cui_blame`)
- [x] Feature flags (`cui_feature_flags`)
- [x] API explorer (`cui_api_explorer`)

The original Developer tools and More developer tools pages exercise these
[graphical APIs](../../src/Fpas.Ui/README.md#developer-tools). The page content
matches the pinned CUI reference on Windows at 1100 × 1700 and scale 1.
Software and OpenGL comparisons covered three scroll positions per page with
the same animation clock. Focused tests cover trace folding and measurement,
regex results, histogram selection and clearing, reduced motion, and URL
and query-parameter synchronization.
The native workarounds are tracked in [CUI-035](cui-upstream-bugs.md#cui-035)
and [CUI-036](cui-upstream-bugs.md#cui-036).

### Charts and instruments

- [x] Line/area chart, smoothing and stacked dither (`cui_line_chart`),
  grouped/stacked bars (`cui_bar_chart`) and responsive donut
  (`cui_donut_chart`): [API](../../src/Fpas.Ui/README.md#charts).
  Charts reproduces their original data, labels, formatting and theme palette.
- [x] Sparkline (`cui_sparkline`): a standalone example on Charts shows
  the native compact line and endpoint.
- [x] Gauge (`cui_gauge`): original CPU and Memory examples in Charts.
- [x] Pattern meter (`cui_pattern_meter`): original storage parts, free capacity,
  native hover/keyboard readout and Grayscale checkbox. CUI owns local selection;
  the bridge only translates data, tracks focus and retires native state.
  Focused native tests compare all five chart widgets without pixel tolerance;
  binding tests check copied-data lifetime, validation and meter keyboard input.
  Charts page content is pixel-identical to pinned CUI at scale 1, in light/dark
  software/OpenGL captures at 1240x2200 and 600x2800. Comparisons exclude the
  differing navigation chrome. Large unrepresentable automatic ranges are
  guarded as [CUI-019](cui-upstream-bugs.md#cui-019).
- [x] Heatmap (`cui_heatmap`), calendar heatmap (`cui_calendar_heatmap`),
  treemap (`cui_treemap`), radar (`cui_radar_chart`), funnel (`cui_funnel`),
  split bar (`cui_split_bar`) and waffle (`cui_waffle`):
  [API](../../src/Fpas.Ui/README.md#visualizations). More charts reproduces the
  original first seven examples, including their data, text, 1.3:1 column ratio,
  palette, labels and native responsive behavior. CUI owns drawing and input;
  the bridge supplies frame-owned text and optional activation/selection messages.
  Calendar activation resolves a native visible-week index to the original date.
  Focused checks cover native geometry and pixels, copied-data lifetime, disabled
  input, model-owned treemap selection, calendar clipping and width changes.
  The full More charts page is validated with native content captures at matching
  theme, scale, viewport and renderer; application navigation is excluded.
- [x] Sankey (`cui_sankey`), bar list (`cui_bar_list`), activity rings
  (`cui_activity_rings`), scatter/bubble chart (`cui_scatter_chart`), radial
  gauge (`cui_radial_gauge`) and candlestick/OHLC chart (`cui_candlestick`):
  [API](../../src/Fpas.Ui/README.md#flows-ranks-and-instruments).
  More charts reproduces the complete original page, including Shuffle,
  native ring progress above one, bubble sizes, all three radial styles and
  2,000 OHLC periods. CUI owns layout, animation, hit-testing, zoom, panning,
  crosshair/keyboard input and following appended periods. The bridge only
  translates data/events, owns deferred text and retires retained state.
  Focused checks cover pixel/geometry parity, input-data lifetime, original
  callback indices, native rank changes and released item/viewport allocations.
  Full-page content matches the pinned native gallery with zero differing pixels
  in light/dark themes, software/OpenGL renderers and 1240×4100/600×4800
  viewports at scale one. The 640 native pixel/geometry comparisons and 19
  managed binding cases also cover input and allocation retirement.
  [CUI-020](cui-upstream-bugs.md#cui-020) and
  [CUI-021](cui-upstream-bugs.md#cui-021) have direct native reproductions and
  small protected adapter regressions.
- [x] Dot-matrix chart (`cui_dot_chart`): the original caffeine chart shows
  stacked cup doses, an hours axis and native annotations. Windows tests cover
  activation and the keyboard fallback for [CUI-028](cui-upstream-bugs.md#cui-028).
- [x] Event track (`cui_event_track`): the original cup timeline shares the
  chart's scale. Windows checks cover drag, keyboard movement, add and remove.
- [x] Readout (`cui_readout`): the original At Bed, Peak and Cups readouts
  react to changes in the timeline.
- [x] Dot-matrix display (`cui_dot_display`): the original bedtime status.

At 1100×1700 and scale one, the original Code sections and the Instruments
page content match the pinned Windows gallery in Software and OpenGL. The
additional SQL sample is outside the original Code comparison.

Line and bar charts expose native annotations through `Charts.Annotation`;
focused native parity covers a threshold and point with labels and details.
The dot-matrix chart uses the same annotation API. Direct Windows tests compare
X and Y references, an accented point, and label details pixel for pixel.
At 1100 × 1390, the Instruments content matches in both Software and OpenGL,
including the threshold, bedtime and peak annotations.

Include axis formatting through `cui_axis_units`, including `CUI_AXIS_HOURS`:
12.25 reads 12:15 and 30 reads 06:00 on the next day. Match these semantics
in the GUI chart bindings. For right-to-left views, readout and dot-display
placement follow reading direction while chart and event-track time axes remain left to
right. Arabic-script labels require CUI's optional static-text shaping support,
which remains a separate native build option.

### Conversation and media

- [x] Chat message (`cui_chat_message`): the original grouped conversation
  shows outgoing and incoming bubbles, sender information and read receipts.
  The API accepts avatar paths and native Markdown rendering. Markdown link and
  task proposals remain part of the open Markdown work.
- [x] Chat divider (`cui_chat_divider`): the original Today divider.
- [x] Typing indicator (`cui_typing_indicator`): Jonas and the native dots.
  The adapter stops the animation with reduced motion. [CUI-033](cui-upstream-bugs.md#cui-033)
- [x] Comment (`cui_comment`): the original three threaded comments.
  Windows checks cover pointer and keyboard likes and Reply events.
- [x] Presence (`cui_presence`): the original overlapping avatars with away
  state, overflow and native hover expansion.
- [x] Counter (`cui_counter`): Add a sale updates the original rolling amount
  with grouped thousands.
- [x] Ticker (`cui_ticker`): the original figures and native hover pause.

- [x] Carousel and slides (`cui_carousel`, `cui_carousel_slide`): the original
  four release slides with native arrows, dots, gestures and automatic advance.
  The Windows test selects a dot and moves with the keyboard.
- [x] Image viewer (`cui_image_viewer`): the original landscape with the CUI
  toolbar. The Windows test zooms with the wheel, pans and fits with the keyboard.
- [x] Image cropper (`cui_cropper`): the same landscape with the original
  square crop. The Windows test resizes a corner and checks the aspect ratio.
- [x] Image compare (`cui_image_compare`): the original grayscale and color
  scenes. The Windows test drags the divider and moves it with the keyboard.
- [x] Card stack and items (`cui_card_stack_begin`, `cui_card_stack_item_begin`):
  the original destination deck with Saved and Skipped toasts. The Windows
  test dismisses cards by mouse and keyboard and checks both directions.

The five media examples match the pinned Windows gallery pixel for pixel with
Software and OpenGL at scale one and a 1100×2130 client area. The Social and
media content differs only at the typing dots because FPAS honors reduced
motion. Feedback retains small timing and antialiasing differences. The
standalone sparkline matches direct CUI in the fixed-frame Software comparison.

- [x] Now playing (`cui_now_playing`): [API][fpas-motion]. Motion shows the
  original Low Tide track with playback, transport and seeking. The Windows
  test pauses playback and seeks on the native scrubber.
- [x] Waveform (`cui_waveform`): [API][fpas-motion]. Motion shows the original
  96 peaks. The Windows test drags across the waveform and checks the proposed
  position.
- [x] Voice waveform (`cui_voice_wave`): the original live timeline and
  seekable clip. Windows checks pointer and keyboard seeking.
- [x] Voice note (`cui_voice_note`): the original incoming note plus recorded
  takes. Windows checks play, seek and the playback-speed chip.
- [x] Voice session (`cui_voice_session`): the original shared timeline,
  scripted transcript, mute and end controls. Windows checks both call actions.
  The demo uses synthetic voice levels like the CUI original.

### Planning and diagram editors

- [x] Kanban board, columns, and cards (`cui_kanban_begin`, `cui_kanban_column_begin`,
  `cui_kanban_card_begin`): native columns, composed card content and drag between columns.
- [x] Week calendar (`cui_week_calendar`): native weekly events with keyboard move and resize.
- [x] Month calendar (`cui_month_calendar`): native month navigation, date selection and event dots.
- [x] Gantt chart (`cui_gantt`): native tasks, milestones, dependencies and keyboard move and resize.

The original Planning page matches pixel for pixel in Windows Software and
OpenGL at 1100×2200 and scale one. The adapters restore Enter activation of
read-only week events and Gantt tasks and click activation of read-only week
events. [Planning API][fpas-files-planning]
[CUI-032](cui-upstream-bugs.md#cui-032)

- [x] Node editor with pins, links, and node content (`cui_node_editor_begin`,
  `cui_node_begin`, `cui_node_input`, `cui_node_output`,
  `cui_node_content_begin`, `cui_node_link`): Canvas and bars uses the original
  three-node graph; the native test covers node dragging, wheel zoom and the
  disabled-input workaround. [CUI-037](cui-upstream-bugs.md#cui-037)

### Canvas and bars

- [x] Format bar (`cui_format_bar`): the original color, styles, alignment,
  list and delete controls. A native test toggles Bold.
- [x] Live cursors, presence pill, and selection ring (`cui_live_cursors`,
  `cui_presence_pill`, `cui_selection_ring`): the original collaborators,
  speaking indicator, cursor speech and node selection. Custom avatar images
  in the presence pill remain an unexposed native option.
- [x] Canvas controls (`cui_canvas_controls`): the original zoom, fit, Grid
  and Layers controls. The native test selects the next zoom preset and resets
  to 100 percent.
- [x] Result card (`cui_result_card`): the original picture, caption and
  status chip, with the floating toolbar below it.
- [x] Selection bar (`cui_selection_bar`): the five-file example with native
  bulk actions and Escape-to-clear. The Windows gallery test selects a fourth
  file and clears the selection. The native count-format override and count-only
  mode remain unexposed.

The [Canvas and bars APIs](../../src/Fpas.Ui/README.md#canvas-and-bars) use the
graphical profile. Windows captures at 1100 × 1700 and 550 × 1700, scale 1,
match the original page content at rest and after scrolling the narrow layout.
The wide OpenGL capture differs in one pixel by one RGB level on an
antialiased edge. The Creative controls regression captures match exactly at
both sizes in software and OpenGL.

### Motion

- [x] Shimmer text (`cui_shimmer_text`): [API][fpas-motion]. Motion shows the
  original thinking line beside the dots loader.
- [x] Typewriter text (`cui_typewriter`): [API][fpas-motion]. Motion shows
  streamed text with a caret. Native tests verify completion, replay and
  reduced motion. The GUI can emit a completion message and reports completion
  in the snapshot.
- [x] Text reveal (`cui_text_reveal`): [API][fpas-motion]. Motion shows the
  original release-build line. Replay restarts both text effects.
- [x] Rotating words (`cui_word_rotate`): [API][fpas-motion]. Motion rotates
  through the original four deployment targets.
- [x] Rolling number (`cui_number_roll`): [API][fpas-motion]. The additional
  Motion example changes a formatted currency value. The native test checks
  that the CUI-030 fallback preserves text outside the animation capacity.
- [x] Gradient text (`cui_gradient_text`): [API][fpas-motion]. Motion shows the
  original animated Kiln 2.4 heading.
- [x] Spotlight card (`cui_spotlight`): [API][fpas-motion]. Motion shows three
  original cards with pointer lighting and a beam on OpenGL 3.3.
- [x] Loader and its motion variants (`cui_loader`): [API][fpas-motion]. Motion
  shows dots, bars, pulse, orbit and grid in the original order.

Motion requires the graphical profile. The Windows gallery tests cover
Software and OpenGL. At 1100 × 1700, scale 1, with reduced motion and matched
clocks, the original content has zero differing OpenGL pixels and one Software
pixel on the active tab's antialiased edge. The supplemental number example
follows the original page content.

### AI interfaces

- [x] Reasoning panel (`cui_reasoning_begin`): the original expandable thought
  with app-supplied content and running/finished state.
- [x] Tool call (`cui_tool_call`): the original search and file calls, including
  expandable native JSON payloads for Search.
- [x] Agent plan (`cui_agent_plan`): the original research plan and its step
  states. Running glyphs honor reduced motion. [CUI-034](cui-upstream-bugs.md#cui-034)
- [x] Agent status (`cui_agent_status`): the original busy report-reading pill.
- [x] Source list (`cui_source_list`): the original horizontally scrolling cards.
- [x] Citation (`cui_citation`): the three original inline source previews.
- [x] Composer (`cui_chat_composer`): the original AI editor and Voice recorder.
  Windows checks UTF-8, Shift+Enter, send, stop, recording, cancel and voice send.
- [x] Model picker (`cui_model_picker`): the embedded menu and a supplementary
  standalone example. Windows checks keyboard selection.
- [x] Prompt suggestions (`cui_prompt_suggestions`): the original four cards.
  Choosing one fills the draft.
- [x] Response actions (`cui_response_actions`): the original copy, regenerate,
  voting and share toolbar. Windows checks vote and regenerate proposals.
- [x] Context meter (`cui_context_meter`): the original context percentage and
  breakdown by system, files and messages.
- [x] Voice orb (`cui_voice_orb`): a supplemental example on Voice for CUI's
  legacy widget. Clicking it switches from idle to listening.

At 1100 × 1800 and scale one, the original AI and Voice sections match the
pinned Windows gallery pixel for pixel in Software and OpenGL. The Voice
comparison fixes the demo time at 250 ms. The AI comparison uses a fixed
native animation phase to match the reduced-motion workaround. The standalone
model picker and legacy voice orb follow the original content.

[revision]: https://github.com/yappologistic/CUI/tree/86d9781bbc3c35c264ec46287ad89c1d86a4c4de
[fpas-motion]: ../../src/Fpas.Ui/README.md#motion-and-compact-controls
[fpas-creative]: ../../src/Fpas.Ui/README.md#creative-controls
[fpas-files-planning]: ../../src/Fpas.Ui/README.md#files-and-planning
[components]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/components.md
[more]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/components-more.md
[header]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/include/cui/cui.h
[gallery]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/examples/gallery_ui.c
[gallery-more]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/examples/gallery_more.c
[theming]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/theming.md
[layout]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/layout.md
[icons]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/icons.md
[app-id]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/docs/getting-started.md#your-apps-name-and-icon-on-linux
[input-limits]: https://github.com/yappologistic/CUI/blob/86d9781bbc3c35c264ec46287ad89c1d86a4c4de/README.md#limitations
[emoji]: https://www.unicode.org/reports/tr51/
[width]: https://www.unicode.org/reports/tr11/
[vendor]: ../../third-party/cui/UPSTREAM.md
[fpas-themes]: ../../src/Fpas.Ui/Widgets/Theming/Themes.fpas
[fpas-layout]: ../../src/Fpas.Ui/README.md#layout-and-style
[fpas-native-layout]: ../../src/Fpas.Ui/README.md#native-graphical-layout
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
[fpas-tour]: ../../src/Fpas.Ui/README.md#guided-tour
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
[fpas-smart-inputs]: ../../src/Fpas.Ui/README.md#smart-inputs
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
[fpas-data-display]: ../../src/Fpas.Ui/README.md#data-display
[fpas-command-palette]: ../../src/Fpas.Ui/README.md#command-palette
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
