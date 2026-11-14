#include "../../fpas_cui.h"

int fpas_cui_flame_graph(cui_ctx *ui, const char *id, const cui_flame_span *spans, int count, fpas_flame_view *view, double minimum, float height, float row_height, int width_kind, float width, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0xF1A3u, 0, 0);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    const cui_flame_opts options = { .width = fpas_extent(width_kind, width), .height = height,
        .row_height = row_height, .min_range = minimum };
    cui_flame_view range = {view->start, view->end, view->scroll};
    const cui_id wheel = ui->scroll_target;
    if (!enabled) {
        /* CUI-039 (docs/future/cui-upstream-bugs.md#cui-039): raw wheel and in-progress gestures bypass cui__interact. */
        if (wheel == (key ^ 0xF1A5C011u)) ui->scroll_target = 0;
        cui__fl_state *state = cui__state_lazy(ui, key ^ 0xF1A3u, sizeof *state, false);
        if (state) state->mode = FL_IDLE;
    }
    cui_push_disabled(ui, !enabled);
    const cui_flame_result result = cui_flame_graph_ex(ui, id, spans, count, &range, &view->selected, &options);
    cui_pop_disabled(ui);
    if (!enabled) ui->scroll_target = wheel;
    fpas_creative_node(ui, root, key);
    *view = (fpas_flame_view){range.start, range.end, range.scroll, view->selected, result.zoomed};
    return enabled && (result.view_changed || result.selection_changed || result.zoomed >= 0);
}
