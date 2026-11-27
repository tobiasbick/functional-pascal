# CUI gallery comparison

The CUI layout integration review is complete.

## Gallery pages

Record the CUI reference commit and link to the comparison evidence for each page.
For pixel comparisons, use the same platform, renderer, window size, display scale,
theme and widget state.

| CUI gallery page | FPAS GUI gallery page | Layout | Pixel match | Behavior | Example coverage | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| [Overview](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L128) | [Overview](../../ui/gui/overview/README.md) | Verified at three widths | Exact for the page body with matching sample text | Verified | All examples present | [Overview evidence](#overview-evidence) |
| [Buttons](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L182) | [Buttons](../../ui/gui/actions/README.md#buttons-gallery-page) | Native cards and rows verified at three widths | Exact for the page body with closed and open menus | Verified | All six sections and shared gallery controls | [Buttons evidence](#buttons-evidence) |
| [Inputs](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L262) | [Inputs](../../ui/gui/inputs/README.md#inputs-gallery-page) | Native cards and responsive grids at three widths | 20 exact comparisons with matching email. Two edited views differ at the caret. | Verified | All three sections and 14 examples present | [Inputs evidence](#inputs-evidence) |
| [Selection](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L303) | [Selection](../../ui/gui/selection/README.md#selection-gallery-page) | Native cards and responsive grids at six widths | Exact for the settled page body | Verified | All six sections and eight widget families | [Selection evidence](#selection-evidence) |
| [Navigation](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L354) | [Navigation](../../ui/gui/navigation/README.md#navigation-gallery-page) | Native cards and responsive controls at six widths | Exact for the settled page body | Verified with stable Finish focus in FPAS | All three sections and both tab styles | [Navigation evidence](#navigation-evidence) |
| [Overlays](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L388) | [Overlays](../../ui/gui/overlays/README.md#overlays-gallery-page) | Native cards and popup placement at six widths | Exact for the initial page body, open Filters and context menu | Verified with working Apply in FPAS | All four sections and their overlays | [Overlays evidence](#overlays-evidence) |
| [Feedback](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L448) | [Feedback](../../ui/gui/feedback/README.md#feedback-gallery-page) | Native cards and two responsive grids at six widths | Exact outside the moving Indexing bar in 24 captures | Alert dismissal, upload cycle and project toast | All Feedback examples | [Feedback evidence](#feedback-evidence) |
| [Data display](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L568) | [Data display](../../ui/gui/data-display/README.md#data-display-gallery-page) | Native table, responsive grid and direct task-row children | Exact for the page body at six widths in both themes | Sorting, selection, drag and drop, expansion and invoice scrolling | All Data display examples | [Data display evidence](#data-display-evidence) |
| [Charts](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_ui.c#L700) | [Charts](../../ui/gui/charts/README.md#charts-gallery-page) | Native cards and two responsive grids | Exact for the page body in 32 captures | Chart hover, meter keys and persistent Grayscale | All eight Charts examples | [Charts evidence](#charts-evidence) |
| [Actions](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L19) | [Actions](../../ui/gui/actions/README.md#actions-gallery-page) | Native wrapping rows, swipe card and fixed dial row | Exact for the page body in 34 captures | Menus, toggles, confirmation, save, reactions and swipe | All eight Actions sections | [Actions evidence](#actions-evidence) |
| [Smart inputs](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L114) | [Smart inputs](../../ui/gui/smart-inputs/README.md#smart-inputs-gallery-page) | Native columns, wrapping rows and anchored picker | Exact for the page body in 29 captures | Editing, scrubbing, recording and emoji selection | All six Smart inputs sections | [Smart inputs evidence](#smart-inputs-evidence) |
| [Pickers](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L171) | [Pickers](../../ui/gui/pickers/README.md#pickers-gallery-page) | Native wrapping rows, calendar and wheel cards, and slider layout | Exact for the page body in 32 Linux captures | Search, date and time selection, wheel, dial and pointer sliders | All five sections and eight widget families | [Pickers evidence](#pickers-evidence) |
| [Capture](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L231) | [Capture](../../ui/gui/capture/README.md#capture-gallery-page) | Native wrapping row, avatar row and signature box | Exact in 23 captures. Live stroke thickness varies with frame timing. | File drops, avatar replacement and signature input | All three sections and five examples | [Capture evidence](#capture-evidence) |
| [Disclosure](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L261) | [Disclosure](../../ui/gui/disclosure/README.md#disclosure-gallery-page) | Native rows, cards and veil panel | Exact in 28 of 36 captures | Announcement, popup, expansion, checklist, toast and veil | The announcement and eight sections | [Disclosure evidence](#disclosure-evidence) |
| [Reader](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L369) | [Reader](../../ui/gui/reader/README.md#reader-gallery-page) | Native article viewport and fixed-width outline | Exact in 24 captures | Pointer and keyboard jumps, scroll progress and active entry | All eight outline entries | [Reader evidence](#reader-evidence) |
| [Data grid](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L414) | [Data grid](../../ui/gui/data-grid/README.md#data-grid-gallery-page) | Native filter bar, grid and source tree table | Exact in 31 of 32 captures | Sorting, resizing, selection, editing and disclosure | All three original sections | [Data grid evidence](#data-grid-evidence) |
| [Code](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L519) | [Code](../../ui/gui/code/README.md#code-gallery-page) | Native code blocks, equal-width pair and JSON views | 25 exact capture pairs | Copy, horizontal scroll and JSON folding | All five original examples | [Code evidence](#code-evidence) |
| [Social and media](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L581) | [Social and media](../../ui/gui/social-and-media/README.md#social-and-media-gallery-page) | Native cards, weighted columns and wrapping media row | Exact in 20 of 27 capture pairs | Likes, sales, slides, image controls and card dismissal | All nine original sections | [Social and media evidence](#social-and-media-evidence) |
| [Planning](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L731) | [Planning](../../ui/gui/planning/README.md#planning-gallery-page) | Native board, week calendar, Gantt chart and month calendar | Exact in 22 of 30 capture pairs | Card movement, appointment and task editing, date selection | All four original sections | [Planning evidence](#planning-evidence) |
| [More charts](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L812) | [More charts](../../ui/gui/more-charts/README.md#more-charts-gallery-page) | Native cards, weighted rows and wrapping gauges | Exact in 31 of 35 capture pairs | Chart hover, candlestick navigation and Shuffle | All 13 original chart types | [More charts evidence](#more-charts-evidence) |
| [Motion](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L945) | [Motion](../../ui/gui/motion/README.md#motion-gallery-page) | Cards, dock and weighted columns at eight widths | Light theme matches outside loader regions, with one edge pixel at three widths. Dark theme also shows the native resting-shadow difference after a theme change. | Replay, hover, transport, seek, tile toggles and tab selection | All original sections | [Motion evidence](#motion-evidence) |
| [AI](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1035) | [AI](../../ui/gui/ai/README.md#ai-gallery-page) | Two weighted columns and native conversation controls at eight widths | Exact in all 16 settled captures | Disclosure, voting, draft, model selection and response lifecycle | All original sections | [AI evidence](#ai-evidence) |
| [Audio](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1119) | [Audio](../../ui/gui/audio/README.md#audio-gallery-page) | Synth and mixer cards wrap at eight tested widths | Exact in 16 settled captures with the same signal phase | Knob and fader input, metering, and tour navigation | All original controls and tour steps | [Audio evidence](#audio-evidence) |
| [Files](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1277) | [Files](../../ui/gui/files/README.md#files-gallery-page) | Shared browser, dialog row, project table and terminal | Exact in 13 of 16 settled captures. Three captures differ at borders. | Native file navigation, project tree and terminal input | All original sections | [Files evidence](#files-evidence) |
| [Developer tools](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1510) | [Developer tools](../../ui/gui/developer-tools/README.md#developer-tools-gallery-page) | Native widget layout and section widths | Exact in all 16 settled views | Keyboard, pointer and editor input | All ten original sections | [Developer tools evidence](#developer-tools-evidence) |
| [More developer tools](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1684) | [More developer tools](../../ui/gui/more-developer-tools/README.md#more-developer-tools-gallery-page) | Native widget layout and section widths | Exact in 16 settled views with the same sample text | All 18 input captures match | All ten original sections | [More developer tools evidence](#more-developer-tools-evidence) |
| [Instruments](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1716) | [Instruments](../../ui/gui/instruments/README.md) | Wrapping readouts and a shared chart axis | Exact in all 16 settled views | Eight of nine input captures match. FPAS shows 0 for an empty track. | All three original sections | [Instruments evidence](#instruments-evidence) |
| [Creative controls](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1765) | [Creative controls](../../ui/gui/creative-controls/README.md) | Wrapping tags and two responsive grids | Exact in all 16 settled views | All 24 input captures match | All five original sections | [Creative controls evidence](#creative-controls-evidence) |
| [Canvas and bars](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1877) | [Canvas and bars](../../ui/gui/canvas-and-bars/README.md) | Grouped toolbar, layered canvas and wrapping result row | Exact in 16 settled views with matching cursor positions | 28 comparison steps match. Fit differs for a node moved beyond the original bounds. | All four original sections | [Canvas and bars evidence](#canvas-and-bars-evidence) |
| [Voice](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L2021) | [Voice](../../ui/gui/voice/README.md) | Capped columns, native recorder and voice session card | Exact in 16 settled views at the same demo time | All 17 captures after input or clock steps match | All three original sections | [Voice evidence](#voice-evidence) |

## CUI functions

Record each reviewed `cui_*` function once. Link to the gallery pages that exercise it.

| CUI function | FPAS GUI API | Integration status | Gallery pages | Evidence |
| --- | --- | --- | --- | --- |
| [`cui_accent_name`](../../ui/gui/actions/theme-switcher.md) | `Themes.Accent` and localized labels | Adopted | Buttons shell | Name each accent swatch |
| [`cui_accordion_begin`](../../ui/gui/data-display/accordion.md) | `Accordions.Accordion` | Adopted | Data display | Open or close an independent FAQ entry |
| [`cui_accordion_end`](../../ui/gui/data-display/accordion.md) | `Accordions.Accordion` | Adopted | Data display | End the body of an expanded entry |
| [`cui_activity_rings`](../../ui/gui/more-charts/activity-rings.md) | `ActivityRings.ActivityRings` | Adopted | More charts | Show three targets, including progress above one |
| [`cui_agent_plan`](../../ui/gui/ai/agent-plan.md) | `AgentPlans.AgentPlan` | Adopted | AI | Show five steps and their current progress |
| [`cui_agent_status`](../../ui/gui/ai/agent-status.md) | `AgentStatuses.AgentStatus` | Adopted | AI | Show the busy report-reading status |
| [`cui_alert`](../../ui/gui/feedback/alert.md) | `Alerts.Alert` | Adopted | Feedback | Dismiss the storage warning and show four tones |
| [`cui_api_explorer`](../../ui/gui/more-developer-tools/api-explorer.md) | `ApiExplorers.ApiExplorer` | Adopted | More developer tools | Select endpoints and read parameters |
| [`cui_aspect_picker`](../../ui/gui/creative-controls/aspect-ratio-picker.md) | `AspectPickers.AspectPicker` | Adopted | Creative controls | Choose an aspect ratio from native shapes |
| [`cui_avatar`](../../ui/gui/disclosure/avatar.md) | `Avatars.Avatar` | Adopted | Overview, Feedback, Data display, Disclosure, Data grid, Planning | Team initials and presence indicators |
| [`cui_avatar_group`](../../ui/gui/feedback/avatar-group.md) | `AvatarGroups.AvatarGroup` | Adopted | Feedback | Show four team members and an overflow count |
| [`cui_avatar_upload`](../../ui/gui/capture/avatar-upload.md) | AvatarUploads.AvatarUpload | Adopted | Capture | Replace a profile photo after an accepted image drop |
| [`cui_badge`](../../ui/gui/feedback/badge.md) | `Badges.Badge` | Adopted | Overview, Buttons shell, Feedback, Data display, Data grid, Planning | Team roles and the sidebar version |
| [`cui_banner`](../../ui/gui/disclosure/banner.md) | Banners.Banner | Adopted | Disclosure | Show, dismiss and reopen an announcement |
| [`cui_bar_chart`](../../ui/gui/charts/bar-chart.md) | `BarCharts.BarChart` | Adopted | Charts | Grouped visits and stacked test-run bars |
| [`cui_bar_list`](../../ui/gui/more-charts/bar-list.md) | `BarLists.BarList` | Adopted | More charts | Rank the six page amounts by value |
| [`cui_blame`](../../ui/gui/more-developer-tools/blame.md) | `Blames.Blame` | Adopted | More developer tools | Show source lines with commit attribution |
| [`cui_box`](../../ui/gui/layout/container-options.md) | `Elements.Row/Column`, `Stacks.Stack`, `Grids.Grid` | Adopted | Overview, Inputs, Buttons shell, Selection, Feedback, Data display, Charts, Actions, Smart inputs, Pickers, Capture, Disclosure, Code, Social and media, Motion, Creative controls, Canvas and bars | Responsive grids and the layered sidebar brand |
| [`cui_breadcrumb`](../../ui/gui/navigation/breadcrumb.md) | `Breadcrumbs.Breadcrumb` | Adopted | Navigation | Select an ancestor and show its caption in a toast |
| [`cui_button`](../../ui/gui/actions/button.md) | `Buttons.Button` | Adopted | Overview, Buttons, Navigation, Overlays, Feedback, Disclosure, Social and media, More charts, Motion, Audio, Files, Creative controls | Workspace actions and the variant, size, icon, state and group examples |
| [`cui_button_group`](../../ui/gui/actions/button-group.md) | `ButtonGroups.ButtonGroup` | Adopted | Buttons | Bold, Italic and Underline share one frame |
| [`cui_calendar_heatmap`](../../ui/gui/more-charts/calendar-heatmap.md) | `CalendarHeatmaps.CalendarHeatmap` | Adopted | More charts | Show 364 daily activity values |
| [`cui_candlestick`](../../ui/gui/more-charts/candlestick.md) | `Candlesticks.Candlestick` | Adopted | More charts | Inspect the 2,000 price and volume periods |
| [`cui_canvas`](../../ui/gui/text-and-display/canvas.md) | `Canvases.Canvas` | Adopted | Buttons shell | Paint and activate each accent swatch |
| [`cui_canvas_controls`](../../ui/gui/canvas-and-bars/canvas-controls.md) | `CanvasControls.CanvasControls` | Adopted | Canvas and bars | Zoom, fit and toggle canvas tools |
| [`cui_card`](../../ui/gui/feedback/card.md) | `Cards.Card` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays, Feedback, Data display, Charts, Actions, Pickers, Social and media, More charts, Motion, Audio, Voice | Workspace, Team and example cards |
| [`cui_card_stack_begin`](../../ui/gui/social-and-media/card-stack.md) | CardStacks.CardStack | Adopted | Social and media | Start the destination deck |
| [`cui_card_stack_end`](../../ui/gui/social-and-media/card-stack.md) | CardStacks.CardStack | Adopted | Social and media | Report a dismissal direction |
| [`cui_card_stack_item`](../../ui/gui/social-and-media/card-stack.md) | CardStacks.CardStack | Adopted | Social and media | Compose the three destination cards |
| [`cui_carousel`](../../ui/gui/social-and-media/carousel.md) | Carousels.Carousel | Adopted | Social and media | Navigate the four release slides |
| [`cui_carousel_slide`](../../ui/gui/social-and-media/carousel.md) | Carousels.Carousel | Adopted | Social and media | Compose each release slide |
| [`cui_chat_composer`](../../ui/gui/voice/composer.md) | `Composers.Composer` | Adopted | AI, Voice | Edit drafts, manage attachments and models, and record or send a voice take |
| [`cui_chat_divider`](../../ui/gui/social-and-media/chat-divider.md) | ChatDividers.ChatDivider | Adopted | Social and media | Separate the conversation by date |
| [`cui_chat_message`](../../ui/gui/social-and-media/chat-message.md) | ChatMessages.ChatMessage | Adopted | Social and media, AI | Show grouped bubbles and a receipt |
| [`cui_checkbox`](../../ui/gui/selection/checkbox.md) | `Checkboxes.Checkbox` | Adopted | Overview, Selection, Overlays, Charts, Canvas and bars | Summary preference also updates the disabled example in Selection |
| [`cui_checklist`](../../ui/gui/disclosure/onboarding-checklist.md) | Checklists.Checklist | Adopted | Disclosure | Toggle completion and advance to the next step |
| [`cui_chip`](../../ui/gui/selection/chip.md) | `Chips.Chip` | Adopted | Selection | Independent category choices wrap inside the card |
| [`cui_citation`](../../ui/gui/ai/citation.md) | `Citations.Citation` | Adopted | AI | Preview each numbered source |
| [`cui_code_block`](../../ui/gui/code/code-block.md) | CodeBlocks.CodeBlock | Adopted | Code | Highlight and copy the C, Python and shell examples |
| [`cui_color_of`](../../ui/gui/canvas-and-bars/painting.md) | `Themes.ColorToken` and `Painting.Surface.Color` | Adopted | Buttons shell, Canvas and bars | Use the active CUI palette for swatches during rendering |
| [`cui_color_picker`](../../ui/gui/inputs/color-picker.md) | `ColorPickers.ColorPicker` | Adopted | Inputs | Brand color swatch |
| [`cui_column`](../../ui/gui/layout/column.md) | `Elements.Column` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays, Feedback, Data display, Charts, Actions, Smart inputs, Pickers, Capture, Disclosure, Reader, Data grid, Code, Social and media, Planning, More charts, Motion, AI, Files, Developer tools, More developer tools, Instruments, Creative controls, Canvas and bars, Voice | Section headings, form captions and field stacks |
| [`cui_command_palette`](../../ui/gui/overlays/command-palette.md) | `CommandPalettes.CommandPalette` | Adopted | Overview, Buttons shell, Overlays | Search and select a gallery command with Primary+K |
| [`cui_comment`](../../ui/gui/social-and-media/comment.md) | Comments.Comment | Adopted | Social and media | Show a thread with likes and replies |
| [`cui_commit_graph`](../../ui/gui/developer-tools/commit-graph.md) | `CommitGraphs.CommitGraph` | Adopted | Developer tools | Draw and select ten commits across branch lanes |
| [`cui_confirm_button`](../../ui/gui/actions/confirm-button.md) | `ConfirmButtons.ConfirmButton` | Adopted | Actions | Two-step Delete project confirmation |
| [`cui_confirm_popover`](../../ui/gui/disclosure/confirm-popover.md) | ConfirmPopovers.ConfirmPopover | Adopted | Disclosure | Confirm or cancel the branch preview |
| [`cui_context_menu`](../../ui/gui/overlays/context-menu.md) | `Menus.ContextMenu` | Adopted | Overlays | Right-click the muted card to open commands |
| [`cui_context_meter`](../../ui/gui/ai/context-meter.md) | `ContextMeters.ContextMeter` | Adopted | AI | Show 57 percent usage and the three context parts |
| [`cui_control_tile`](../../ui/gui/motion/control-tile.md) | `ControlTiles.ControlTile` | Adopted | Motion | Toggle Wi-Fi, Hotspot, Focus, AirShare and Dark appearance |
| [`cui_copy_button`](../../ui/gui/actions/copy-button.md) | `CopyButtons.CopyButton` | Adopted | Buttons | Copy the displayed install command |
| [`cui_countdown`](../../ui/gui/disclosure/countdown.md) | Countdowns.Countdown | Adopted | Disclosure | Display standard and compact countdowns |
| [`cui_counter`](../../ui/gui/social-and-media/counter.md) | Counters.Counter | Adopted | Social and media | Count sales with grouped thousands |
| [`cui_coverage`](../../ui/gui/more-developer-tools/coverage.md) | `Coverages.Coverage` | Adopted | More developer tools | Fold folders and sort coverage totals |
| [`cui_cropper`](../../ui/gui/social-and-media/image-cropper.md) | ImageCroppers.ImageCropper | Adopted | Social and media | Adjust the square crop |
| [`cui_data_grid_begin`](../../ui/gui/data-grid/data-grid.md) | DataGrids.DataGrid | Adopted | Data grid | Lay out a virtual grid and realize rows in the viewport |
| [`cui_data_grid_cell`](../../ui/gui/data-grid/data-grid.md) | DataGrids.Cell.Content | Adopted | Data grid | Compose badge and score cells with native widgets |
| [`cui_data_grid_cell_text`](../../ui/gui/data-grid/data-grid.md) | DataGrids.Cell.Text | Adopted | Data grid, Files | Display email and numeric cells as native text |
| [`cui_data_grid_edit`](../../ui/gui/data-grid/data-grid.md#interaction) | DataGrids.Cell.Editable | Adopted | Data grid | Edit a name next to its avatar |
| [`cui_data_grid_end`](../../ui/gui/data-grid/data-grid.md) | DataGrids.DataGrid | Adopted | Data grid | Complete the viewport and publish the native proposal |
| [`cui_data_grid_row`](../../ui/gui/data-grid/data-grid.md) | DataGrids.Row | Adopted | Data grid | Build a visible row by display position |
| [`cui_date_from_days`](../../ui/gui/planning/README.md#dates) | `DateOnly` and `AddDays` | CLR date arithmetic | Planning, More charts | Resolve a Monday, a timeline origin, and appointment dates |
| [`cui_date_picker`](../../ui/gui/inputs/date-picker.md) | `DatePickers.DatePicker` | Adopted | Inputs | Due date and its calendar popup |
| [`cui_date_range_picker`](../../ui/gui/pickers/date-range-picker.md) | `DateRangePickers.DateRangePicker` | Adopted | Pickers | Select dates with or without presets |
| [`cui_date_to_days`](../../ui/gui/planning/README.md#dates) | `DateOnly.DayNumber` | CLR date arithmetic | Planning, More charts | Convert dates to ordinal day numbers |
| [`cui_date_today`](../../ui/gui/planning/README.md#dates) | `DateTime.Today` | CLR date arithmetic | Planning, More charts | Read the local calendar day |
| [`cui_delta_time`](../../ui/gui/motion/README.md#playback-clock) | `Canvas.Painter.DeltaTime` | Adopted | Motion | Read native frame delta in a canvas. Motion uses its own playback clock. |
| [`cui_dialog`](../../ui/gui/overlays/dialog.md) | `Dialogs.Dialog` | Adopted | Buttons, Overlays | Options > Delete opens the project confirmation |
| [`cui_dialog_footer`](../../ui/gui/overlays/action-footer.md) | `ActionFooters.ActionFooter` | Adopted | Buttons, Overlays | Cancel and Delete project actions |
| [`cui_dock`](../../ui/gui/motion/dock.md) | `Docks.Dock` | Adopted | Motion | Center the icon dock in the row |
| [`cui_dock_item`](../../ui/gui/motion/dock.md) | `Docks.Entry.Item` | Adopted | Motion | Show icons with status dots, badges and labels |
| [`cui_dock_separator`](../../ui/gui/motion/dock.md) | `Docks.Entry.Separator` | Adopted | Motion | Separate the shortcuts from Settings and Trash |
| [`cui_donut_chart`](../../ui/gui/charts/donut-chart.md) | `DonutCharts.DonutChart` | Adopted | Charts | Platform shares with slice and legend hover |
| [`cui_dot_chart`](../../ui/gui/instruments/dot-chart.md) | `DotCharts.DotChart` | Adopted | Instruments | Plot the sample curves with threshold and time annotations |
| [`cui_dot_display`](../../ui/gui/instruments/dot-display.md) | `DotDisplays.DotDisplay` | Adopted | Instruments | Display the bedtime status in a dot font |
| [`cui_draw_circle`](../../ui/gui/canvas-and-bars/painting.md) | `Painting.Surface.Circle` | Adopted | Buttons shell | Paint the accent swatch and its ring |
| [`cui_dropped_file`](../../ui/gui/capture/dropzone.md#interaction) | Dropzone accepted paths | Adopted | Capture | Show the first accepted path from the tall dropzone |
| [`cui_dropzone`](../../ui/gui/capture/dropzone.md) | Dropzones.Dropzone | Adopted | Capture | Accept files with tall and compact targets and native feedback |
| [`cui_emoji_picker`](../../ui/gui/smart-inputs/emoji-picker.md) | `EmojiPickers.EmojiPicker` | Adopted | Smart inputs | Browse, search and insert emoji with recent entries and skin tone |
| [`cui_empty_state`](../../ui/gui/feedback/empty-state.md) | `EmptyStates.EmptyState` | Adopted | Feedback | Create a project from the empty view |
| [`cui_event_track`](../../ui/gui/instruments/event-track.md) | `EventTracks.EventTrack` | Adopted | Instruments | Edit cup times on the shared time axis |
| [`cui_expandable_card`](../../ui/gui/disclosure/expandable-card.md) | ExpandableCards.ExpandableCard | Adopted | Disclosure | Open the Billing and Security details |
| [`cui_expandable_text`](../../ui/gui/disclosure/expandable-text.md) | ExpandableTexts.ExpandableText | Adopted | Disclosure | Open and close a two-line text preview |
| [`cui_fader`](../../ui/gui/audio/fader.md) | `Faders.Fader` | Adopted | Audio | Edit gain on six channel strips with stereo levels |
| [`cui_feature_flags`](../../ui/gui/more-developer-tools/feature-flags.md) | `FeatureFlagEditors.FeatureFlags` | Adopted | More developer tools | Edit environment toggles and rollout shares |
| [`cui_file_browser`](../../ui/gui/files/file-browser.md) | `FileBrowsers.FileBrowser` | Adopted | Files | Navigate the local fixture and choose a file |
| [`cui_file_picker`](../../ui/gui/files/file-picker.md) | `FilePickers.FilePicker` | Adopted | Files | Open and save files through the native modal |
| [`cui_filter_bar`](../../ui/gui/data-grid/filter-bar.md) | FilterBars.FilterBar | Adopted | Data grid | Choose filter values and enter a search |
| [`cui_format_bar`](../../ui/gui/canvas-and-bars/format-bar.md) | `FormatBars.FormatBar` | Adopted | Canvas and bars | Edit the text-format state and confirm deletion |
| [`cui_fs_list`](../../ui/gui/files/file-browser.md) | `FileListings.Source` | Adopted | Files | Use the native local listing or supply an FPAS reader |
| [`cui_funnel`](../../ui/gui/more-charts/funnel.md) | `Funnels.Funnel` | Adopted | More charts | Show the four conversion stages |
| [`cui_gantt`](../../ui/gui/planning/gantt.md) | `Gantts.Gantt` | Adopted | Planning | Move and resize tasks with days, milestones and dependencies |
| [`cui_gap`](../../ui/gui/layout/spacing.md#fixed-gaps-and-spacers) | `Elements.Gap` | Adopted | Buttons shell, Reader, More charts | Fixed space between sections of destinations |
| [`cui_gauge`](../../ui/gui/charts/gauge.md) | `Gauges.Gauge` | Adopted | Charts | CPU and memory gauges in a centered row |
| [`cui_gradient_slider`](../../ui/gui/creative-controls/gradient-slider.md) | `GradientSliders.GradientSlider` | Adopted | Creative controls | Edit hue, white balance and opacity |
| [`cui_gradient_text`](../../ui/gui/motion/gradient-text.md) | `GradientTexts.GradientText` | Adopted | Motion | Draw the animated Kiln 2.4 title |
| [`cui_graph_view`](../../ui/gui/developer-tools/dependency-graph.md) | `DependencyGraphs.DependencyGraph` | Adopted | Developer tools | Inspect and fit the dependency graph |
| [`cui_heading`](../../ui/gui/text-and-display/text.md) | `Texts.Text` with a heading appearance | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays, Feedback, Data display, Charts, Actions, Smart inputs, Pickers, Capture, Disclosure, Reader, Data grid, Code, Social and media, Planning, More charts, Motion, AI, Audio, Files, Developer tools, More developer tools, Instruments, Creative controls, Canvas and bars, Voice | Four heading levels |
| [`cui_heatmap`](../../ui/gui/more-charts/heatmap.md) | `Heatmaps.Heatmap` | Adopted | More charts | Plot requests by weekday and hour |
| [`cui_histogram_slider`](../../ui/gui/pickers/histogram-slider.md) | `HistogramSliders.HistogramSlider` | Adopted | Pickers | Adjust two price endpoints over 24 bins |
| [`cui_hold_button`](../../ui/gui/actions/hold-button.md) | `HoldButtons.HoldButton` | Adopted | Buttons | A completed hold shows the Deleted toast |
| [`cui_hover_card`](../../ui/gui/disclosure/hover-card.md) | HoverCards.HoverCard | Adopted | Disclosure | Show the profile card from the inline link |
| [`cui_icon`](../../ui/gui/text-and-display/icon.md) | `Icons.Icon` | Adopted | Overlays, Data display, Actions, Disclosure, Planning, Canvas and bars | Ellipsis in the context-menu target |
| [`cui_icon_button`](../../ui/gui/actions/icon-button.md) | `IconButtons.IconButton` | Adopted | Buttons, Overlays, Smart inputs, Motion | Copy, Share, Like and Delete icons with tooltips |
| [`cui_image`](../../ui/gui/text-and-display/image.md) | Images.Image | Adopted | Disclosure | Display the rounded landscape inside the veil |
| [`cui_image_compare`](../../ui/gui/social-and-media/image-compare.md) | ImageComparisons.ImageCompare | Adopted | Social and media | Compare the grayscale and color landscape |
| [`cui_image_create`](../../ui/gui/text-and-display/image.md#usage) | MediaImages.Source.Rgba | Managed resource | Disclosure, Social and media, Canvas and bars | Upload the generated landscape |
| [`cui_image_destroy`](../../ui/gui/text-and-display/image.md#usage) | GUI texture owner | Managed resource | Disclosure, Social and media, Canvas and bars | Release the image texture with its owner |
| [`cui_image_viewer`](../../ui/gui/social-and-media/image-viewer.md) | ImageViewers.ImageViewer | Adopted | Social and media | Zoom and pan the landscape |
| [`cui_inline_edit`](../../ui/gui/smart-inputs/inline-edit.md) | `InlineEdits.InlineEdit` | Adopted | Smart inputs | Edit and commit the title or owner in place |
| [`cui_input`](../../ui/gui/inputs/text-field.md) | `TextFields.TextField` | Adopted | Overview, Inputs, Buttons, Overlays, Smart inputs | Name and email in the profile dialog |
| [`cui_json_view`](../../ui/gui/code/json-viewer.md) | JsonViews.JsonView | Adopted | Code | Fold JSON objects and arrays and show a parse error |
| [`cui_kanban_begin`](../../ui/gui/planning/kanban.md) | `Kanbans.Kanban` | Adopted | Planning | Start the four-column board |
| [`cui_kanban_card_begin`](../../ui/gui/planning/kanban.md) | `Kanbans.Card` | Adopted | Planning | Build an interactive card with a stable ID |
| [`cui_kanban_card_end`](../../ui/gui/planning/kanban.md) | `Kanbans.Card` | Adopted | Planning | Close the composed card body |
| [`cui_kanban_column_begin`](../../ui/gui/planning/kanban.md) | `Kanbans.Column` | Adopted | Planning | Show the caption, count, color dot, width and work limit |
| [`cui_kanban_column_end`](../../ui/gui/planning/kanban.md) | `Kanbans.Column` | Adopted | Planning | Close the column after its cards |
| [`cui_kanban_end`](../../ui/gui/planning/kanban.md) | `Kanbans.Kanban` | Adopted | Planning | Report a card move to the model |
| [`cui_keymap`](../../ui/gui/developer-tools/keymap.md) | `Keymaps.Keymap` | Adopted | Developer tools | Display shortcut categories and conflicts |
| [`cui_knob`](../../ui/gui/audio/knob.md) | `Knobs.Knob` | Adopted | Audio | Adjust cutoff, resonance, attack and pan |
| [`cui_kv_editor`](../../ui/gui/developer-tools/key-value-editor.md) | `KeyValueEditors.KeyValueEditor` | Adopted | Developer tools, More developer tools | Edit the sample header rows |
| [`cui_label`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `Label` | Adopted | Inputs, Buttons, Navigation, Overlays, Capture, More charts, Canvas and bars | Form captions and the Recent section in Notifications |
| [`cui_last_id`](../../ui/gui/overlays/guided-tour.md) | `Tours.Step.Target` | Adopted | Audio | Resolve tour anchors through stable control IDs |
| [`cui_last_rect`](../../ui/gui/layout/layout-inspection.md#read-the-bounds) | `Session.Snapshot` control bounds and `CuiWindow.LastRectangle` | Adopted | Selection | Read the Bluetooth row bounds for input tests |
| [`cui_level_meter`](../../ui/gui/audio/level-meter.md) | `LevelMeters.LevelMeter` | Adopted | Audio | Draw the horizontal stereo master meter |
| [`cui_line_chart`](../../ui/gui/charts/line-chart.md) | `LineCharts.LineChart` | Adopted | Charts | Smooth revenue and stacked dither fills with crosshairs |
| [`cui_link`](../../ui/gui/disclosure/link.md) | Links.Link | Adopted | Disclosure | Activate and focus an inline link |
| [`cui_list_item`](../../ui/gui/data-display/list-item.md) | `ListItems.ListItem` | Adopted | Buttons, Overlays, Data display | Notifications with initials and a subtitle |
| [`cui_live_cursors`](../../ui/gui/canvas-and-bars/live-cursors.md) | `LiveCursors.LiveCursors` | Adopted | Canvas and bars | Draw moving collaborator pointers and messages |
| [`cui_loader`](../../ui/gui/motion/loader.md) | `Loaders.Loader` | Adopted | Motion | Draw dots, bars, pulse, orbit and grid |
| [`cui_make_id`](../../ui/gui/text-and-display/image.md#usage) | GUI texture owner | Managed resource | Social and media, Canvas and bars | Derive native texture IDs from the widget ID |
| [`cui_menu`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Menu` | Adopted | Buttons | Labeled Options and icon-only More actions |
| [`cui_menu_item`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Entry.Action` | Adopted | Buttons, Overlays, Actions, Files | Icons, shortcut captions and a danger item |
| [`cui_menu_label`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Entry.Label` | Adopted | Buttons | Project section caption |
| [`cui_menu_separator`](../../ui/gui/actions/dropdown-menu.md) | `Menus.Entry.Separator` | Adopted | Buttons, Overlays, Actions, Files | Line above Delete |
| [`cui_meter`](../../ui/gui/feedback/meter.md) | `Meters.Meter` | Adopted | Feedback | Show storage use and the warning threshold |
| [`cui_metric`](../../ui/gui/overview/metric.md) | `Metrics.Metric` | Adopted | Overview | Three tiles, including a delta and sparkline for Frame time |
| [`cui_money_input`](../../ui/gui/smart-inputs/money-field.md) | `MoneyFields.MoneyField` | Decimal adapter | Smart inputs | Edit USD and EUR amounts with native text input and CLR decimal |
| [`cui_month_calendar`](../../ui/gui/planning/month-calendar.md) | `MonthCalendars.MonthCalendar` | Adopted | Planning | Browse months, select a day and open appointments |
| [`cui_multi_select`](../../ui/gui/pickers/multi-select.md) | `MultiSelects.MultiSelect` | Adopted | Pickers | Search and change a checklist with selected chips |
| [`cui_muted`](../../ui/gui/text-and-display/text.md) | `Texts.Text` with `Appearance.Muted` | Adopted | Actions, Pickers, Capture, Disclosure, Data grid, Files, Canvas and bars | Small muted action results |
| [`cui_nav_item`](../../ui/gui/navigation/sidebar.md) | `Navigation.Item` in `Navigation.Sidebar` | Adopted | Buttons shell | Select a gallery destination |
| [`cui_nav_section`](../../ui/gui/navigation/sidebar.md) | `Navigation.Section` | Adopted | Buttons shell | Components and Extended captions |
| [`cui_node_begin`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.Node` | Adopted | Canvas and bars | Build each specimen node |
| [`cui_node_content`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.Node.Body` | Adopted | Canvas and bars | Place specimen text in native node content |
| [`cui_node_editor_begin`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.NodeEditor` | Adopted | Canvas and bars | Open the native graph with a shared viewport |
| [`cui_node_editor_end`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.NodeEditor` | Adopted | Canvas and bars | Finish and lay out the native graph |
| [`cui_node_end`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.Node` | Adopted | Canvas and bars | Finish a specimen node |
| [`cui_node_input`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.Pin` | Adopted | Canvas and bars | Add the two input pins |
| [`cui_node_link`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.Link` | Adopted | Canvas and bars | Connect the prompt and avoidance pins |
| [`cui_node_output`](../../ui/gui/canvas-and-bars/node-editor.md) | `NodeEditors.Pin` | Adopted | Canvas and bars | Add output pins to the first two nodes |
| [`cui_now_playing`](../../ui/gui/motion/now-playing.md) | `NowPlaying.NowPlaying` | Adopted | Motion | Show transport, track time, artwork and seek bar |
| [`cui_number`](../../ui/gui/inputs/number-field.md) | `NumberFields.NumberField` | Adopted | Inputs | Seats with native fill width, bounds, and step buttons |
| [`cui_option_action`](../../ui/gui/creative-controls/option-action.md) | `OptionActions.OptionAction` | Adopted | Creative controls | Choose the background or export format and invoke its action |
| [`cui_otp_input`](../../ui/gui/inputs/one-time-code.md) | `OneTimeCodes.OneTimeCodeInput` | Adopted | Inputs | Six-box verification code and completion |
| [`cui_pagination`](../../ui/gui/navigation/pagination.md) | `Paginations.Pagination` | Adopted | Navigation | Move through twelve pages with native compact layout |
| [`cui_palette_strip`](../../ui/gui/creative-controls/palette-strip.md) | `PaletteStrips.PaletteStrip` | Adopted | Creative controls | Lock a swatch and shuffle the remaining colors |
| [`cui_password_strength`](../../ui/gui/inputs/password-strength.md) | `PasswordStrengthMeters.PasswordStrengthMeter` | Adopted | Inputs | Live estimate under New password |
| [`cui_pattern_meter`](../../ui/gui/charts/pattern-meter.md) | `PatternMeters.PatternMeter` | Adopted | Charts | Storage parts, grayscale and keyboard inspection |
| [`cui_pipeline`](../../ui/gui/developer-tools/pipeline.md) | `Pipelines.Pipeline` | Adopted | Developer tools | Draw grouped stages and select jobs |
| [`cui_pop_id`](../../ui/gui/host.md#frame-preparation-and-presentation) | Host identity scopes | Host owned | Actions, Reader, Motion, Audio, Canvas and bars, Voice | Close each swipe row scope |
| [`cui_popover`](../../ui/gui/overlays/popover.md) | `Popovers.Popover` | Adopted | Overlays, Smart inputs | Show the Filters controls in an anchored panel |
| [`cui_popover_close`](../../ui/gui/overlays/popover.md#interaction) | Model visibility update | Adopted | Overlays, Smart inputs | Apply closes the Filters panel |
| [`cui_popover_open`](../../ui/gui/disclosure/confirm-popover.md) | Model-owned Open state | Adopted | Disclosure | Open the confirmation from its trigger |
| [`cui_popover_toggle`](../../ui/gui/overlays/popover.md#interaction) | Model visibility update | Adopted | Overlays, Smart inputs | The Filters button toggles the panel |
| [`cui_presence`](../../ui/gui/social-and-media/presence.md) | Presences.Presence | Adopted | Social and media | Show idle avatars and an overflow count |
| [`cui_presence_pill`](../../ui/gui/canvas-and-bars/presence-pill.md) | `PresencePills.PresencePill` | Adopted | Canvas and bars | Show collaborator avatars |
| [`cui_problems`](../../ui/gui/developer-tools/problems-panel.md) | `ProblemsPanels.Problems` | Adopted | Developer tools | Filter grouped diagnostics and navigate their rows |
| [`cui_progress`](../../ui/gui/feedback/progress-bar.md) | `ProgressBars.ProgressBar` | Adopted | Feedback, Data grid, Planning | Show determinate and indeterminate progress with plain and dither fills |
| [`cui_progress_ring`](../../ui/gui/feedback/progress-ring.md) | `ProgressRings.ProgressRing` | Adopted | Feedback | Show both goal percentages |
| [`cui_prompt_suggestions`](../../ui/gui/ai/prompt-suggestions.md) | `PromptSuggestions.PromptSuggestions` | Adopted | AI | Fill the draft from one of four cards |
| [`cui_push_id_int`](../../ui/gui/host.md#frame-preparation-and-presentation) | Stable widget IDs and host identity scopes | Host owned | Actions, Reader, Motion, Audio, Canvas and bars, Voice | Identify each swipe row independently |
| [`cui_query_builder`](../../ui/gui/developer-tools/query-builder.md) | `QueryBuilders.QueryBuilder` | Adopted | Developer tools | Edit nested groups and conditions |
| [`cui_query_text`](../../ui/gui/developer-tools/query-builder.md) | `Fpas.Ui.Gui.QueryBuilders.Description` | Adopted | Developer tools | Format the current query as text below the editor |
| [`cui_radar_chart`](../../ui/gui/more-charts/radar-chart.md) | `RadarCharts.RadarChart` | Adopted | More charts | Compare two series around six axes |
| [`cui_radial_gauge`](../../ui/gui/more-charts/radial-gauge.md) | `RadialGauges.RadialGauge` | Adopted | More charts | Show bands, a target and a ring |
| [`cui_radio`](../../ui/gui/selection/radio-button.md) | `RadioButtons.RadioButton` | Adopted | Selection | Three save-frequency choices in a card |
| [`cui_radio_cards`](../../ui/gui/selection/radio-cards.md) | `RadioCards.RadioCards` | Adopted | Selection | Choose a save destination with a description, icon and optional badge |
| [`cui_range_calendar`](../../ui/gui/pickers/date-range-calendar.md) | `DateRangeCalendars.DateRangeCalendar` | Adopted | Pickers | Link the two-month calendar to the Stay field |
| [`cui_range_slider`](../../ui/gui/selection/range-slider.md) | `RangeSliders.RangeSlider` | Adopted | Selection | Adjust the two price endpoints |
| [`cui_rating`](../../ui/gui/selection/rating.md) | `Ratings.Rating` | Adopted | Selection | Set or clear a five-star rating |
| [`cui_ratio_slider`](../../ui/gui/creative-controls/ratio-slider.md) | `RatioSliders.RatioSlider` | Adopted | Creative controls | Adjust the two labeled shares |
| [`cui_reactions`](../../ui/gui/actions/reactions.md) | `Reactions.Reactions` | Adopted | Actions | Toggle four independent reaction counts |
| [`cui_readout`](../../ui/gui/instruments/readout.md) | `Readouts.Readout` | Adopted | Instruments | Show the bedtime value, peak and cup count |
| [`cui_reasoning`](../../ui/gui/ai/reasoning.md) | `Reasonings.Reasoning` | Adopted | AI | Expand the supplied reasoning and show its duration |
| [`cui_regex_tester`](../../ui/gui/more-developer-tools/regex-tester.md) | `RegexTesters.RegexTester` | Adopted | More developer tools | Mark matches and groups in sample text |
| [`cui_reorder_begin`](../../ui/gui/data-display/reorderable-list.md) | `ReorderableLists.ReorderableList` | Adopted | Data display | Start the list with a model-owned task order |
| [`cui_reorder_end`](../../ui/gui/data-display/reorderable-list.md) | `ReorderableLists.ReorderableList` | Adopted | Data display | Save a changed drop and show the Order saved toast |
| [`cui_reorder_item`](../../ui/gui/data-display/reorderable-list.md) | `ReorderableLists.ReorderableList` | Adopted | Data display | Draw the native grip with a title and badge |
| [`cui_request_bar`](../../ui/gui/more-developer-tools/http-request-bar.md) | `RequestBars.RequestBar` | Adopted | More developer tools | Edit the HTTP method and URL |
| [`cui_request_redraw`](../../ui/gui/host.md#invalidation-and-inspection) | `Frames.Redraw` and `CuiWindow.RequestFrame` | Adopted | Actions | Request a frame while timed feedback is active |
| [`cui_request_wakeup`](../../ui/gui/canvas-and-bars/painting.md) | `Fpas.Ui.Gui.Frames.Redraw` and the MVU session clock | Adopted | Canvas and bars, Voice | Advance the collaborator poses, recordings and voice timeline |
| [`cui_response_actions`](../../ui/gui/ai/response-actions.md) | `ResponseActions.ResponseActions` | Adopted | AI | Copy the sample answer, vote and regenerate |
| [`cui_response_status`](../../ui/gui/more-developer-tools/http-response-status.md) | `ResponseStatuses.ResponseStatus` | Adopted | More developer tools | Show the sample HTTP result |
| [`cui_result_card`](../../ui/gui/canvas-and-bars/result-card.md) | `ResultCards.ResultCard` | Adopted | Canvas and bars | Show the result image, badge and caption |
| [`cui_row`](../../ui/gui/layout/row.md) | `Elements.Row` | Adopted | Overview, Buttons, Selection, Navigation, Overlays, Feedback, Data display, Actions, Smart inputs, Pickers, Capture, Disclosure, Reader, Code, Social and media, Planning, More charts, Motion, AI, Audio, Files, Instruments, Creative controls, Canvas and bars | Wrapping actions, padded team rows and the copy command |
| [`cui_ruler_slider`](../../ui/gui/pickers/ruler-slider.md) | `RulerSliders.RulerSlider` | Adopted | Pickers | Adjust height and temperature with the native tapes |
| [`cui_sankey`](../../ui/gui/more-charts/sankey.md) | `Sankeys.Sankey` | Adopted | More charts | Follow eight flows between conversion nodes |
| [`cui_scatter_chart`](../../ui/gui/more-charts/scatter-chart.md) | `ScatterCharts.ScatterChart` | Adopted | More charts | Plot frame times and node counts with bubble sizes |
| [`cui_schedule_picker`](../../ui/gui/developer-tools/schedule-picker.md) | `SchedulePickers.SchedulePicker` | Adopted | Developer tools | Edit the cron expression and preview the next runs |
| [`cui_scroll`](../../ui/gui/layout/scroll-view.md) | `Elements.ScrollView` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays, Reader | Shared scrolling content area |
| [`cui_scroll_to`](../../ui/gui/layout/scroll-view.md#request-a-scroll-offset) | `Scrolls.To` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays | Sidebar selection moves the main viewport to the start with native scroll motion |
| [`cui_scrub`](../../ui/gui/smart-inputs/scrub-field.md) | `ScrubFields.ScrubField` | Adopted | Smart inputs | Drag, fine-adjust or type the three bounded values |
| [`cui_search`](../../ui/gui/inputs/search-field.md) | `SearchFields.SearchField` | Adopted | Inputs | Search projects with native clear action |
| [`cui_secondary`](../../ui/gui/text-and-display/text.md) | `Elements.Styled` with `Secondary` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays, Feedback, Data display, Charts, Actions, Smart inputs, Pickers, Capture, Disclosure, Data grid, Code, Social and media, Planning, More charts, Motion, AI, Files, Developer tools, More developer tools, Instruments, Creative controls, Canvas and bars, Voice | Card and section descriptions |
| [`cui_seed_field`](../../ui/gui/creative-controls/seed-field.md) | `SeedFields.SeedField` | Adopted | Creative controls | Roll a seed and toggle its lock |
| [`cui_segmented`](../../ui/gui/actions/segmented-control.md) | `SegmentedControls.SegmentedControl` | Adopted | Buttons | View choices with icons and a small date range selector |
| [`cui_select`](../../ui/gui/inputs/select.md) | `Selects.Select` | Adopted | Inputs | Country and searchable Language lists |
| [`cui_selection_bar`](../../ui/gui/canvas-and-bars/selection-bar.md) | `SelectionBars.SelectionBar` | Adopted | Canvas and bars | Offer actions for the selected files |
| [`cui_selection_ring`](../../ui/gui/canvas-and-bars/selection-ring.md) | `SelectionRings.Options` | Adopted | Canvas and bars | Mark a node with its collaborator label |
| [`cui_separator`](../../ui/gui/layout/divider.md) | `Elements.Divider` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays | Desktop sidebar divider |
| [`cui_set_accent`](../../ui/gui/host.md#system-appearance) | `Themes.Light/Dark` with an accent | Adopted | Buttons shell, Overlays | Swatch clicks and the palette change the accent |
| [`cui_set_clipboard_text`](../../ui/gui/host.md#clipboard) | `Clipboard.WriteText` | Adopted | Files | Copy a project node name through the GUI host |
| [`cui_set_theme`](../../ui/gui/host.md#system-appearance) | `Desktop` theme provider | Adopted | Overview, Buttons shell, Overlays | Primary+J and the theme selector switch appearance |
| [`cui_sheet`](../../ui/gui/overlays/sheet.md) | `Sheets.Sheet` | Adopted | Buttons shell, Overlays | Compact navigation drawer and Notifications |
| [`cui_shimmer_text`](../../ui/gui/motion/shimmer-text.md) | `ShimmerTexts.ShimmerText` | Adopted | Motion | Draw the highlighted waiting message |
| [`cui_shortcut`](../../ui/gui/host.md#application-shortcuts) | `Shortcuts.Binding` and the native palette shortcut | Adopted | Overview, Buttons shell | Primary+J and Primary+K |
| [`cui_shortcut_recorder`](../../ui/gui/smart-inputs/shortcut-recorder.md) | `ShortcutRecorders.ShortcutRecorder` | Adopted | Smart inputs | Record, cancel and clear a key combination |
| [`cui_signature_pad`](../../ui/gui/capture/signature-pad.md) | SignaturePads.SignaturePad | Adopted | Capture | Draw ink strokes on a 520-unit pad and clear the ink |
| [`cui_skeleton`](../../ui/gui/feedback/skeleton.md) | `Skeletons.Skeleton` | Adopted | Feedback | Use fixed, fill and percentage widths |
| [`cui_skeleton_group`](../../ui/gui/feedback/skeleton-group.md) | `SkeletonGroups.SkeletonGroup` | Adopted | Feedback | Share the dither sweep across loading placeholders |
| [`cui_slider`](../../ui/gui/selection/slider.md) | `Sliders.Slider` | Adopted | Selection, Overlays | Continuous volume and stepped brightness with a percent format |
| [`cui_source_list`](../../ui/gui/ai/source-list.md) | `SourceLists.SourceList` | Adopted | AI | Show the three sources in a scrollable row |
| [`cui_spacer`](../../ui/gui/layout/spacing.md#fixed-gaps-and-spacers) | `Elements.Spacer()` | Adopted | Buttons shell, Planning, AI | Flexible space before the version badge |
| [`cui_speed_dial`](../../ui/gui/actions/speed-dial.md) | `SpeedDials.SpeedDial` | Adopted | Actions | Open Create and display the chosen action |
| [`cui_spinner`](../../ui/gui/feedback/spinner.md) | `Spinners.Spinner` | Adopted | Feedback | Show the default arc, dither styles and themed inline spinners |
| [`cui_split_bar`](../../ui/gui/more-charts/split-bar.md) | `SplitBars.SplitBar` | Adopted | More charts | Divide time into five proportional sections |
| [`cui_split_button`](../../ui/gui/actions/split-button.md) | `SplitButtons.SplitButton` | Adopted | Actions | Save and Export with a related command menu |
| [`cui_spotlight`](../../ui/gui/motion/spotlight-card.md) | `Spotlights.Spotlight` | Adopted | Motion | Draw three flexible cards with a beam on the middle card |
| [`cui_stack_trace`](../../ui/gui/more-developer-tools/stack-trace.md) | `StackTraces.StackTrace` | Adopted | More developer tools | Fold library groups and inspect source |
| [`cui_state`](../../ui/gui/text-and-display/image.md#usage) | GUI texture owner | Managed resource | Disclosure, Social and media, Canvas and bars | Cache a texture for the widget in its window |
| [`cui_state_on_free`](../../ui/gui/text-and-display/image.md#usage) | GUI texture owner | Managed resource | Disclosure, Social and media, Canvas and bars | Release native state when its owner retires |
| [`cui_status_button`](../../ui/gui/actions/status-button.md) | `StatusButtons.StatusButton` | Adopted | Actions | Show busy and success feedback for Save changes |
| [`cui_status_spinner`](../../ui/gui/feedback/status-spinner.md) | `StatusSpinners.StatusSpinner` | Adopted | Feedback | Show busy, success and error for Upload |
| [`cui_status_tag`](../../ui/gui/creative-controls/status-tag.md) | `StatusTags.StatusTag` | Adopted | Creative controls | Display and remove workflow state tags |
| [`cui_step_slider`](../../ui/gui/creative-controls/step-slider.md) | `StepSliders.StepSlider` | Adopted | Creative controls | Select a variation count and invoke Generate |
| [`cui_stepper`](../../ui/gui/navigation/stepper.md) | `Steppers.Stepper` | Adopted | Navigation | Display progress through four steps |
| [`cui_swipe_item_begin`](../../ui/gui/actions/swipe-actions.md) | `SwipeActions.SwipeItem` | Adopted | Actions | Open a swipe row with commands on both sides |
| [`cui_swipe_item_end`](../../ui/gui/actions/swipe-actions.md) | `SwipeActions.SwipeItem` | Adopted | Actions | Accept one side and action index after a swipe |
| [`cui_switch`](../../ui/gui/selection/switch.md) | `Switches.Switch` | Adopted | Buttons, Selection, Overlays | Public profile and notification preferences |
| [`cui_tab_bar`](../../ui/gui/motion/tab-bar.md) | `TabBars.TabBar` | Adopted | Motion | Select one of four tabs and show an Inbox badge |
| [`cui_table`](../../ui/gui/data-display/table.md) | `Tables.Table` | Adopted | Data display, Data grid | Sortable columns, selection and mixed cell content |
| [`cui_table_cell`](../../ui/gui/data-display/table.md) | `Tables.Cell.Content` | Adopted | Data display, Data grid | Compose an avatar and name, an email or a badge in a cell |
| [`cui_table_cell_text`](../../ui/gui/data-display/table.md) | `Tables.Cell.Text` | Adopted | Data display, Data grid | Draw plain role and right-aligned revenue cells |
| [`cui_table_row`](../../ui/gui/data-display/table.md) | `Tables.Row` | Adopted | Data display, Data grid | Keep one selected person across sort changes |
| [`cui_tabs`](../../ui/gui/navigation/tabs.md) | `Tabs.Tabs` | Adopted | Navigation | Underline tabs with in-place content and pill tabs with icons |
| [`cui_tag_input`](../../ui/gui/inputs/tag-input.md) | `TagInputs.TagInput` | Adopted | Inputs | Editable Topics list |
| [`cui_terminal`](../../ui/gui/files/terminal.md) | `Terminals.Terminal` | Adopted | Files | Display the ANSI terminal and submit an input line |
| [`cui_terminal_clear`](../../ui/gui/files/terminal.md) | `Terminal revision` | Adopted | Files | Clear the output on the clear command |
| [`cui_terminal_init`](../../ui/gui/files/terminal.md) | `Terminals.Options` | Adopted | Files | Allocate history with byte and line capacities |
| [`cui_terminal_printf`](../../ui/gui/files/terminal.md) | `Terminal output` | Adopted | Files | Format text in FPAS and append it to output |
| [`cui_terminal_write`](../../ui/gui/files/terminal.md) | `Terminal output` | Adopted | Files | Append the welcome message and built-in command output |
| [`cui_test_explorer`](../../ui/gui/more-developer-tools/test-explorer.md) | `TestExplorers.TestExplorer` | Adopted | More developer tools | Browse suites and request test runs |
| [`cui_text`](../../ui/gui/text-and-display/formatted-text.md) | `Elements.Styled` and `Elements.FormattedText` | Adopted | Overview, Buttons, Overlays, Feedback, Data display, Actions, Disclosure, Reader, Data grid, Social and media, Planning, Motion, AI, Developer tools, Canvas and bars | Introduction, single-line team labels and the monospaced install command |
| [`cui_text_reveal`](../../ui/gui/motion/text-reveal.md) | `TextReveals.TextReveal` | Adopted | Motion | Replay the release-build text by word |
| [`cui_textarea`](../../ui/gui/inputs/text-area.md) | `TextAreas.TextArea` | Adopted | Inputs | Full-width multiline About field |
| [`cui_theme_switcher`](../../ui/gui/actions/theme-switcher.md) | `ThemeSwitchers.ThemeSwitcher` | Adopted | Actions | Choose Light, Dark or System in a compact row |
| [`cui_theme_toggle`](../../ui/gui/actions/theme-toggle.md) | `ThemeToggles.ThemeToggle` | Adopted | Actions | Toggle between light and dark with stable focus |
| [`cui_ticker`](../../ui/gui/social-and-media/ticker.md) | Tickers.Ticker | Adopted | Social and media | Scroll the six figures and trends |
| [`cui_time`](../../ui/gui/canvas-and-bars/painting.md) | `Painting.Surface.Time` and the MVU session clock | Adopted | Feedback, Actions, Disclosure, AI, Canvas and bars, Voice | Complete the upload after two seconds |
| [`cui_time_dial`](../../ui/gui/pickers/time-dial.md) | `TimeDials.TimeDial` | Adopted | Pickers | Set hours and minutes on the analog face |
| [`cui_time_histogram`](../../ui/gui/more-developer-tools/time-histogram.md) | `TimeHistograms.TimeHistogram` | Adopted | More developer tools | Select time ranges on stacked bars |
| [`cui_time_picker`](../../ui/gui/pickers/time-picker.md) | `TimePickers.TimePicker` | Adopted | Pickers | Select 24-hour or 12-hour time with native columns |
| [`cui_timeline_item`](../../ui/gui/feedback/timeline-item.md) | `TimelineItems.TimelineItem` | Adopted | Feedback | Show three activity entries and end the connector on the final entry |
| [`cui_toast`](../../ui/gui/overlays/toast.md) | `Toasts.Show` | Adopted | Overview, Buttons, Inputs, Navigation, Overlays, Feedback, Data display, Actions, Disclosure, Social and media, Planning, Motion, AI, Files, Voice | Workspace, hold, project and code completion feedback |
| [`cui_toc`](../../ui/gui/reader/table-of-contents.md) | TableOfContents.TableOfContents | Adopted | Reader | Jump to a section and track the active entry |
| [`cui_toc_anchor`](../../ui/gui/reader/scroll-anchor.md) | ScrollAnchors.ScrollAnchor | Adopted | Reader | Mark each article heading with a zero-height destination |
| [`cui_toggle_group`](../../ui/gui/actions/toggle-group.md) | `ToggleGroups.ToggleGroup` | Adopted | Actions | Independent icon toggles and exclusive alignment |
| [`cui_tool_call`](../../ui/gui/ai/tool-call.md) | `ToolCalls.ToolCall` | Adopted | AI | Expand Search input and output and show file progress |
| [`cui_toolbar`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Toolbar` | Adopted | Canvas and bars | Draw grouped and floating bars |
| [`cui_toolbar_button`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Item.Button` | Adopted | Canvas and bars | Build action and toggle buttons |
| [`cui_toolbar_cycle`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Item.Cycle` | Adopted | Canvas and bars | Cycle the scale and format |
| [`cui_toolbar_group`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Item.Group` | Adopted | Canvas and bars | Group related toolbar controls |
| [`cui_toolbar_primary`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Item.Primary` | Adopted | Canvas and bars | Draw the Generate action |
| [`cui_toolbar_radio`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Item.Radio` | Adopted | Canvas and bars | Choose one tool mode |
| [`cui_toolbar_separator`](../../ui/gui/canvas-and-bars/toolbar.md) | `Toolbars.Item.Separator` | Adopted | Canvas and bars | Separate the floating actions and choices |
| [`cui_tooltip`](../../ui/gui/overlays/tooltip.md) | `Tooltips.Tooltip` | Adopted | Buttons shell, Overlays | Show the name of the hovered accent |
| [`cui_tour`](../../ui/gui/overlays/guided-tour.md) | `Tours.Tour` | Adopted | Audio | Explain the knob, meter and fader in three steps |
| [`cui_trace_sort`](../../ui/gui/developer-tools/trace-viewer.md) | `TraceGeometry.SortSpans` | Adopted | Developer tools | Sort spans by track, depth and start time |
| [`cui_trace_view`](../../ui/gui/developer-tools/trace-viewer.md) | `TraceViewers.TraceViewer` | Adopted | Developer tools | Select spans, measure time and collapse lanes |
| [`cui_tree_begin`](../../ui/gui/data-display/tree.md) | `Trees.Branch and Trees.Leaf` | Adopted | Data display | Render folders and source files with their native icons |
| [`cui_tree_cell`](../../ui/gui/data-grid/tree-cell.md) | TreeTables.TreeCell | Adopted | Data grid | Show branching and indentation in an ordinary table |
| [`cui_tree_clicked`](../../ui/gui/data-display/tree.md) | `Trees.Branch and Trees.Leaf` | Adopted | Data display | Select a component file without changing other leaf rows |
| [`cui_tree_end`](../../ui/gui/data-display/tree.md) | `Trees.Branch` | Adopted | Data display | End an expanded folder body |
| [`cui_tree_move`](../../ui/gui/data-grid/tree-table.md) | `Fpas.Ui.Gui.TreeTables.Move` | Adopted | Files | Move sample project nodes and rebase their indices |
| [`cui_tree_table_begin`](../../ui/gui/data-grid/tree-table.md) | `TreeTables.TreeTable` | Adopted | Files | Begin the native project tree table |
| [`cui_tree_table_end`](../../ui/gui/data-grid/tree-table.md) | `TreeTables.Change` | Adopted | Files | Adopt native row and drop proposals |
| [`cui_tree_table_menu`](../../ui/gui/data-grid/tree-table.md) | `TreeTables.TreeTable Menu` | Adopted | Files | Open the menu for a project node |
| [`cui_tree_table_name`](../../ui/gui/data-grid/tree-table.md) | `TreeTables.Node.Label` | Adopted | Files | Draw a folder or file glyph beside the node name |
| [`cui_tree_table_row`](../../ui/gui/data-grid/tree-table.md) | `TreeTables.Node` | Adopted | Files | Draw the visible nodes with size and kind cells |
| [`cui_treemap`](../../ui/gui/more-charts/treemap.md) | `Treemaps.Treemap` | Adopted | More charts | Compare the areas of ten bundle sizes |
| [`cui_typewriter`](../../ui/gui/motion/typewriter.md) | `Typewriters.Typewriter` | Adopted | Motion, AI | Replay the streamed text and its caret |
| [`cui_typing_indicator`](../../ui/gui/social-and-media/typing-indicator.md) | TypingIndicators.TypingIndicator | Adopted | Social and media | Show the participant who is typing |
| [`cui_uptime`](../../ui/gui/more-developer-tools/uptime.md) | `Uptimes.Uptime` | Adopted | More developer tools | Inspect day-level service availability |
| [`cui_url_params`](../../ui/gui/more-developer-tools/http-request-bar.md#query-parameters) | `Fpas.Ui.Gui.UrlParameters.Parse` | Adopted | More developer tools | Read the parameter rows from the URL |
| [`cui_url_set_params`](../../ui/gui/more-developer-tools/http-request-bar.md#query-parameters) | `Fpas.Ui.Gui.UrlParameters.Replace` | Adopted | More developer tools | Apply parameter edits to the URL |
| [`cui_user_menu`](../../ui/gui/actions/user-menu.md) | `UserMenus.UserMenu` | Adopted | Actions | Open the account menu from a fixed-width trigger |
| [`cui_veil`](../../ui/gui/disclosure/veil.md) | Veils.Veil | Adopted | Disclosure | Reveal and cover a picture, sample identifier and spoiler |
| [`cui_viewport_size`](../../ui/gui/layout/layout-inspection.md#read-the-bounds) | `Viewport.LogicalWidth` | Adopted | Overview, Buttons, Inputs, Selection, Navigation, Overlays | Desktop sidebar and compact layout |
| [`cui_virtual_list`](../../ui/gui/data-display/virtual-list.md) | `VirtualLists.VirtualList` | Adopted | Data display | Realize the viewport range of a 10,000-invoice list |
| [`cui_voice_note`](../../ui/gui/voice/voice-note.md) | `VoiceNotes.VoiceNote` | Adopted | Voice | Play, seek and cycle the speed of a voice message |
| [`cui_voice_resample`](../../ui/gui/voice/voice-waveform.md) | `CuiWindow.ResampleVoice` | Adopted | Voice | Keep a compact peak waveform for the recorded message |
| [`cui_voice_session`](../../ui/gui/voice/voice-session.md) | `VoiceSessions.VoiceSession` | Adopted | Voice | Show the scripted call, live transcript and call controls |
| [`cui_voice_wave`](../../ui/gui/voice/voice-waveform.md) | `VoiceWaves.VoiceWave` | Adopted | Voice | Display scrolling live levels and seek in a recorded clip |
| [`cui_waffle`](../../ui/gui/more-charts/waffle.md) | `Waffles.Waffle` | Adopted | More charts | Show the platform shares of 100 installs |
| [`cui_wants_keyboard`](../../ui/gui/host.md#application-shortcuts) | GUI host shortcut guard | Adopted | Overview, Buttons shell | Native editors keep keyboard ownership during Primary+J |
| [`cui_waterfall`](../../ui/gui/developer-tools/network-waterfall.md) | `NetworkWaterfalls.NetworkWaterfall` | Adopted | Developer tools | Inspect timing bars and select requests |
| [`cui_waveform`](../../ui/gui/motion/waveform.md) | `Waveforms.Waveform` | Adopted | Motion | Show and seek within 96 audio peaks |
| [`cui_week_calendar`](../../ui/gui/planning/week-calendar.md) | `WeekCalendars.WeekCalendar` | Adopted | Planning | Move and resize timed appointments |
| [`cui_wheel_picker`](../../ui/gui/pickers/wheel-picker.md) | `WheelPickers.WheelPicker` | Adopted | Pickers | Operate the three joined date wheels |
| [`cui_word_rotate`](../../ui/gui/motion/rotating-words.md) | `WordRotations.WordRotate` | Adopted | Motion | Cycle through four destinations |

## Overview evidence

The source reference is CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
Capture page index 0 with CUI's `cui_screenshots` and render the FPAS Overview
with the software renderer. Use Windows, 150% scale, neutral accent, English,
and light and dark themes. Capture the initial view at 1240 × 1000,
640 × 1000 and 500 × 1000 logical units.

Use the FPAS heading, introduction, sample emails and email placeholder in the
reference capture. Compare all RGBA channels. At 1240 logical units, compare
pixels from x = 363 to the viewport edge. At the two compact widths, compare
pixels from y = 105 to the viewport edge. All six page-body comparisons have
zero differing pixels.

An additional 500 × 1400 capture includes every stacked Team row. At both
compact widths, using the same header title also gives a full-frame match in
both themes. The desktop sidebar displays the gallery's own name and logo.

The native parity test compares a truncated formatted label and its hover tooltip
against direct CUI calls. The gallery contract drives the workspace controls,
keyboard shortcuts, command palette and scroll navigation with native input.
[Native parity test](../../../src/Fpas.Ui.Gui.Cui/native/tests/layout_parity.c)
[Gallery contract](../../../tests/Fixtures/gui-host/CompletionContract.fpas)

The focused Overview contract passes with the Windows software and OpenGL
renderers. The full Windows software run stops at a range ruler pointer check
that also fails on the unchanged FPAS baseline.

## Buttons evidence

The audit covers 44 CUI calls in `page_buttons`, the shared layout and sidebar,
and the overlays available from Buttons. The [CUI function table](#cui-functions)
links each call to its FPAS API and option reference. CUI reserves the
`selectable` field on text and gives it no runtime behavior.

Capture page index 1 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
Use Windows, the software renderer, 150% scale, English, and the neutral accent.
Capture light and dark at 1240 × 1000, 640 × 1000 and 500 × 1400 logical units.
Repeat at all three widths with a height of 1420 logical units.
At 1240 × 1420, capture Options and More actions open in each theme.
Move the pointer off the menu after activation and let animations settle.

Compare all RGBA channels to the viewport edges. At desktop width, start at
physical x = 363. At compact widths, start at physical y = 105. These bounds
exclude the sidebar or compact header with its gallery name.
All 16 comparisons have zero differing pixels. The taller captures at 1240
and 640 logical units include the full closed dropdown card.
See the [copy button capture](../../ui/gui/actions/copy-button.md)
for the monospaced install command and its native control.

The [Buttons contract](../../../tests/Fixtures/gui-host/ButtonsContract.fpas)
drives the loading and reset buttons, the group, both selectors, and the hold
button with native input. [Dropdown coverage](../../../tests/Fixtures/gui-host/DropdownContract.fpas)
drives both menus. The [shared dialog contract](../../../tests/Fixtures/gui-host/CompletionContract.fpas)
checks Cancel, confirmation and the toast action. See the [widget guides](../../ui/gui/actions/README.md#buttons-gallery-page)
for the example sources and FPAS APIs.

The Buttons and dropdown contracts pass under Windows with software and
OpenGL. The [native action test](../../../src/Fpas.Ui.Gui.Cui/native/tests/link_render.c)
checks the copy callback and hold completion. The full Windows software run
reaches the same range ruler pointer check as the Overview run.

### Linux X11 check

Run both desktop galleries with the software renderer and 100% scale.
Set the neutral accent and use the theme selector to capture Light and Dark.
At 1240, 640, and 500 logical units wide, set the height to 1420.
Compare the page body from x = 242 on desktop and y = 70 at compact widths.
At desktop width, open Options and More actions in both themes.
The six closed views and four open menu views have zero differing pixels.

Copy feedback, reset, short hold, and the settled hold toast match.
At 1240 × 820, both views match after wheel scrolling and after sidebar reselection.
The focused contract checks the intermediate scroll position during the reset.
The spinner and entering toast can show different animation phases.
Segment highlight animation also depends on frame timing.

The Buttons body also matches with the blue accent and after closing the compact
drawer. In both galleries, Primary+K opens the palette over the drawer.
Escape closes the palette and leaves the drawer open. Primary+J changes
appearance with the drawer open. The focus contract checks native editor priority.

The drawer and Delete dialog use CUI panels and scrims. The FPAS host also
draws blocked background controls in their disabled style.
The sidebar and drawer show the FPAS name and FPAS-specific destinations.
FPAS keeps the Save button ID stable while its label changes.

## Inputs evidence

Use page index 2 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The three original sections contain 14 examples from ten widget families.
The [CUI function table](#cui-functions) covers all 17 calls in the page and
its section helper. Shared rows, navigation, and overlays use the same
options as the [Buttons shell](#buttons-evidence).
The [Inputs guide](../../ui/gui/inputs/README.md#inputs-gallery-page)
describes the card and grid structure.
Set the reference email to the gallery's reserved sample `maya@example.com`.

Capture Windows software rendering at 150% scale, in English with the neutral
accent. Capture light and dark at 1240 × 1000, 640 × 1000 and 500 × 1400 logical
units. Repeat at all three widths with a height of 1420 logical units.
Compare RGBA through the viewport edges, including the scrollbar.
Start at physical x = 363 on desktop and y = 105 at compact widths.
All 12 initial comparisons have zero differing pixels.

Enable reduced motion for interaction captures. At 500 × 1420, scroll five
wheel lines down to show the full Pickers card. Compare from physical
y = 48 through the viewport edges. Both themes match exactly.

At 1240 × 1420, open the Country, Language and Due date popups separately.
All six comparisons match through the viewport edges. Type `abc` in Password to compare
validation. In both themes, 72 pixels differ at the caret's blink phase.

The [text fields contract](../../../tests/Fixtures/gui-host/TextFieldContract.fpas)
checks native editing and shared profile values. The
[credentials contract](../../../tests/Fixtures/gui-host/CredentialFieldsContract.fpas)
checks tags, code completion and edits in both password fields.
The [picker fields contract](../../../tests/Fixtures/gui-host/PickerFieldsContract.fpas)
checks the select and numeric examples. The
[date contract](../../../tests/Fixtures/gui-host/DateCalendarContract.fpas)
and [color contract](../../../tests/Fixtures/gui-host/ColorEmojiContract.fpas)
check their native popups. See the [input guides](../../ui/gui/inputs/README.md)
for widget options and example sources.

All five contracts pass under Windows with software and OpenGL.
The native input regression, smart input and calendar tests also pass.

### Linux X11 check

Use the software renderer at 100% scale with the same sample email.
Capture Light and Dark at 1240, 640, and 500 logical units wide and 1420 high.
At 500 logical units, also capture the scrolled page in both themes.
Compare the page body from x = 242 on desktop and y = 70 at compact widths.

Open Country, Language, Due date, and Brand color at desktop width.
Select Japan, then filter and select Zig. Enter and clear a search query.
Increase Seats, add a tag, complete the verification code, and enter a
short password. Twenty of the 21 page-body comparisons match exactly.
The open Language popup differs at 32 pixels in the blinking caret.

## Selection evidence

Use page index 3 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers the 15 CUI functions in the page
and its section helper. The [Selection guide](../../ui/gui/selection/README.md)
describes the native grid, cards and wrapping chip row.

On Linux X11, use the software renderer at 100% scale. Capture Light and
Dark at widths of 1240, 640, 568, 567, 500 and 470 logical units and a height
of 1420. At 470 logical units, scroll four wheel lines down in both themes.
Let the scrollbar fade before capture. Compare RGB to the viewport
edges, starting at x = 242 on desktop and y = 70 at compact widths.

At desktop width, capture the blue accent and restore the neutral accent.
Toggle the newsletter and Bluetooth. Use Space on the Bluetooth switch.
Click the disabled checkbox and select the same radio row twice.
Select a radio card with the pointer and move to the next card with Right.
Drag Volume and the lower price endpoint. Step Brightness to 50% with Right.
Toggle Engineering and select the middle star twice to set and clear the rating.
All 29 views have zero differing pixels in the compared area.

The [Selection contract](../../../tests/Fixtures/gui-host/SelectionContract.fpas)
checks the model, focus and grid resize from 520 to 568 and then 567.
The [native input test](../../../src/Fpas.Ui.Gui.Cui/native/tests/input_regressions.c)
compares a formatted slider and a range slider against direct CUI calls.

## Navigation evidence

Use page index 4 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers the 12 CUI functions in the page
and its section helper. The [Navigation guide](../../ui/gui/navigation/README.md)
describes the native cards and the tab content column.

On Linux X11, use the software renderer at 100% scale. Capture Light and Dark
at widths of 1240, 760, 759, 640, 500 and 360 logical units and a height of 1420.
At 360 × 620, scroll down and wait for the scrollbar to fade. Compare RGB to
the viewport edges, starting at x = 242 on desktop and y = 70 at compact widths.

Select an underline tab and move one tab with Right. Select Billing in the
pill strip. Click the current breadcrumb and open and dismiss an ancestor toast.
Advance the pager by click and Enter. Test the last-page boundary and resize
to compact width after going back one page.

Click the stepper and verify that its state stays the same. Go back to the
first step and test the disabled Back button. Advance to Done and activate
Finish. Capture the success toast and its dismissal. All 32 views have zero
differing pixels in the compared area.

The [Navigation contract](../../../tests/Fixtures/gui-host/NavigationContract.fpas)
checks native card insets, focus, page boundaries, reflow and toasts.
The [tab binding tests](../../../tests/Fpas.Ui.Gui.Cui.Tests/Navigation/CuiTabsTests.cs)
check the native size and activation of icon-only tabs.
FPAS keeps keyboard focus when Next becomes Finish.
See [CUI-022](../cui-upstream-bugs.md#cui-022) for the CUI gallery difference.

## Overlays evidence

Use page index 5 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 29 CUI functions in the page,
the section helper, and the dialog, sheet and palette content.

On Linux X11, use the software renderer at 100% scale. Capture Light and Dark
at widths of 1240, 760, 759, 640, 500 and 360 logical units and a height of 1420.
At 360 × 620, scroll down and wait for the scrollbar to fade. Compare RGB to
the viewport edges, starting at x = 242 on desktop and y = 70 at compact widths.

At desktop width, open the Notifications sheet and change Weekly digest.
Edit the profile email to `maya@example.com` in both galleries before saving.
Cancel and confirm project deletion. Change the Filters controls and close the
popover. Open the context menu by right-clicking the muted card. Check Escape
and a menu choice. Capture the toast stack, the archive action, and two tooltips.
Open the palette, search for Go to Inputs, and activate the command with Enter.

Thirty of the 38 page-body comparisons have zero differing pixels. This
includes all initial views, the open Filters panel and context menu, both
tooltips, the toast stack and the destination after palette activation.
The sheet and deletion dialog also match inside their panels.

While a modal is open, FPAS draws blocked background controls with disabled
styling. Caret blink timing can also change the visible palette query caret.
The initial profile dialog shows different sample email values. After editing
to the same value, the only difference inside the panel is at the switch edge.
See [CUI-025](../cui-upstream-bugs.md#cui-025)
for the Apply difference and [CUI-024](../cui-upstream-bugs.md#cui-024) for the
tooltip limit.

The [dialog contract](../../../tests/Fixtures/gui-host/DialogContract.fpas)
checks drag, resize, dismissal and focus. The
[popover contract](../../../tests/Fixtures/gui-host/PopoverContract.fpas)
checks native spacing and closure. The
[context-menu contract](../../../tests/Fixtures/gui-host/ContextContract.fpas)
checks the target bounds and activation. The
[toast contract](../../../tests/Fixtures/gui-host/ToastContract.fpas) and
[tooltip contract](../../../tests/Fixtures/gui-host/TooltipContract.fpas)
check timed behavior. The
[shared gallery contract](../../../tests/Fixtures/gui-host/CompletionContract.fpas)
checks palette shortcuts, notification values and both deletion launchers.

## Feedback evidence

Use page index 6 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers the 23 CUI functions in the page
and its section helper. The [Feedback guide](../../ui/gui/feedback/README.md)
describes the direct page children and the two responsive grids.

On Linux X11, use the software renderer, 100% scale and reduced motion.
Capture Light and Dark at widths of 1240, 760, 759, 568, 567 and 360 logical
units and a height of 1680. Capture the bottom of the page in both themes
at 1240 × 1680 and 360 × 1000. Wait for the scrollbar to fade. Compare RGB
to the viewport edges, starting at x = 242 on desktop and y = 70 at compact
widths.

The plain Indexing bar moves under reduced motion. Compare two frames from
each gallery to identify the moving strip. Exclude only that strip from the
settled pixel comparison.

Start two uploads. Capture each busy state and the success and error results.
Dismiss the storage alert, leave Feedback and return to the page. Scroll to
the empty state and activate New project to show the success toast.
All 24 comparisons match outside the moving Indexing bar. Returning to
Feedback leaves the dismissed alert hidden and preserves the spinner timer.

The [Feedback contract](../../../tests/Fixtures/gui-host/FeedbackContract.fpas)
checks the grid, focus, alerts, upload replies and the project action.
The [skeleton contract](../../../tests/Fixtures/gui-host/SkeletonContract.fpas)
checks native placeholder geometry. See [CUI-026](../cui-upstream-bugs.md#cui-026)
for the narrow progress track guard.

## Data display evidence

Use page index 7 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 25 CUI functions in the page
and its section helper. The [Data display guide](../../ui/gui/data-display/README.md)
describes the native table, responsive Tree and List grid, direct reorder
rows, accordion and invoice viewport.

Use Linux X11, the software renderer, 100% scale and reduced motion. Use
the same five `example.com` addresses in the comparison fixture. Compare
RGB from x = 242 in the desktop layout and from y = 70 in the compact
layout. The first positions match in Light and Dark at widths of 1240, 760,
759, 568, 567 and 360. The bottom of the page also matches in both themes
at 1240 × 1680 and 360 × 1000.

Select a person and sort by Name, Role and Status. Select a component file,
collapse and reopen its branch, expand Assets and click `main.c`. Keep the
first accordion open while opening a second one, then close the first.
Drag the first task to the last position and hold the pointer there until
the release frame. Both galleries show the same drag, saved order and
Order saved toast. Scroll inside the invoice viewport to show later rows.
The comparison captures for these states match in both galleries.

The [gallery contract](../../../tests/Fixtures/gui-host/GalleryTableContract.fpas)
checks row selection across sort changes, task drag and the saved-order toast. The
[accordion contract](../../../tests/Fixtures/gui-host/DisclosureContract.fpas)
checks independent expansion and keyboard input. The
[native collection tests](../../../src/Fpas.Ui.Gui.Cui/native/tests/collection_regressions.c)
cover table options, reorder state retirement and virtual list range changes.
See [CUI-027](../cui-upstream-bugs.md#cui-027) for viewport growth without input.

## Charts evidence

Use page index 8 of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 11 CUI functions in the page and
its section helper. The [Charts guide](../../ui/gui/charts/README.md#charts-gallery-page)
describes the native cards, grouped examples and centered gauges.

Use Linux X11, the software renderer, 100% scale and reduced motion. Compare
RGB from x = 242 in the desktop layout and y = 70 in the compact layout.
Compare the top of the page in both themes at widths of 1240, 760, 759, 568,
567 and 360. Compare the bottom at 1240 × 1680 and 360 × 1000 in both themes.

Move the revenue crosshair between samples. Inspect the grouped bars, donut
slices and legend, dither bands, storage parts and legend, and stacked bars.
Toggle Grayscale, navigate away and return. Use Right and End on the focused
meter. The captures match for all 32 comparisons, including interaction states.

The [native chart tests](../../../src/Fpas.Ui.Gui.Cui/native/tests/charts.c)
compare the adapter with direct CUI calls. The
[binding tests](../../../tests/Fpas.Ui.Gui.Cui.Tests/Display/CuiChartTests.cs)
exercise option validation, native keyboard selection and rendering.
See [CUI-028](../cui-upstream-bugs.md#cui-028) through
[CUI-030](../cui-upstream-bugs.md#cui-030) for chart edge cases.

## Actions evidence

Use the first extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 27 CUI functions in the page and
its section helper. Use the same reserved account address in both galleries.

Use Linux X11, the software renderer, 100% scale and reduced motion. Compare
RGB from x = 242 in the desktop layout and y = 70 in the compact layout.
The page body matches in Light and Dark at widths of 1240, 760, 759, 568,
567 and 360. The compact bottom matches at 360 × 1000 in both themes.

Open the split menus, select Save and publish, and change both toggle groups.
Confirm Delete project, start a save, and change a reaction. Compare the
account menu, three swipe outcomes and the open speed dial and its selection.
The 34 exact captures include the Busy and Saved faces. The armed confirmation
differs at the countdown line endpoint because the frames use different times.

The FPAS appearance selector follows the shared theme when the toggle changes
it. The CUI sample keeps its caller-owned selection. FPAS closes the split menu
on the second chevron click and restores Save changes after the success timeout.
See [CUI-031](../cui-upstream-bugs.md#cui-031) through
[CUI-035](../cui-upstream-bugs.md#cui-035) for the native findings.

The [Actions contracts](../../ui/gui/actions/README.md#source-files) check
keyboard and pointer input, scope retirement and timed save feedback. The
[toggle group contract](../../../tests/Fixtures/gui-host/ToggleGroupContract.fpas)
checks exclusive and independent choices and the tooltip limit.

## Smart inputs evidence

Use the second extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 15 CUI functions in the page and
its section helper. The [Smart inputs guide](../../ui/gui/smart-inputs/README.md)
describes the native composition and the shared field and picker options.

Use Linux X11, the software renderer, Noto Color Emoji, 100% scale and
reduced motion. Compare RGB from x = 242 in the desktop layout and y = 70
in the compact layout. The page body matches in Light and Dark at widths of
1240, 760, 759, 568, 567 and 360. It also matches at the compact bottom in
both themes at 360 × 820.

Commit the title with Enter. Cancel an owner edit with Escape and commit
another on blur. Drag the opacity, fine-adjust the radius, and type a radius.
Edit both amounts and affixed fields. Record, cancel and clear a shortcut.
Open the emoji picker, search for Grinning face, and choose the result.
Both message values contain the complete emoji. The tested text field font
does not draw its glyph. The picker shows it.

The page body matches in 29 captures. The recorder pulse and search caret
vary with frame timing. FPAS removes thousands separators from the amount
when pointer focus starts editing. The CUI sample can keep them. FPAS closes
the emoji popover after insertion. The CUI sample can leave it open.
See [CUI-025](../cui-upstream-bugs.md#cui-025),
[CUI-036](../cui-upstream-bugs.md#cui-036) through
[CUI-040](../cui-upstream-bugs.md#cui-040) for the native findings.

The [native tests](../../../src/Fpas.Ui.Gui.Cui/native/tests/smart_inputs.c)
check input layout and native interaction. The
[Money contract](../../../tests/Fixtures/gui-host/MoneyDecimalContract.fpas)
checks exact decimal values. The
[emoji contract](../../../tests/Fixtures/gui-host/ColorEmojiContract.fpas)
checks complete Unicode insertion, recent emoji, and skin tone.

## Pickers evidence

Use the third extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers the 15 calls in the page and its
section helper. The [Pickers guide](../../ui/gui/pickers/README.md#pickers-gallery-page)
describes the direct rows and native cards.

Use Linux X11, the software renderer, 100% scale and reduced motion.
Compare RGB from x = 242 in the desktop layout and y = 70 in the compact
layout. The page body matches at 1240, 760, 759, 568, 567 and 360 logical
units in Light and Dark. It also matches in both themes at the bottom of the
360 × 1000 viewport.

Open Languages, choose Zig, filter for Kotlin and dismiss the list. Select
Today in the preset picker and choose December 8–16 for Stay with the inline
calendar. Change the start time to 08:35 and switch the end time to AM.
Scroll the day wheel, select 13:15 on the dial, drag both rulers and move the
histogram's lower endpoint. The open popups and settled input states match
for 32 exact comparisons in total.

The [gallery contracts](../../ui/gui/pickers/README.md#source-files) cover
native pointer input, keyboard focus and publication of controlled values.
See [CUI-041](../cui-upstream-bugs.md#cui-041) through
[CUI-045](../cui-upstream-bugs.md#cui-045) for native edge cases.

## Capture evidence

Use the fourth extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 11 calls in the page and its
section helper. The [Capture guide](../../ui/gui/capture/README.md#capture-gallery-page)
describes the three dropzones, avatar row and signature box.

Use Linux X11 with the software renderer, 100% scale and reduced motion.
Compare RGB from x = 242 in the desktop layout and y = 70 in the compact
layout. Capture Light and Dark at a height of 1200 logical units. Use widths
1240, 760, 759, 568, 567 and 360. Scroll to the bottom at 360 × 680 in both
themes. All 14 layout captures match.

Drop a text file on Attach a file and Add images to compare accepted and
rejected feedback. Drop a PNG on Add images. Drop two PNGs in succession on
the avatar. Drop a text file and a PNG together on the tall dropzone. The
caption shows the PNG path. Draw a signature and clear it. Compare all
10 interaction captures. Only the live ink raster differs. CUI computes
stroke thickness from pointer speed and frame time in each window.

The original gallery discards browse activations. The FPAS gallery follows
the example. Applications can supply a browse callback to open a file chooser.
FPAS releases the signature data when the pad leaves the view. CUI can
retain the ink during a short page switch.

The [Capture contract](../../../tests/Fixtures/gui-host/CaptureContract.fpas)
checks native layout and draws and clears a signature. See
[CUI-046](../cui-upstream-bugs.md#cui-046) and
[CUI-047](../cui-upstream-bugs.md#cui-047) for native edge cases.

## Disclosure evidence

Use the fifth extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 23 calls in the page and section
helper, plus four calls for texture ownership. The
[Disclosure guide](../../ui/gui/disclosure/README.md#disclosure-gallery-page)
describes the native rows, cards and panel.

Use Linux X11 with the software renderer, 100% scale and reduced motion.
Set both countdowns to the same values. Use the FPAS sample identifier in
the reference. Compare RGB from x = 242 in the desktop layout and y = 70
in the compact layout.

Capture Light and Dark at widths 1240, 760, 759, 568, 567 and 360 and a
height of 1900 logical units. Scroll to the bottom at 360 × 820 in both
themes. The compact bottom and all Light layout captures match. The
Billing card border in Dark differs by at most two levels per RGB channel.

Open the hover card and move away. Open, cancel and accept the confirmation.
Expand the text. Close and reopen Billing, and open Security. Advance the
checklist and show the archive and release notes toasts. Dismiss and reopen
the banner. Reveal the picture, sample identifier and spoiler, then cover
the identifier. The interaction frames match except for four pixels in the
timed archive toast. Across 36 captures, 28 match exactly.

The [gallery contracts](../../ui/gui/disclosure/README.md#source-files) check
native input and controlled countdown replies. See
[CUI-048](../cui-upstream-bugs.md#cui-048) through
[CUI-053](../cui-upstream-bugs.md#cui-053) for native edge cases.

## Reader evidence

Use the sixth extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers the ten calls in the page. The
[Reader guide](../../ui/gui/reader/README.md#reader-gallery-page) describes
the native viewport and its outline.

Use Linux X11 with the software renderer, 100% scale and reduced motion.
Compare RGB from x = 242 in the desktop layout and y = 70 in the compact
layout. Capture Light and Dark at 1240, 760, 759, 568, 567 and 360 logical
units wide and 900 high.

Click Install. Scroll inside the article and press Enter to return to
Install. Use Tab and Space to open With CMake. Select Without CMake and
Next steps. Resize with the article scrolled, return to the original width,
scroll to the end and jump back to Introduction. All 24 captures match.

The [Reader contract](../../../tests/Fixtures/gui-host/ReaderContract.fpas)
checks anchor alignment, the active entry and the outer scroll position.
See [CUI-054](../cui-upstream-bugs.md#cui-054) and
[CUI-055](../cui-upstream-bugs.md#cui-055) for the frame-order regressions.

## Data grid evidence

Use the seventh extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 20 calls in the page and its
section helper. The [Data grid guide](../../ui/gui/data-grid/README.md#data-grid-gallery-page)
describes the original sections and the supplemental virtual tree table.

Use the reserved `example.com` domain in the reference sample. The filter
state is independent of the people grid. Both galleries retain the previous
display order for equal keys in the Linux comparison.

Use Linux X11 with the software renderer, 100% scale and reduced motion.
Capture Light and Dark at widths 1240, 760, 759, 568, 567 and 360 and a height
of 1480 logical units. Compare RGB from x = 242 in the desktop layout and
y = 70 in the compact layout. End at the bottom of the original source tree.

Open the Team chip and select Design. Open Add filter, type in Search and
clear the filters. Sort by Team and Score, and resize the Name column.
Check single, range and full selection. Cancel one name edit and commit
another. Scroll the grid and expand and collapse the source tree.

The original sections match exactly in 31 of 32 captures. During an edit,
FPAS keeps the avatar tied to the committed name. CUI uses the draft name.

The FPAS gallery preserves person identity when order changes. The original
keeps selection and cursor by display position. FPAS reapplies the name sort
after a name edit. The original reapplies it at the next sort change.

See [CUI-056](../cui-upstream-bugs.md#cui-056) through
[CUI-062](../cui-upstream-bugs.md#cui-062) for the native selection,
editing, column, tree and filter findings.

## Code evidence

Use the eighth extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers seven calls in the page and its
section helper. The C example passes its source to one code block as text.
The [Code guide](../../ui/gui/code/README.md#code-gallery-page) covers the
equal-width row and the supplemental FPAS examples.

Run both galleries on X11 at 100% scale with the software renderer and
reduced motion. At 1580 pixels of height, compare both themes at widths
360, 567, 568, 759, 760 and 1240. Copy the three snippets and compare their
clipboard bytes. At width 920, scroll the shell block horizontally in both
directions. At width 1240, collapse and reopen the root, platforms and renderer,
then expand the first author.

All 25 capture pairs match pixel for pixel within the original page content.
The three copied snippets match byte for byte. The capture boundaries stop
after the last original example.

See [CUI-063](../cui-upstream-bugs.md#cui-063) through
[CUI-065](../cui-upstream-bugs.md#cui-065) for the indentation and scroll
findings.

## Social and media evidence

Use the ninth extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) includes 29 calls from the page,
the section helper and texture cleanup. The
[Social and media guide](../../ui/gui/social-and-media/README.md#social-and-media-gallery-page)
describes the original composition.

On X11 with the software renderer at 100% scale, select the same carousel
slide in both windows before a layout capture. Use reduced motion to hold
the ticker and typing indicator at the same phase.

Capture the visible page body in both themes at widths 360, 567, 568,
759, 760, 800, 920 and 1240 pixels. Use a window height of 2300 pixels.
At width 1240, remove the first like and restore it, then add a sale.
Select a carousel dot, scroll sideways and drag a slide. Zoom and pan the
landscape, resize the crop and move the comparison divider. Swipe the top
card right to show the Saved toast. Hover over a presence avatar.

Of 27 capture pairs, 20 match pixel for pixel. Six Dark captures differ at
the shadows below the top card. Each affected channel differs by at most
10 levels. The presence hover capture differs because FPAS expands the row
under the pointer. The other ten interaction captures match.

The [gallery contract](../../../tests/Fixtures/gui-host/MediaContract.fpas)
checks the like count, revenue and media actions in the FPAS model.
The [native Social test](../../../src/Fpas.Ui.Gui.Cui/native/tests/social.c)
checks widget options, input and layout, including carousel auto-advance.

See [CUI-066](../cui-upstream-bugs.md#cui-066) through
[CUI-076](../cui-upstream-bugs.md#cui-076) for the native findings.

## Planning evidence

Use the tenth extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers the 23 calls from the page and
its section helper. The
[Planning guide](../../ui/gui/planning/README.md#planning-gallery-page)
describes the four sections and the date calculations.

Use X11, the software renderer at 100% scale, and reduced motion. Run
both galleries in the same local time zone. Compare the visible page
body at 2300 pixels of height in both themes. Use widths 360, 567, 568,
759, 760, 800, 920, and 1240 pixels.

At width 1240, move a card between columns and reorder within a column.
Scroll the board, then activate an Add button and a card. Move and
resize a week appointment and a Gantt task. Scroll the timeline.
Advance the month, go back, and select a day.

Of 30 capture pairs, 22 match pixel for pixel. All light and interaction
captures match. The eight dark captures differ only at the Kanban card
shadows, by at most three levels in each RGB channel. See
[CUI-076](../cui-upstream-bugs.md#cui-076).

The [native Planning tests](../../../src/Fpas.Ui.Gui.Cui/native/tests/planning.c)
check input and layout for all four widgets. The
[FPAS interaction contract](../../../tests/Fixtures/gui-host/PlanningInteractionContract.fpas)
checks date navigation, model-owned edits and interrupted drags.

See [CUI-077](../cui-upstream-bugs.md#cui-077) through
[CUI-086](../cui-upstream-bugs.md#cui-086) for the native findings.

## More charts evidence

Use the eleventh extended page of CUI commit `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
The [function table](#cui-functions) covers 24 original calls, including the
section helper and date operations. The
[More charts guide](../../ui/gui/more-charts/README.md#more-charts-gallery-page)
describes the composition and chart order.

Use X11, reduced motion and the software renderer at 100% scale. Use the
same local date in both galleries. Compare the visible page body at 4200
pixels of height in both themes. Use widths 360, 567, 568, 759, 760,
800, 920 and 1240 pixels.

At width 1240, inspect the chart marks and legends with the pointer.
Zoom and pan the candlestick chart, then navigate with Home and End.
Apply Shuffle and check that both galleries retain all six labels and
show the new amounts in descending order.

Of 35 capture pairs, 31 match pixel for pixel. All 16 layout captures
match. FPAS ignores the calendar gap and keeps the candlestick plot
visible in the two keyboard-focus captures. Shuffle produces different
random amounts in each gallery.

The [native tests](../../../src/Fpas.Ui.Gui.Cui/native/tests/more_charts.c)
check labels, animated ranks, the candlestick and the focus ring. The
[Treemap contract](../../../tests/Fixtures/gui-host/TreemapContract.fpas)
checks activation and selection messages in the FPAS model.

See [CUI-087](../cui-upstream-bugs.md#cui-087) through
[CUI-095](../cui-upstream-bugs.md#cui-095) for the native findings.

## Motion evidence

The reference is CUI revision `6864776f1a77b920a26f4ec51f5c4cf1be0b04a6`.
Its [Motion page](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L945)
uses a text-effects card, three equally weighted spotlight cards, five loaders
in a row, and a centered dock. Two equally weighted columns hold the grid of
control tiles and the playback controls. The
[FPAS gallery](../../ui/gui/motion/README.md#motion-gallery-page) uses this
structure and places an additional rolling number example below it.

The Linux X11 comparison uses the software renderer at scale 1. Both
windows are 2200 pixels tall. The 16 pairs cover widths of 1240, 920,
800, 760, 759, 568, 567, and 360 pixels in light and dark themes. The
player is paused and set to the same position. The captures end with the
original tab bar. CUI still animates the loaders under reduced motion.
Outside the loader regions, five light captures match exactly. The other
three differ by one channel level on one pixel at the active tab edge.
Dark captures also show a player card shadow difference of at most three
levels per color channel after switching themes. At 1240 pixels, the player
card matches exactly when CUI starts in dark mode. See
[CUI-076](../cui-upstream-bugs.md#cui-076).

With motion enabled, both galleries replay the typewriter and text reveal,
rotate words, animate the loaders, magnify the dock icons on hover and
advance the player. Pausing keeps the seek position. The controls in both
galleries show the Files and Next track toasts, update the five settings
tiles, seek in the waveform on click and drag, select an Inbox tab, and
show the spotlight glow on hover.

The native tests cover reduced motion, text timing and options, hover and
disabled state, the dock and tab layout, and audio seek input. The page uses
the [FPAS playback clock](../../ui/gui/motion/README.md#playback-clock).
See [CUI-096 through CUI-106](../cui-upstream-bugs.md#cui-096) for the native
findings.

## AI evidence

The reference is [CUI revision `6864776`](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1035).
Both layouts use a 1.6-to-1 width ratio for the conversation and plan columns.
The composer and suggestions use the native controls. The standalone model
picker in the [FPAS gallery](../../ui/gui/ai/README.md#ai-gallery-page) follows
the original examples.

The Linux X11 comparison uses the software renderer at scale 1 with reduced
motion. Each window is 2200 pixels tall. The 16 pairs cover widths of 1240,
920, 800, 760, 759, 568, 567, and 360 pixels in light and dark themes. All
pixels match in the original page body after the layout settles.

Both galleries expand the reasoning and tool payloads and show the citation
preview and context breakdown. Selecting a suggestion fills the draft.
The composer selects a model, removes an attachment and shows the attach toast.
Send, Stop and Regenerate update the conversation and plan. A response
finishes while another page is visible. While the response runs, elapsed time
and loader phase can differ between captures.

Native tests cover input ownership, horizontal focus, layout changes and
the composer recorder. The FPAS gallery contract also runs in software and
OpenGL and checks text input, voting and the response clock.

See the [response clock](../../ui/gui/ai/README.md#response-clock) and the
[native findings](../cui-upstream-bugs.md#cui-107).

## Audio evidence

The reference is [CUI revision `6864776`](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1119).
The tour button sits above a wrapping row. The synth and mixer use the same
native controls, grouping, padding and gaps. The Cutoff knob, master meter
and Bass fader anchor the three guided tour steps.

The Linux X11 comparison uses the software renderer at scale 1. It holds
the synthetic signal at the first phase in both galleries. The 16 capture
pairs use widths of 1240, 920, 800, 760, 759, 568, 567, and 360 pixels at a
height of 2200 pixels in light and dark themes. The entire page body matches
after the layout settles.

All 14 captures in the input sequence match at the same signal phase. The
sequence covers the three tour steps, Back, Done and Escape. It also covers
knob drag and reset, pan adjustment, and fader drag, wheel, Home and End.
With a live signal, the master and channel meters change in both galleries.

The Audio gallery contract passes with the software renderer and OpenGL.
The native tests cover clip reset, focus ownership, drag release and all
three tour targets.

See the [signal clock](../../ui/gui/audio/README.md#signal-clock) and the
[native findings](../cui-upstream-bugs.md#cui-115).

## Files evidence

The reference uses the unchanged [CUI revision `6864776`](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1277).
The browser and picker read the same temporary fixture files in the two
processes. The file browser and dialog buttons precede the 14-node project
tree and terminal. The standalone FPAS path bar follows those sections.

The Linux X11 comparison uses the software renderer at scale 1. The 16 view
pairs use eight widths from 360 to 1240 pixels and both themes. Thirteen
pairs match exactly in the original page body. At 800 pixels, the corner
of the tree table differs in up to 30 pixels. At 360 pixels in dark mode,
the terminal border differs by one channel level on a single line.

The Open and Save as dialog panels match exactly. The underlying browser
and button row use the FPAS disabled appearance while a dialog is open.
Both galleries navigate folders, filter the listing, and return a path from
each dialog. The project tree matches for a stable right click, Copy name,
Info, moving to the top level, dragging into a folder, and Reset. The
terminal matches for `help`, `build`, history recall and `clear`.

The Files and Code gallery contract passes with the software renderer and
OpenGL. Native tests cover the keyboard menu, clipboard text, tree moves
and input ownership.

See the [Files gallery](../../ui/gui/files/README.md#files-gallery-page) and
the [native findings](../cui-upstream-bugs.md#cui-121).

## Developer tools evidence

The [reference page](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1510)
and the FPAS page use the same ten sections, maximum widths and sample data.
FPAS prepares the trace with the native sort and formats the edited query
with the native query description function. The schedule preview uses the
same fixed clock and UTC in both galleries.

The Linux X11 comparison uses the software renderer at scale 1. All 16
page bodies match pixel for pixel at widths of 360, 567, 568, 759, 760,
800, 920 and 1240 pixels in light and dark themes.

Fifteen of 16 input captures match. The sequence covers trace zoom,
measurement and lane folding, waterfall selection, dependency selection,
zoom and fit, commit and pipeline selection, diagnostic filters, the
key-value editor, query groups, schedule presets and keymap modifiers. The
one difference is the FPAS timing tooltip over a waterfall bar.

The Developer tools gallery contract passes in software and OpenGL.
Native tests also cover retained selection, popover focus, disabled input,
responsive layout and schedule buffer limits.

See the [Developer tools gallery](../../ui/gui/developer-tools/README.md#developer-tools-gallery-page)
and the [native findings](../cui-upstream-bugs.md#cui-128).

## More developer tools evidence

The [reference page](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1684)
and the FPAS page use the same section order, maximum widths and gaps.
The HTTP section groups the request bar, parameters and response status.
FPAS uses the CUI query functions for the URL and parameter edits.

The Linux X11 comparison uses the software renderer at scale 1. It covers
widths of 360, 567, 568, 759, 760, 800, 920 and 1240 pixels in light and dark
themes. The original example uses two different domains in the regex
sample. The comparison uses the FPAS sample in both galleries. All 16
settled page bodies then match pixel for pixel.

All 18 input captures match at 1240 pixels. The sequence covers test suite
folding and filtering, stack disclosure, blame hover, and coverage sorting
and folding. It also covers regex edits, the request method menu, URL and
parameter edits, endpoint selection and filtering, rollout and flag
controls, uptime hover, and histogram range selection and clearing.

The Developer tools gallery contract passes in software and OpenGL.
Focused tests also cover keyboard focus, hidden rows, UTF-8 labels,
responsive layouts and URL buffer limits.

See the [More developer tools gallery](../../ui/gui/more-developer-tools/README.md#more-developer-tools-gallery-page)
and the [native findings](../cui-upstream-bugs.md#cui-137).

## Instruments evidence

The [reference page](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1716)
and the FPAS page use the same three sections. The readouts wrap in a row
with 40 logical units of spacing. The chart and track share the time range,
and the track draws the axis. The dot-matrix display shows the bedtime
status from the same sample model.

The Linux X11 comparison uses the software renderer at scale 1. It covers
widths of 360, 567, 568, 759, 760, 800, 920 and 1240 pixels in light and dark
themes. All 16 page bodies match pixel for pixel with reduced motion.

The input sequence covers chart hover and adding a cup, dragging a marker,
and moving a marker by a quarter hour with the keyboard. It also covers
double-click removal, clearing the track, adding to an empty track and the
eight-cup limit. Eight of nine captures match. The empty FPAS readout
displays 0. CUI displays 5+ in that state.

The gallery model contract and widget checks cover the empty readout,
chart readout messages and track changes with software and OpenGL. The
native checks cover pointer release, popup input, text alignment and
marquee wakeups.

See the [Instruments gallery](../../ui/gui/instruments/README.md) and the
[native findings](../cui-upstream-bugs.md#cui-150).

## Creative controls evidence

The [reference page](https://github.com/yappologistic/CUI/blob/6864776f1a77b920a26f4ec51f5c4cf1be0b04a6/examples/gallery_more.c#L1765)
and the FPAS page use the same section order and sample values. Both wrap
the status tags and use two grids with a minimum column width of 260
logical units. Each grid stacks its controls when the column width is
too small.

The Linux X11 comparison uses the software renderer at scale 1 with reduced
motion. It checks eight widths from 360 to 1240 pixels in light and dark
themes. At a height of 2300 pixels, the captures include the whole page.
All 16 settled page bodies match pixel for pixel.

All 24 input captures match at 1240 pixels. The sequence covers removing
and restoring tags, slider changes and tool actions, swatch locks and
shuffle, aspect selection, background and export options, and seed controls.
The seed stays fixed when the dice is disabled.

The gallery contract passes in software and OpenGL. Focused tests cover
right-to-left keyboard navigation, popup input ownership, coalesced drag
release, hidden parts and reduced motion.

See the [Creative controls gallery](../../ui/gui/creative-controls/README.md)
and the [native findings](../cui-upstream-bugs.md#cui-156).

## Canvas and bars evidence

Both galleries use the same section order, node specimens and picture. The
graph layers the node editor, zoom rail, cursors and presence pill. The
result row wraps the card, floating toolbar and anchored file selection.
The FPAS gallery also has a separate Presence avatars example after those
sections.

The Linux X11 comparison uses the software renderer at scale 1. It checks
the original page body at eight widths from 360 to 1240 pixels in both
themes. All 16 settled page bodies match pixel for pixel. Reduced motion
and the initial collaborator positions keep the frames comparable.
Separate runs with the original clocks check cursor movement.

The comparison also checks file selection and Escape, toolbar modes, style
and color changes, Delete confirmation, zoom, toggles and node dragging.
The 28 settled captures after input match at 1240 pixels. At 360 pixels the
toolbar and its overflow menu also match before opening, while open and
after Escape closes it.

Fit uses the original rectangle at startup. After a node moves beyond that
rectangle, FPAS includes its current bounds. CUI clips part of the moved
node in the corresponding capture. The gallery contract passes with the
software and OpenGL renderers.

See the [Canvas and bars gallery](../../ui/gui/canvas-and-bars/README.md)
and the [native findings](../cui-upstream-bugs.md#cui-160).

## Voice evidence

Both galleries show the incoming voice note and recorder, the voice session
card and two waveforms in that order. The content beneath each section
heading fills the available width up to 620 logical points. FPAS follows
these sections with a standalone voice orb.

The Linux X11 comparison uses the software renderer at scale 1 and reduced
motion. All 16 settled page bodies match at eight widths from 360 to 1240
pixels in light and dark themes. Both hosts start at the same demo time so
the timeline, waveform and transcript represent the same samples.

At 1240 pixels, all 17 captures after input or clock steps match. The
sequence checks playback, speed, seeks, recording, Escape, Voice Send,
mute and call restart. It also checks the transcript and waveform during
thinking and speaking. The pointer stays inside the window until the
button consumes its release. [CUI-174](../cui-upstream-bugs.md#cui-174)

With their own clocks, both galleries advance the transcript and waveform
over time. The Voice and AI gallery contract passes in software and OpenGL.

See the [Voice gallery](../../ui/gui/voice/README.md) and the
[native findings](../cui-upstream-bugs.md#cui-171).
