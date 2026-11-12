#include "../fpas_cui.h"

int fpas_cui_knob(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, float reset, const char *format, const char *unit,
    int hide_value, float size, int bipolar, int log_scale, uint32_t rgba, int enabled, int focus, int *control) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0x4B0Bu, wheel = group ^ 0x4B0Cu;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const bool literal_label = strstr(label, "##") != NULL;
    const cui_knob_opts options = {
        .label = cui__frame_strdup(ui, literal_label && !cui__display_len(label) ? "x" : label, -1),
        .format = format, .unit = unit, .step = step, .reset = reset,
        .disabled = !enabled, .hide_value = hide_value != 0,
        .size = size, .bipolar = bipolar != 0, .log_scale = log_scale != 0,
        .color = { (uint8_t)rgba, (uint8_t)(rgba >> 8), (uint8_t)(rgba >> 16), (uint8_t)(rgba >> 24) }
    };
    float next = *value;
    /* CUI normalizes and snaps even on a paint-only frame. Compare the input
       proposal with that baseline before changing the application model. */
    const float position = cui__dl_pos(*value, minimum, maximum, log_scale != 0);
    const float idle = cui__dl_value(position, minimum, maximum, step, log_scale != 0);
    cui_knob_ex(ui, id, &next, minimum, maximum, &options);
    const int root = fpas_find(ui, group);
    *control = fpas_find(ui, key);
    if (literal_label) {
        const int caption = ui->nodes[root].last;
        fpas_text(ui, caption, label);
        const float width = ceilf(cui__text_width(ui, CUI_FONT_BODY, ui->nodes[caption].font_size, label, -1, false));
        const float diameter = ui->nodes[*control].w.value;
        ui->nodes[root].w = CUI_PX(CUI_MAX(ui->nodes[root].w.value, CUI_MIN(width, diameter * 2)));
    }
    if (!hide_value && unit && strstr(unit, "##"))
        fpas_text(ui, ui->nodes[*control].next, cui__dl_text(ui, format, unit, next));
    ui->last_node = root;
    fpas_own(ui, key, group, wheel, 0);
    if (!allowed || !isfinite(next) || next == idle || next == *value) return 0;
    *value = next;
    return 1;
}
