#ifndef FPAS_CUI_H
#define FPAS_CUI_H

#include <cui.h>
#include "widgets/charts/sankey.h"
#include "widgets/charts/bar_list.h"
#include "widgets/charts/activity_rings.h"
#include "widgets/charts/scatter_chart.h"
#include "widgets/charts/radial_gauge.h"
#include "widgets/charts/candlestick.h"
#include "widgets/developer/code_block.h"
#include "widgets/developer/json_view.h"
#include "widgets/display/readout.h"
#include "widgets/display/dot_display.h"
#include "widgets/charts/dot_chart.h"
#include "widgets/charts/event_track.h"
#include "widgets/motion/motion_text.h"
#include "widgets/navigation/path_bar.h"
#include "widgets/files/file_browser.h"
#include "widgets/files/file_picker.h"
#include "widgets/planning/week_calendar.h"
#include "widgets/planning/month_calendar.h"
#include "widgets/planning/gantt.h"
#include "widgets/planning/kanban.h"
#include "widgets/feedback/status_tag.h"
#include "widgets/inputs/ratio_slider.h"
#include "widgets/inputs/gradient_slider.h"
#include "widgets/inputs/step_slider.h"
#include "widgets/pickers/palette_strip.h"
#include "widgets/pickers/aspect_picker.h"
#include "widgets/actions/option_action.h"
#include "widgets/inputs/seed_field.h"
#include "widgets/motion/shimmer_text.h"
#include "widgets/motion/typewriter.h"
#include "widgets/motion/text_reveal.h"
#include "widgets/motion/word_rotate.h"
#include "widgets/motion/number_roll.h"
#include "widgets/motion/gradient_text.h"
#include "widgets/motion/spotlight.h"
#include "widgets/feedback/loader.h"
#include "widgets/navigation/dock.h"
#include "widgets/audio/control_tile.h"
#include "widgets/audio/waveform.h"
#include "widgets/navigation/tab_bar.h"
#include "widgets/audio/now_playing.h"
#include "widgets/conversation/chat_message.h"
#include "widgets/conversation/chat_divider.h"
#include "widgets/conversation/typing_indicator.h"
#include "widgets/conversation/comment.h"
#include "widgets/conversation/presence.h"
#include "widgets/feedback/counter.h"
#include "widgets/feedback/ticker.h"
#include "widgets/feedback/status_spinner.h"
#include "widgets/feedback/skeleton_group.h"
#include "widgets/charts/sparkline.h"
#include "widgets/media/media_image.h"
#include "widgets/media/carousel.h"
#include "widgets/containers/card_stack.h"
#include "widgets/media/image_viewer.h"
#include "widgets/media/image_cropper.h"
#include "widgets/media/image_compare.h"
#include "widgets/audio/voice_wave.h"
#include "widgets/audio/voice_note.h"
#include "widgets/audio/voice_session.h"
#include "widgets/audio/voice_orb.h"
#include "widgets/ai/composer.h"
#include "widgets/ai/reasoning.h"
#include "widgets/ai/tool_call.h"
#include "widgets/ai/agent_plan.h"
#include "widgets/ai/agent_status.h"
#include "widgets/ai/source_list.h"
#include "widgets/ai/citation.h"
#include "widgets/ai/context_meter.h"
#include "widgets/ai/response_actions.h"
#include "widgets/ai/prompt_suggestions.h"
#include "widgets/ai/model_picker.h"
#include "widgets/developer/developer_data.h"
#include "widgets/developer/problems.h"
#include "widgets/developer/pipeline.h"
#include "widgets/developer/keymap.h"
#include "widgets/developer/commit_graph.h"
#include "widgets/developer/dependency_graph.h"
#include "widgets/developer/network_waterfall.h"
#include "widgets/developer/trace_viewer.h"
#include "widgets/developer/test_explorer.h"
#include "widgets/developer/stack_trace.h"
#include "widgets/developer/response_status.h"
#include "widgets/developer/uptime.h"
#include "widgets/charts/time_histogram.h"
#include "widgets/developer/coverage.h"
#include "widgets/developer/blame.h"
#include "widgets/developer/api_explorer.h"
#include "widgets/developer/key_value_editor.h"
#include "widgets/developer/query_builder.h"
#include "widgets/pickers/schedule_picker.h"
#include "widgets/developer/regex_tester.h"
#include "widgets/developer/request_bar.h"
#include "widgets/developer/feature_flags.h"
#include "widgets/canvas/format_bar.h"
#include "widgets/canvas/live_cursors.h"
#include "widgets/canvas/presence_pill.h"
#include "widgets/canvas/selection_ring.h"
#include "widgets/canvas/canvas_controls.h"
#include "widgets/canvas/result_card.h"
#include "widgets/canvas/selection_bar.h"
#include "widgets/desktop/toolbar.h"
#include "widgets/desktop/menu_bar.h"
#include "widgets/desktop/status_bar.h"
#include "widgets/desktop/document_tabs.h"
#include "widgets/desktop/split_view.h"
#include "widgets/desktop/dockspace.h"
#include "widgets/editors/node_editor.h"
#include "widgets/display/texts.h"
#include "widgets/display/icon.h"
#include "widgets/display/emoji.h"
#include "widgets/display/keycap.h"
#include "widgets/media/image.h"
#include "widgets/canvas/canvas.h"
#include "widgets/overlays/veil.h"
#include "widgets/display/markdown.h"
#include "widgets/editors/code_editor.h"
#include "widgets/developer/diff_view.h"
#include "widgets/developer/hex_view.h"
#include "widgets/developer/log_console.h"
#include "widgets/developer/terminal.h"
#include "widgets/editors/property_inspector.h"
#include "widgets/editors/flame_graph.h"
#include "widgets/editors/curve_editor.h"
#include "widgets/editors/gradient_editor.h"
#include "widgets/editors/timeline_editor.h"
#include "widgets/charts/heatmap.h"
#include "widgets/charts/calendar_heatmap.h"
#include "widgets/charts/treemap.h"
#include "widgets/charts/radar_chart.h"
#include "widgets/charts/funnel.h"
#include "widgets/charts/split_bar.h"
#include "widgets/charts/waffle.h"
#include "widgets/charts/line_chart.h"
#include "widgets/charts/bar_chart.h"
#include "widgets/charts/donut_chart.h"
#include "widgets/charts/gauge.h"
#include "widgets/charts/pattern_meter.h"
#include "widgets/inputs/inline_edit.h"
#include "widgets/inputs/scrub_field.h"
#include "widgets/inputs/money_field.h"
#include "widgets/inputs/shortcut_recorder.h"
#include "widgets/feedback/feedback.h"
#include "widgets/feedback/banner.h"
#include "widgets/feedback/countdown.h"
#include "widgets/feedback/checklist.h"
#include "widgets/overlays/tour.h"
#include "widgets/navigation/table_of_contents.h"
#include "widgets/containers/grid.h"
#include "widgets/containers/card.h"
#include "widgets/feedback/badge.h"
#include "widgets/charts/metric.h"
#include "widgets/media/avatar_group.h"
#include "widgets/containers/empty_state.h"
#include "widgets/collections/list_item.h"
#include "widgets/collections/tree.h"
#include "widgets/collections/virtual_list.h"
#include "widgets/collections/reorderable_list.h"
#include "widgets/collections/table.h"
#include "widgets/collections/data_grid.h"
#include "widgets/collections/tree_table.h"
#include "widgets/collections/filter_bar.h"
#include "widgets/overlays/command_palette.h"
#include "widgets/navigation/scroll_anchor.h"
#include "widgets/containers/disclosure.h"
#include "host/host_widgets.h"
#include "widgets/actions/button.h"
#include "widgets/actions/button_group.h"
#include "widgets/navigation/sidebar.h"
#include "widgets/overlays/sheet.h"
#include "widgets/overlays/dialog.h"
#include "widgets/actions/action_footer.h"
#include "widgets/actions/icon_button.h"
#include "widgets/actions/copy_button.h"
#include "widgets/actions/hold_button.h"
#include "widgets/actions/confirm_button.h"
#include "widgets/actions/status_button.h"
#include "widgets/actions/segmented.h"
#include "widgets/actions/toggle_group.h"
#include "widgets/inputs/checkbox.h"
#include "widgets/inputs/switch.h"
#include "widgets/inputs/radio_button.h"
#include "widgets/inputs/radio_cards.h"
#include "widgets/inputs/slider.h"
#include "widgets/inputs/range_slider.h"
#include "widgets/files/file_dialog.h"
#include "widgets/files/dropzone.h"
#include "widgets/media/avatar.h"
#include "widgets/files/avatar_upload.h"
#include "widgets/files/signature_pad.h"
#include "widgets/audio/knob.h"
#include "widgets/audio/fader.h"
#include "widgets/audio/level_meter.h"
#include "widgets/inputs/ruler_slider.h"
#include "widgets/inputs/histogram_slider.h"
#include "widgets/inputs/chip.h"
#include "widgets/inputs/rating.h"
#include "widgets/navigation/tabs.h"
#include "widgets/navigation/breadcrumb.h"
#include "widgets/navigation/pagination.h"
#include "widgets/navigation/stepper.h"
#include "widgets/feedback/toast.h"
#include "widgets/overlays/tooltip.h"
#include "widgets/display/link.h"
#include "widgets/overlays/popover.h"
#include "widgets/overlays/hover_card.h"
#include "widgets/overlays/confirm_popover.h"
#include "widgets/inputs/text_field.h"
#include "widgets/inputs/search_field.h"
#include "widgets/inputs/text_area.h"
#include "widgets/inputs/tag_input.h"
#include "widgets/inputs/one_time_code.h"
#include "widgets/inputs/password_strength.h"
#include "widgets/inputs/select.h"
#include "widgets/inputs/number_field.h"
#include "widgets/actions/split_button.h"
#include "widgets/overlays/menu.h"
#include "widgets/conversation/reactions.h"
#include "widgets/overlays/user_menu.h"
#include "widgets/actions/speed_dial.h"
#include "widgets/actions/swipe_item.h"
#include "widgets/inputs/multi_select.h"
#include "widgets/pickers/calendar.h"
#include "widgets/pickers/date_picker.h"
#include "widgets/pickers/color_picker.h"
#include "widgets/pickers/emoji_picker.h"
#include "widgets/pickers/date_range_calendar.h"
#include "widgets/pickers/date_range_picker.h"
#include "widgets/pickers/time_picker.h"
#include "widgets/pickers/wheel_picker.h"
#include "widgets/pickers/time_dial.h"

