#include "../fpas_cui.h"

int fpas_cui_range_slider(cui_ctx *ui, const char *id, const char *label, float *lower,
    float *upper, float minimum, float maximum, int enabled, int focus, int *track) {
    const cui_id group = cui_make_id(ui, id);
    const cui_id key = group ^ 0x52u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const bool adjusting = allowed && fpas_slider_dragging(ui, key);
    float next_lower = *lower;
    float next_upper = *upper;
    cui_push_disabled(ui, !enabled);
    cui_range_slider(ui, cui__frame_printf(ui, "x%s", id), &next_lower, &next_upper, minimum, maximum);
    cui_pop_disabled(ui);
    *track = fpas_finish_slider(ui, group, key, label, allowed);
    if (!adjusting || !isfinite(next_lower) || !isfinite(next_upper)) return 0;
    next_lower = cui__clampf(next_lower, minimum, maximum);
    next_upper = cui__clampf(next_upper, next_lower, maximum);
    const bool changed = next_lower != *lower || next_upper != *upper;
    *lower = next_lower;
    *upper = next_upper;
    return changed;
}
