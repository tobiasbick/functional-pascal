#include "../../fpas_cui.h"

int fpas_cui_ruler_slider(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, int major_every, float spacing, float width,
    const char *format, const char *unit, int enabled, int focus, int *track) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0x2017u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    if (!allowed) {
        cui__ruler_state *state = cui__state_lazy(ui, group, sizeof *state, false);
        if (state) state->dragging = false;
    }

    const cui_ruler_opts options = {
        .step = step, .major_every = major_every, .spacing = spacing,
        .width = width > 0 ? CUI_PX(width) : CUI_FILL,
        .format = cui__frame_strdup(ui, format, -1), .unit = unit
    };
    float next = *value;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_ruler_slider_ex(ui, cui__frame_printf(ui, "x%s", id),
        &next, minimum, maximum, &options);
    cui_pop_disabled(ui);

    const int root = fpas_find(ui, group);
    fpas_literal(ui, root, label, CUI_C(ui, TEXT_SECONDARY));
    if (!allowed) ui->nodes[root].opacity = CUI__DISABLED;
    *track = fpas_find(ui, key);
    ui->last_id = key;
    fpas_own(ui, key, group, key ^ CUI__VERTICAL_WHEEL_STATE, 0);
    if (!allowed || !changed || !isfinite(next) || next == *value) return 0;
    *value = next;
    return 1;
}