/** Presents the completed current frame without waiting for more input.
 * @param app Live UI-thread application with a frame open.
 * @return Nonzero when a nonempty framebuffer was presented. */
CUI_API int fpas_cui_present(cui_app *app);

/** Pumps a frame, optionally keeping a close-requested window alive for cleanup.
 * @param app The owned application.
 * @param defer_close Nonzero to return a final drawable frame after close.
 * @return Zero on close, one for a frame, two for a deferred close request. */
CUI_API int fpas_cui_frame(cui_app *app, int defer_close);

/** Reads the current frame's last widget node.
 * @param ui Current UI-thread context.
 * @return The frame-local node index. */
CUI_API int fpas_cui_last_node(cui_ctx *ui);

/** Reads finalized layout for a tracked node.
 * @param ui Current UI-thread context after presentation.
 * @param node A node from that frame.
 * @return Its logical rectangle, or an empty rectangle for an invalid node. */
CUI_API cui_rect fpas_cui_node_rect(cui_ctx *ui, int node);

/** Finishes deferred paint without presenting before resource and window destruction.
 * @param app Live UI-thread application. Destroy it without opening or presenting another frame. */
CUI_API void fpas_cui_finish_frame_for_destroy(cui_app *app);

/** Begins a full-width column. Close it with cui_box_end.
 * @param ui Current UI-thread context.
 * @param padding Nonnegative logical padding.
 * @param gap Nonnegative logical child spacing. */
