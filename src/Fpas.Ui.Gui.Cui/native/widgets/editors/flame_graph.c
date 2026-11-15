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
    cui_push_disabled(ui, !enabled);
    const cui_flame_result result = cui_flame_graph_ex(ui, id, spans, count, &range, &view->selected, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    *view = (fpas_flame_view){range.start, range.end, range.scroll, view->selected, result.zoomed};
    return enabled && (result.view_changed || result.selection_changed || result.zoomed >= 0);
}
