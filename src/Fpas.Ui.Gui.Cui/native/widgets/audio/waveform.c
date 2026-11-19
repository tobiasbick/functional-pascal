// [Documentation](../../../../../docs/ui/gui/motion/waveform.md)
#include "../../fpas_cui.h"

int fpas_cui_waveform(cui_ctx *ui, const char *id, const float *peaks, int count, float *progress,
    int width_kind, float width, float height, float bar, float gap, int palette, uint32_t rgba, int flags) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) != 0 && !cui_is_disabled(ui);
    const bool on_release = enabled && progress && (flags & 4);
    const float original = progress ? *progress : 0;
    cui__wf2_state *before = cui__state_lazy(ui, key, sizeof *before, false);
    if (before && (!enabled || !progress)) before->dragging = false;
    const bool dragging = before && before->dragging;
    const cui_rect bounds = cui__prev_rect(ui, key);
    const bool release = on_release && ui->mreleased[0] && bounds.w > 0 &&
        (dragging || (ui->mpressed[0] && ui->hot == key));
    if (release) *progress = cui__clampf((ui->mx - bounds.x) / bounds.w, 0, 1);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    const cui_waveform_opts options = { .width = fpas_extent(width_kind, width),
        .height = height, .bar = bar, .gap = gap, .color = fpas_chart_color(palette, rgba) };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_waveform_ex(ui, id, peaks, count, progress, &options);
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
