# FPas GUI Gallery

[Illustrated GUI widget guide](../../../../docs/ui/gui/README.md)

Build and run `GuiWidgetGallery.fpasproj` after the
[local SDK bootstrap](../../../../docs/build/development.md#build-an-fpas-application).
The [CUI build reference](../../../../docs/ui/gui/native-build.md#prerequisites)
lists the native prerequisites.

The gallery follows CUI's desktop composition with a full-height 240-point sidebar,
a centered content column, and a left drawer below 760 logical points. The
sidebar and drawer share the Light and Dark selector and eight CUI accent
swatches. The gallery shows the accepted widget examples from the
[CUI inventory](../../../../docs/cui-widget-inventory.md).
See the [UI handbook](../../../../docs/ui/README.md) for widget examples and
screenshots.
See the [CUI gallery comparison](../../../../docs/future/cui-compare/README.md)
for the reviewed pages and their widget guides.

The companion [native API example](../native-api/README.md) demonstrates canvas
wheel handling, custom axis labels and trace links/geometry without changing
the original gallery composition.
The [editor commands example](../editor-commands/README.md) shows optional
editor requests, application shortcuts, account images, tour labels and
aggregate-selection settings alongside the gallery's default widget examples.

## Desktop launcher

The [FPAS distribution](../../../../docs/build/distribution.md#gallery-distribution)
publishes `fpas-gui-gallery` with Native AOT. The window title is **FPas GUI Gallery**.
The Windows executable embeds the [gallery icon](packaging/windows/FpasGuiGallery.ico).
The Linux [desktop launcher](packaging/linux/io.github.tobiasbick.FpasGuiGallery.desktop)
uses the matching [SVG icon](packaging/linux/io.github.tobiasbick.FpasGuiGallery.svg).

## Source structure

- `App/`: navigation, responsive window composition, root state and message routing.
- `Shared/`: common section composition and the owner of values used on several
  pages, including appearance, profile data and filter settings.
- `Pages/<Topic>/`: a page's state, messages, update and view. Stateful widget
  examples live in that page's `Widgets/` directory. Actions owns its background
  save commands. Overview owns the workspace form's local preference.
  Feedback retains alert dismissal and animation state in CUI.

Namespaces follow these directories, for example
`GuiGallery.Pages.Actions.State` and `GuiGallery.Pages.Actions.Widgets.SplitButton`.
There is one MVU session. The root embeds child views with `Fpas.Ui.Composition.Map`
and translates their transitions with `Fpas.Ui.Mvu.Composition.MapTransition`.
Each feature receives a stable background-effect scope. Shared values have one
owner, so the profile dialog and Inputs, or Filters and Selection, stay in sync.

## Localization

The GUI and TUI galleries share [English](Shared/Strings.resx) and
[German](Shared/Strings.de.resx) catalogs. The app uses the system UI culture
and falls back to English. On Linux, start a new process with
`LANG=de_DE.UTF-8` to see the German text. The catalogs cover navigation,
page headings, section descriptions, the overview and shared example controls.
Example data and per-widget demo text can retain their original wording.

Use [Strings.fpas](Shared/Strings.fpas) for captions. Keep resource keys and
widget IDs stable. See the [SDK localization guide](../../../../docs/build/msbuild.md#embedded-resources-and-localization)
for how the assemblies are built and loaded.

## Pages

- **Overview** shows the original Components, Frame time and Dependencies
  metrics. The native grid lays out the Workspace form and Team card with
  presence avatars and role badges. Name and email are shared with Inputs and
  Edit profile. Creating the demo workspace shows the success toast.
  Open command palette and Ctrl+K show the native searchable command list.
- [**Buttons**](../../../../docs/ui/gui/actions/README.md#buttons-gallery-page)
  shows variants, sizes, icons, and input states in rows that wrap as needed.
  Save changes shows the busy state, and Reset enables the button again. The
  formatting group demonstrates three joined actions. The copy button copies
  the displayed command. Hold to delete shows a danger toast after a full hold.
  Segmented control reproduces CUI's List/Board/Timeline icon choices and its
  small Day/Week/Month/Year selector, with independent model-owned selections.
  Dropdown menu clones CUI's Options and icon-only More actions examples,
  including section captions, separators, icons, shortcut hints and a danger
  action. Delete opens the same confirmation as the Overlays page.
- [**Inputs**](../../../../docs/ui/gui/inputs/README.md#inputs-gallery-page)
  shows the original responsive Text fields card with Full name,
  Email, Password, Search projects and About. The email example uses a
  reserved example domain. Typing a short password displays the validation
  error. The multiline About field spans the card and wraps on narrow screens.
  The next card shows Topics, the six-box Verification code starting with
  418, and New password with its strength meter. The two password fields share
  one value.
  Pickers shows Country, searchable Language, Due date, and Seats in their
  original grid positions, stacked at narrow widths, with Brand color beneath
  them. Open Due date to choose a day from its calendar.
- **Selection** shows native notification checkboxes, connectivity switches
  and the exclusive save-frequency radio group. The checkbox and switch
  cards share a row when both fit and stack at narrower widths. Radio cards
  offer three save destinations with Team server selected. Sliders show
  continuous Volume, Brightness in ten-point steps and a draggable Price range.
  The six category chips wrap on narrow screens. Design and Marketing start
  selected. The five-star rating starts at four stars and clears when the
  selected star is clicked again.
- **Navigation** shows underline tabs with model-selected content and pill
  tabs with icons. The breadcrumb shows the activated ancestor in a toast.
  The twelve-page pager keeps the current page, first page and
  last page visible on narrow screens. The stepper starts at Workspace and
  follows the Back and Next buttons. Finish shows an “All set” toast.
- **Overlays** shows the original Edit profile dialog with Name, Email and
  Public profile. Drag the header to move it or an edge or corner to resize it.
  The fields update shared preview values. Cancel closes the
  dialog and Save changes also shows a success toast. The same card opens the
  right-hand Notifications sheet with Comments, Mentions and Weekly digest
  switches and the two original Recent items. Switch values are shared with
  Selection and Edit profile. The Command palette button opens the same
  searchable command list as Overview. Delete
  project opens a confirmation with a native action footer. Confirming it
  displays an Undo toast. Escape, the close button and the scrim dismiss the
  open overlay.
  The context-menu area opens CUI's Cut/Copy/Paste/Delete menu on right click.
  Its commands only update preview state. Arrow keys and Enter choose a command;
  Escape and outside clicks dismiss it.
  The Filters button opens a native popover with two checkboxes, a score slider,
  and Apply. The values are shared with the matching gallery controls.
  The Toasts section includes success, failure and update notifications and
  an Archive a message example with a timed Undo action.
  The Tooltips section reproduces the six CUI toolbar buttons, with formatting
  shortcuts shown as key hints on hover.
- **Data display** shows the sortable People table, the expandable source tree,
  the file list, draggable Tasks, the native accordion and 10,000 invoices in a
  virtual list. People uses reserved example email addresses.
- **Data grid** shows the original Filter bar, editable 2,000-person grid and
  Tree table. The virtual tree below them demonstrates the standalone native
  API, including keyboard navigation and lazy children. The grid uses reserved
  example email addresses and stable ties when sorting.
- **Code** shows the original C, Python and shell code blocks, native copy
  buttons, expandable JSON and the parse-error example. An additional SQL
  block and editor demonstrate native SQL highlighting. Replay the Markdown
  release notes, edit the C tile cache, navigate the config diff, overwrite
  bytes in the PNG specimen, and filter or clear the build log.
- **Developer tools** shows the original trace, network waterfall, dependency
  and commit graphs, pipeline, problems, headers editor, query builder,
  schedule picker and keymap. Fold a trace lane or Shift-drag to measure.
- **More developer tools** shows the original test explorer, stack trace,
  blame, coverage, regex tester, HTTP preview, API explorer, feature flags,
  uptime and time histogram. The URL and parameters update each other in
  the local request example. Drag on the histogram to select an interval.
- **Instruments** shows the original caffeine example. Click the dot chart to
  add a cup, drag it along the event track or use the arrow keys. The readouts
  and bedtime dot display update with the cup times.
- **Motion** shows the original text effects, spotlight cards, loaders, dock,
  control tiles, Now playing, waveform and tab bar. Replay restarts the
  typewriter and reveal. An extra example changes a rolling currency value.
- **Creative controls** shows the original status tags, Coffee/Milk and
  Train/Test ratios, gradient and step sliders, palette locks and shuffle,
  aspect choices, export controls and seed dice.
- **Canvas and bars** shows the original grouped toolbar, format bar, three
  collaborators over the node graph, zoom rail, result picture and floating
  actions. Tick files to update the selection bar. Escape clears the selection.
  Under Presence avatars, choose a PNG or JPEG for Paul or switch back to initials.
- **App chrome** shows the editor chrome, docking workspace and split view.
  Window and clipboard maximizes, minimizes, restores and moves the gallery
  window and reads its placement. It also copies a sample path and pastes
  from the system clipboard.
- **Editing tools** shows the original crate inspector, two-frame flame graph,
  easing curve, gradient ramp and shot timeline. Move or select a timeline key
  to edit the same track in its curve below.
- **Text and display** shows typography, inline code, an icon, emoji, a
  keyboard shortcut, and the original preview picture. Click a color swatch
  to move the selection ring.
- **Files** shows the original browser, its Open and Save as dialogs, and a
  terminal with built-in `help`, `ls`, `colors`, `build`, `echo` and `clear`.
  Double-click a folder or press Enter to navigate into it. A supplementary
  standalone path bar demonstrates ancestor navigation, child menus and path entry.
- **Planning** shows the original Kanban board, week schedule, release Gantt
  and month calendar. Drag a card across columns, move or resize an event or
  task, and change the visible month or select a day.
- **Social and media** shows the original grouped chat, threaded comments,
  presence strip, rolling revenue and ticker. Like a comment or add a sale,
  then browse the release carousel. Zoom or pan the landscape, resize its square
  crop, drag the before-and-after divider, and swipe destination cards to save
  or skip them.
- **AI** shows the original reasoning, tool calls, agent plan, citations, source
  cards and composer. Use a suggestion to fill the draft, then send or stop
  the simulated response. A standalone model picker follows the conversation.
- **Voice** shows an incoming voice note, the recorder, a scripted call and
  live and seekable waveforms. The synthetic levels make it possible to record,
  cancel, send and play demo takes without microphone access. The legacy orb
  follows the original examples.
- **Feedback** shows CUI's four original alerts, including dismissible storage
  warning, upload percentage, indeterminate indexing, dithered export and sync,
  Storage usage and three spinner sizes. Loading reproduces the native skeleton avatar, percentage text
  lines and body placeholder, with CUI-owned shimmer and reduced-motion behavior.
  Activity reproduces the three native deployment, review and
  failed-build entries. Goals shows the native 104-point Onboarding and Quota rings.
  The native grid reflows the Progress, Loading, Activity and Goals cards.
  CUI distributes space around the Goals rings. Badges and avatars shows
  tones, presence and the overlapping team with its overflow count. The
  empty-state New project action shows a success toast. Leaving the page
  releases native dismissal and interpolation state.
  Loading includes the shared dither skeleton group. Dither spinners shows the
  original native styles and the Upload example, which alternates success and
  failure.
- **Reader** shows the original article, nested outline entries and scroll
  fades. Click an entry or activate a focused row with Enter to jump to its
  heading. The marker follows the reader while scrolling.
- **Disclosure** shows clamped About text and Billing and Security expandable
  cards. The model owns header expansion;
  CUI owns the text link's local state. The original branch confirmation uses
  the danger action and Cancel button. All examples retain CUI rendering and
  interaction.
  Hover over @jonas to see the original profile card with initials and the
  online dot. Move onto the card to keep it open, or press Escape to close it.
  Use the Veil examples to reveal a picture, a sample identifier or a spoiler.
  The announcement can show its release-notes toast, dismiss and reopen. The
  standard and compact Countdown examples share a clock. Onboarding checklist
  starts at one completed step and opens the next action as each step completes.
  Archive 3 conversations shows the original timed Undo toast.
- **Actions** shows the primary Save and outline Export split buttons.
  Save reports its main or menu action beside the buttons.
  The Toggle group section shows independent formatting icons and exclusive
  text alignment, with the original Bold and Left defaults.
  Delete project demonstrates two-step confirmation and shows Project deleted
  beside the buttons after confirmation.
  Activate twice within three seconds. Escape or a click outside cancels.
  Save changes shows inline work feedback. It runs for 1.4 seconds, displays
  Saved for 1.6 seconds, and returns to idle.
  Reactions starts with four native icon pills and animates changes to their counts.
  The Theme section places the light/dark/System switcher beside the sun/moon
  toggle. The toggle proposes an explicit opposite light/dark
  choice, including when System was selected, and retains focus after changing.
  User menu opens the original account popup with Profile, Billing, Settings
  and Log out as preview commands.
  Swipe actions shows the three original task rows and the Done, Archive and
  Delete notifications. The Create speed dial displays the chosen action.
- **Pickers** opens with the original searchable Languages multi-select,
  with C, Rust and Swift selected. Date range shows the original presets
  picker and the Stay dates picker linked to the two-month inline calendar.
  Time shows the original 24-hour and 12-hour fields. Wheels and dial shows
  the three joined date wheels and the analog clock face. Ruler and histogram
  shows the Height and Temperature tapes and the Price range over 24 bins.
- **Smart inputs** reproduces the complete original page: inline roadmap and
  owner editing, Opacity/Radius/Blur scrubbing, USD amount and whole-euro salary,
  URL/weight affixes, palette/search shortcut recording and the message's emoji
  popup. Inline text commits on Enter or blur and cancels on Escape. Scrubs
  support native dragging, Shift precision, arrows and typing. Money groups
  digits at rest. Recorders capture, cancel and clear native key combinations.
  Emoji selections append to the message and retain recent emoji and skin tone.
  The page uses the native gallery's flat container structure and clipping,
  including the hanging inline text. Targeted content captures match CUI at
  the same theme, viewport, scale and renderer.
- **Charts** reproduces the original line/area chart, grouped bars, responsive
  donut, stacked dither areas, storage pattern meter, stacked bars and CPU/Memory
  gauges. Hover to inspect native values; Tab and arrows inspect meter parts.
  Grayscale changes only the meter's palette. Native CUI owns drawing,
  layout, legends, animation and local interaction.
- **More charts** reproduces the complete original page: weekday/hour heatmap,
  yearly activity, module treemap, release radar, signup funnel, frame-time split
  bar, platform waffle, Sankey, ranked pages with Shuffle, activity rings, bubble
  chart, three radial instruments and 2,000-period price chart. CUI owns layout,
  labels, hover, animation and chart interaction. Wheel zooms the price chart;
  dragging or Shift+wheel pans. Focus it for arrows, Home/End, plus/minus and zero.
  Sample data follows the original native formulas and seed. Shuffle changes
  only model amounts and retains native label-keyed rank animation.
- **Capture** shows the original tall dropzone, two compact targets,
  Maya Okafor's profile photo, and the signature pad. Click to browse or
  drop an accepted file. The avatar shows the selected image. Draw on the pad
  and use its native Clear action to start over.
- **Audio** shows the original Synth card with Cutoff, Resonance, Attack and Pan
  and a stereo master meter. The six mixer faders show stereo peaks and Kick
  carries the original dB marks. The demo signal runs while Audio is selected.
  The two cards wrap when the available width cannot fit them side by side.
  Take the tour opens the original three native spotlight steps for Cutoff,
  the master meter and Bass. CUI owns the card and spotlight; Back, Next, Done,
  close and Escape propose the model's next step. Reopening starts at the first
  step. The gallery verifies pointer navigation and focused Enter activation.
- **Layout** supplements the original gallery with native overlapping children,
  independent sizes, per-edge padding, mirrored LTR/RTL rows, flexible/fixed
  spacing and a horizontally scrolling card strip.
- The sidebar changes pages and resets the content position even when selecting
  the active destination. CUI centers and bounds the content column and applies
  its individual edge insets. At narrow widths, open it with the **Components**
  icon. Selecting a page dismisses the drawer.
  Theme segments use Tab, Left/Right, Enter, Space or the pointer; selection
  updates the model and its shared palette.

The GUI gallery contract runs this view with both native renderers and captures
the accepted pages and open overlays, including Data display plus light, dark and narrow
Disclosure and Feedback views. It drives the widgets with native pointer and keyboard input.
The same contract also drives the package-only consumer.

## Source files

- `examples/ui/gui/gallery/App/`
  - [State.fpas](App/State.fpas)
  - [Update.fpas](App/Update.fpas)
  - [View.fpas](App/View.fpas)
- `examples/ui/gui/gallery/`
  - [GuiWidgetGallery.fpasproj](GuiWidgetGallery.fpasproj)
  - [main.fpas](main.fpas)
- `examples/ui/gui/gallery/Shared/`
  - [Sections.fpas](Shared/Sections.fpas)
  - [State.fpas](Shared/State.fpas)
  - [Update.fpas](Shared/Update.fpas)
