#include "../../fpas_cui.h"

static bool fpas_slider_propose_range(float *lower, float *upper, float next_lower,
    float next_upper, float minimum, float maximum) {
    if (!isfinite(next_lower) || !isfinite(next_upper)) return false;
    /* CUI converts both endpoints through normalized track coordinates on every frame. */
    const float span = maximum - minimum;
    if (next_lower == minimum + (*lower - minimum) / span * span) next_lower = *lower;
    if (next_upper == minimum + (*upper - minimum) / span * span) next_upper = *upper;
    next_lower = cui__clampf(next_lower, minimum, maximum);
    next_upper = cui__clampf(next_upper, next_lower, maximum);
    const bool adjusted = next_lower != *lower || next_upper != *upper;
    *lower = next_lower;
    *upper = next_upper;
    return adjusted;
}

int fpas_cui_range_slider(cui_ctx *ui, const char *id, const char *label, float *lower,
    float *upper, float minimum, float maximum, int enabled, int focus, int *track) {
    const cui_id group = cui_make_id(ui, id);
    const cui_id key = group ^ 0x52u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    float next_lower = *lower;
    float next_upper = *upper;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_range_slider(ui, cui__frame_printf(ui, "x%s", id),
        &next_lower, &next_upper, minimum, maximum);
    cui_pop_disabled(ui);
    *track = fpas_finish_slider(ui, group, key, label, allowed);
    return allowed && changed &&
        fpas_slider_propose_range(lower, upper, next_lower, next_upper, minimum, maximum);
}
