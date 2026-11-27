// [Documentation](../../../../../docs/ui/gui/developer-tools/trace-viewer.md)
#include "../../fpas_cui.h"

static void fpas_paint_passive_trace(cui_ctx *ui, cui_node *node, void *data) {
    const int hits = ui->hit_count;
    cui__paint_trace(ui, node, data);
    ui->hit_count = hits;
}

void fpas_cui_trace_viewer(cui_ctx *ui, const char *id, const cui_trace_track * tracks, int tracks_count, const cui_trace_span * spans, int spans_count, const cui_trace_marker * markers, int markers_count, double start, double stop, float scroll, double mark_start, double mark_end, int selection, float height, float row_height, int width_kind, float width, int flags, fpas_dev_result * out, const cui_trace_link *links, int link_count, int *hovered_link) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), wheel = key ^ 0x7A1Du, state_key = key ^ 0x7A1Eu;
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    fpas_own(ui, key, state_key, wheel, 0);
    if (!enabled) {
        /* [CUI-128](../../../../../docs/future/cui-upstream-bugs.md#cui-128):
         * The native wheel and a pending gesture bypass the disabled scope. */
        if (ui->scroll_target == wheel) ui->scroll_target = 0;
        fpas_clear_interaction(ui, key);
        cui__tr_state *state = cui__state_lazy(ui, state_key, sizeof *state, false);
        if (state) {
            state->mode = TR_IDLE;
            state->fl.anim_on = false;
        }
    }
    *out = fpas_dev_empty();
    cui_trace_view view = { start, stop, scroll, mark_start, mark_end };
    const cui_trace_opts o = { .width = fpas_extent(width_kind, width), .height = height, .row_height = row_height, .markers = markers, .marker_count = markers_count,
        .links = links, .link_count = link_count };
    cui_push_disabled(ui, !enabled);
    cui_trace_track *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)CUI_MAX(1, tracks_count));
    if (tracks_count > 0) memcpy(copy, tracks, sizeof *copy * (size_t)tracks_count);
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = enabled && fpas_dev_save_keys(ui, key, keys);
    cui_trace_result r = cui_trace_view_ex(ui, id, copy, tracks_count, spans, spans_count, &view, &selection, &o);
    if (muted) memcpy(ui->kpressed, keys, sizeof ui->kpressed);
    cui_pop_disabled(ui);
    *hovered_link = r.hovered_link;
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    if (!enabled) {
        ui->nodes[root].flags &= ~CUI_NF_INTERACTIVE;
        if (ui->nodes[root].paint == cui__paint_trace)
            ui->nodes[root].paint = fpas_paint_passive_trace;
    }
    fpas_creative_node(ui, root, key);
    out->index=selection;
    out->hover=r.hovered;
    out->flags = r.selection_changed | (r.view_changed << 1);
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

cui_rect fpas_cui_trace_span_rect(cui_ctx *ui, const char *id, double start, double end, int track, int depth) {
    const cui_trace_span span = { .start = start, .end = end, .track = track, .depth = depth };
    return cui_trace_span_rect(ui, cui_make_id(ui, id), &span);
}
