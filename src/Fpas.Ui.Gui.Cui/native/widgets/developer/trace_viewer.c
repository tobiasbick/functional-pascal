#include "../../fpas_cui.h"

void fpas_cui_trace_viewer(cui_ctx *ui, const char *id, const cui_trace_track * tracks, int tracks_count, const cui_trace_span * spans, int spans_count, const cui_trace_marker * markers, int markers_count, double start, double stop, float scroll, double mark_start, double mark_end, int selection, float height, float row_height, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    cui_trace_view view = { start, stop, scroll, mark_start, mark_end };
    const cui_trace_opts o = { .width = fpas_extent(width_kind, width), .height = height, .row_height = row_height, .markers = markers, .marker_count = markers_count };
    cui_push_disabled(ui, !enabled);
    cui_trace_track *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)CUI_MAX(1, tracks_count));
    if (tracks_count > 0) memcpy(copy, tracks, sizeof *copy * (size_t)tracks_count);
    cui_trace_result r = cui_trace_view_ex(ui, id, copy, tracks_count, spans, spans_count, &view, &selection, &o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=selection;
    out->hover=r.hovered;
    /* CUI-036 (docs/future/cui-upstream-bugs.md#cui-036): native view_changed omits measurement edits. */
    const bool view_changed = r.view_changed || view.mark_start != mark_start || view.mark_end != mark_end;
    out->flags = r.selection_changed | (view_changed << 1);
    for (int i = 0; i < tracks_count; i++) {
        if (copy[i].collapsed != tracks[i].collapsed) {
            out->flags |= 4;
            out->action = i;
            out->count = copy[i].collapsed;
            break;
        }
    }
    out->x = view.start;
    out->y = view.end;
    out->z = view.scroll;
    out->a = view.mark_start;
    out->b = view.mark_end;
}
