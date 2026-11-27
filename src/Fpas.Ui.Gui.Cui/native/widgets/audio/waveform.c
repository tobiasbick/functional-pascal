// [Documentation](../../../../../docs/ui/gui/motion/waveform.md)
#include "../../fpas_cui.h"

static void fpas_paint_focused_waveform(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_waveform(ui, node, data);
    cui__draw_focus_ring(ui, node->rect, node->radius, 1.0f);
}

int fpas_cui_waveform(cui_ctx *ui, const char *id, const float *peaks, int count, float *progress,
    int width_kind, float width, float height, float bar, float gap, int palette, uint32_t rgba, int flags) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) != 0 && !cui_is_disabled(ui);
    const bool interactive = enabled && progress;
    const bool on_release = interactive && (flags & 4);
    const float original = progress ? *progress : 0;
    cui__wf2_state *before = cui__state_lazy(ui, key, sizeof *before, false);
    if (before && (!interactive || !ui->window_focused)) before->dragging = false;
    float fraction = original;
    const bool release = interactive && fpas_drag_release(ui, key, before && before->dragging, &fraction);
    if (release) *progress = fraction;
    if (interactive && (flags & 2)) ui->focus_next_request = key;
    const cui_waveform_opts options = { .width = fpas_extent(width_kind, width),
        .height = height, .bar = bar, .gap = gap, .color = fpas_chart_color(palette, rgba) };
    cui_push_disabled(ui, !enabled);
    bool changed;
    if (interactive && fpas_drag_focus_ring(ui, key)) {
        /* CUI-099 (docs/future/cui-upstream-bugs.md#cui-099): preserve the native bars and focus ring. */
        const fpas_focus_cover cover = fpas_begin_focus_cover(ui);
        changed = cui_waveform_ex(ui, id, peaks, count, progress, &options);
        fpas_end_focus_cover(ui, cover);
        ui->nodes[node].paint = fpas_paint_focused_waveform;
    } else {
        changed = cui_waveform_ex(ui, id, peaks, count, progress, &options);
    }
    cui_pop_disabled(ui);
    fpas_motion_node(ui, node, key);
    if (release) return *progress != original;
    if (on_release) {
        const cui__wf2_state *after = cui__state_lazy(ui, key, sizeof *after, false);
        if (after && after->dragging) {
            /* CUI captured the pointer position for painting. Keep the transport at its own position. */
            *progress = original;
            return 0;
        }
    }
    return enabled && changed;
}
