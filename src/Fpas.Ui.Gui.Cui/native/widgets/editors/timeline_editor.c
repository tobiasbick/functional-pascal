#include "../../fpas_cui.h"

int fpas_cui_timeline_editor(cui_ctx *ui, const char *id, cui_timeline_track *tracks, int count, fpas_timeline_view *view, const fpas_timeline_options *settings) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0x71AEu, 0, 0);
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
    const cui_id wheel = ui->scroll_target;
    const cui_id hot = ui->hot;
    if (!enabled) {
        /* CUI-039 (docs/future/cui-upstream-bugs.md#cui-039): raw wheel and middle-button pan bypass cui__interact. */
        if (wheel == (key ^ 0x71AE5C01u)) ui->scroll_target = 0;
        if (hot == key) ui->hot = 0;
        cui__tl_state *state = cui__state_lazy(ui, key ^ 0x71AEu, sizeof *state, false);
        if (state) state->mode = TL_IDLE;
    }
    cui_push_disabled(ui, !enabled);
    const cui_timeline_editor_result result = cui_timeline_editor_ex(ui, id, tracks, count, &view->time, &options);
    cui_pop_disabled(ui);
    if (!enabled) {
        ui->scroll_target = wheel;
        ui->hot = hot;
    }
    fpas_creative_node(ui, root, key);
    view->start = range.start;
    view->end = range.end;
    view->scroll = range.scroll;
    const bool changed = result.changed || result.time_changed || result.view_changed || result.selection_changed ||
        view->track != prior_track || view->key != prior_key || view->in != prior_in || view->out != prior_out;
    return enabled ? (changed ? 1 : 0) | (result.play_toggled ? 2 : 0) : 0;
}
