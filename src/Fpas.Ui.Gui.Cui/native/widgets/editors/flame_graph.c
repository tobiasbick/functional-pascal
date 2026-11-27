// [Documentation](../../../../../docs/ui/gui/editing-tools/flame-graph.md)
#include "../../fpas_cui.h"

/** Reads native geometry for one span from the most recently presented flame graph. */
CUI_API cui_rect fpas_cui_flame_span_rect(cui_ctx *ui, const char *id, double start, double end, int depth) {
    const cui_id key = cui_make_id(ui, id);
    if (fpas_find(ui, key) < 0) return (cui_rect){0};
    const cui_flame_span span = {.start = start, .end = end, .depth = depth};
    return cui_flame_span_rect(ui, key, &span);
}

int fpas_cui_flame_graph(cui_ctx *ui, const char *id, const cui_flame_span *spans, int count, fpas_flame_view *view, double minimum, float height, float row_height, int width_kind, float width, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id pointer = key ^ 0xF1A3E4u;
    fpas_own(ui, key, key ^ 0xF1A3u, pointer, 0);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    const cui_flame_opts options = { .width = fpas_extent(width_kind, width), .height = height,
        .row_height = row_height, .min_range = minimum };
    cui_flame_view range = {view->start, view->end, view->scroll};
    cui_push_disabled(ui, !enabled);
    const cui_flame_result result = cui_flame_graph_ex(ui, id, spans, count, &range, &view->selected, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    int *hover = cui_state(ui, pointer, sizeof *hover);
    const bool hover_changed = result.hovered + 1 != *hover;
    if (enabled) *hover = result.hovered + 1;
    *view = (fpas_flame_view){range.start, range.end, range.scroll, view->selected, result.zoomed,
        result.hovered, result.view_changed, result.selection_changed};
    return enabled && (result.view_changed || result.selection_changed || result.zoomed >= 0 || hover_changed);
}
