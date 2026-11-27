// [Documentation](../../../../../docs/ui/gui/editing-tools/timeline-editor.md)
#include "../../fpas_cui.h"

typedef struct { int track, key; bool scrubbing; } fpas_timeline_pointer;

int fpas_cui_timeline_editor(cui_ctx *ui, const char *id, cui_timeline_track *tracks, int count, fpas_timeline_view *view, const fpas_timeline_options *settings) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id pointer = key ^ 0x71A6BEu;
    fpas_own(ui, key, key ^ 0x71AEu, pointer, 0);
    const bool enabled = (settings->flags & 1) && !cui_is_disabled(ui);
    if (enabled && (settings->flags & 8)) ui->focus_next_request = key;
    const int prior_track = view->track, prior_key = view->key;
    const float prior_in = view->in, prior_out = view->out;
    cui_timeline_view range = {view->start, view->end, view->scroll};
    const cui_timeline_editor_opts options = { .start = settings->start, .end = settings->end,
        .fps = settings->fps, .ruler = settings->ruler, .view = &range,
        .in = view->range ? &view->in : NULL, .out = view->range ? &view->out : NULL,
        .current_track = &view->track, .current_key = &view->key,
        .width = fpas_extent(settings->width_kind, settings->width), .height = settings->height,
        .row_height = settings->row_height, .track_width = settings->track_width,
        .toggles = (settings->flags & 2) != 0, .free_drag = (settings->flags & 4) != 0 };
    cui_push_disabled(ui, !enabled);
    const cui_timeline_editor_result result = cui_timeline_editor_ex(ui, id, tracks, count, &view->time, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    view->start = range.start;
    view->end = range.end;
    view->scroll = range.scroll;
    view->activity = (result.changed ? 1 : 0) | (result.time_changed ? 2 : 0) |
        (result.scrubbing ? 4 : 0) | (result.selection_changed ? 8 : 0) |
        (result.view_changed ? 16 : 0) | (result.keys_moved ? 32 : 0);
    view->added_track = result.added_track;
    view->added_key = result.added_key;
    view->removed_keys = result.removed_keys;
    view->removed_clips = result.removed_clips;
    view->clip_track = result.clip_track;
    view->clip = result.clip;
    view->toggled = result.toggled;
    view->hovered_track = result.hovered_track;
    view->hovered_key = result.hovered_key;
    fpas_timeline_pointer *previous = cui_state(ui, pointer, sizeof *previous);
    const bool pointer_changed = previous->scrubbing != result.scrubbing ||
        previous->track != result.hovered_track + 1 || previous->key != result.hovered_key + 1;
    if (enabled) *previous = (fpas_timeline_pointer){ result.hovered_track + 1, result.hovered_key + 1, result.scrubbing };
    const bool changed = result.changed || result.time_changed || result.view_changed || result.selection_changed ||
        view->track != prior_track || view->key != prior_key || view->in != prior_in || view->out != prior_out ||
        pointer_changed;
    return enabled ? (changed ? 1 : 0) | (result.play_toggled ? 2 : 0) : 0;
}
