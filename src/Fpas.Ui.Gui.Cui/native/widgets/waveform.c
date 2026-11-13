#include "../fpas_cui.h"

int fpas_cui_waveform(cui_ctx *ui, const char *id, const float *peaks, int count, float *progress,
    int width_kind, float width, float height, float bar, float gap, int palette, uint32_t rgba, int flags) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) != 0 && !cui_is_disabled(ui);
    if (enabled && (flags & 2)) ui->focus_next_request = key;
    if (!enabled) {
        cui__wf2_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->dragging = false;
    }
    const cui_waveform_opts options = { .width = fpas_extent(width_kind, width),
        .height = height, .bar = bar, .gap = gap, .color = fpas_chart_color(palette, rgba) };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_waveform_ex(ui, id, peaks, count, progress, &options);
    cui_pop_disabled(ui);
    fpas_motion_node(ui, node, key);
    return enabled && changed;
}