CUI_API void fpas_cui_column_begin(cui_ctx *ui, float padding, float gap);

/** Begins a full-width row. Close it with cui_box_end.
 * @param ui Current UI-thread context.
 * @param padding Nonnegative logical padding.
 * @param gap Nonnegative logical child spacing. */
CUI_API void fpas_cui_row_begin(cui_ctx *ui, float padding, float gap);

/** Requests focus for the next matching focusable control.
 * @param ui Current UI-thread context.
 * @param id Borrowed control identity, resolved in the current ID scope. */
CUI_API void fpas_cui_request_focus(cui_ctx *ui, const char *id);

/** Moves the synthetic pointer and refreshes hit testing in the open frame.
 * @param ui Current UI-thread context before drawing widgets.
 * @param x Logical horizontal coordinate.
 * @param y Logical vertical coordinate. */
CUI_API void fpas_cui_move_pointer(cui_ctx *ui, float x, float y);

/** Begins a scroll area. Close it with cui_scroll_end.
 * @param ui Current UI-thread context.
 * @param id Borrowed stable identity.
 * @param height Positive logical height, or zero to fill available space.
 * @param horizontal Nonzero to enable horizontal scrolling. */
CUI_API void fpas_cui_scroll_begin(cui_ctx *ui, const char *id, float height, int horizontal);

/** Sets the Win32 live-resize renderer. The callback draws on the UI thread.
 * @param app UI-thread application with no callback currently in flight.
 * @param render Renderer receiving the open CUI frame and user state, or NULL to detach.
 * @param user Borrowed state valid until detachment or application destruction. */
CUI_API void fpas_cui_set_resize_renderer(cui_app *app, void (*render)(cui_ctx *, void *), void *user);

/** Acquires the platform event sender on the UI thread.
 * @param app Live UI-thread application.
 * @return Owned wake target, or NULL if it could not be opened. */
CUI_API void *fpas_cui_wake_open(cui_app *app);

/** Posts an event to a live window without accessing its CUI state.
 * @param target Wake target whose lifetime is protected by the caller.
 * @return Nonzero if the event was posted. */
CUI_API int fpas_cui_wake_post(void *target);

/** Releases the sender before destroying its window. Serialize with posting.
 * @param target Owned wake target. */
CUI_API void fpas_cui_wake_close(void *target);

#endif
