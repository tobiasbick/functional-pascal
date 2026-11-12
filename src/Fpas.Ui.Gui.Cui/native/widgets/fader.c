#include "../fpas_cui.h"

int fpas_cui_fader(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, float reset, const char *format, const char *unit,
    int hide_value, float length, int horizontal, const float *ticks, int tick_count,
    const float *levels, int channels, int enabled, int focus, int *control) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0xFAD0u, wheel = group ^ 0xFAD1u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const bool literal_label = strstr(label, "##") != NULL;
    const cui_fader_opts options = {
        .label = cui__frame_strdup(ui, literal_label && !cui__display_len(label) ? "x" : label, -1),
        .format = format, .unit = unit, .step = step, .reset = reset,
        .disabled = !enabled, .hide_value = hide_value != 0,
        .length = length, .horizontal = horizontal != 0, .ticks = ticks, .tick_count = tick_count,
        .meter = levels, .meter_channels = channels
    };
    float next = *value;
    /* CUI normalizes and snaps even on a paint-only frame. Compare the input
       proposal with that baseline before changing the application model. */
    const float position = cui__dl_norm(*value, minimum, maximum);
    const float idle = cui__dl_value(position, minimum, maximum, step, false);
    cui_fader_ex(ui, id, &next, minimum, maximum, &options);
    const int root = fpas_find(ui, group);
    *control = fpas_find(ui, key);
    if (literal_label) fpas_text(ui, (horizontal ? ui->nodes[ui->nodes[root].first].first : ui->nodes[root].last), label);
    if (!hide_value && unit && strstr(unit, "##"))
        fpas_text(ui, (horizontal ? ui->nodes[ui->nodes[root].first].last : ui->nodes[root].first), cui__dl_text(ui, format, unit, next));
    ui->last_node = root;
    fpas_own(ui, key, group, wheel, 0);
    if (!allowed || !isfinite(next) || next == idle || next == *value) return 0;
    *value = next;
    return 1;
}
