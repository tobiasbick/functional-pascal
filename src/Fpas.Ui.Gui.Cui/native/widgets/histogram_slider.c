#include "../fpas_cui.h"

int fpas_cui_histogram_slider(cui_ctx *ui, const char *id, const char *label, const float *bins,
    int bin_count, float *lower, float *upper, float minimum, float maximum, float height,
    const char *format, int enabled, int focus, int *track) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0x52u, outer = group ^ 0x4157u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const cui_histogram_opts options = { .height = height, .format = format };
    float next_lower = *lower, next_upper = *upper;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_histogram_slider_ex(ui, cui__frame_printf(ui, "x%s", id), bins, bin_count,
        &next_lower, &next_upper, minimum, maximum, &options);
    cui_pop_disabled(ui);

    const int root = fpas_find(ui, outer);
    fpas_literal(ui, root, label, CUI_C(ui, FOREGROUND));
    if (!allowed) ui->nodes[root].opacity = CUI__DISABLED;
    *track = fpas_find(ui, key);
    ui->last_id = key;
    fpas_own(ui, key, group, outer ^ 1u, 0);
    return allowed && changed &&
        fpas_slider_propose_range(lower, upper, next_lower, next_upper, minimum, maximum);
}
