// [Documentation](../../../../../docs/ui/gui/capture/signature-pad.md)
#include "../../fpas_cui.h"

void fpas_cui_signature_clear(cui_ctx *ui, const char *id) {
    const cui_id key = cui_make_id(ui, id);
    cui__sig_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (!state) return;

    /* Clearing the points also ends any stroke still held by the pointer. */
    state->drawing = false;
    if (ui->active == (key ^ 0x5161u)) ui->active = 0;
    cui_signature_clear(ui, id);
}

/** Draws native ink and exposes the active pad and clear button nodes. */
int fpas_cui_signature_pad(cui_ctx *ui, const char *id, const char *placeholder, int width_kind,
    float width, float height, int ink_palette, uint32_t rgba, int enabled, int focus_clear,
    int *node, int *clear) {
    const cui_id key = cui_make_id(ui, id), canvas = key ^ 0x5161u;
    cui_push_id(ui, id);
    const cui_id clear_key = cui_make_id(ui, "Clear");
    cui_pop_id(ui);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus_clear) ui->focus_next_request = clear_key;
    if (!allowed) {
        cui__sig_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->drawing = false;
    }
    const cui_signature_opts options = {
        .width = fpas_extent(width_kind, width),
        .height = height, .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL,
        .ink = fpas_chart_color(ink_palette == -1 && (rgba >> 24) == 0 ? 0 : ink_palette, rgba)
    };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_signature_pad_ex(ui, id, &options);
    cui_pop_disabled(ui);
    *node = ui->last_node = fpas_find(ui, key);
    *clear = fpas_find(ui, clear_key);
    fpas_own(ui, key, canvas, clear_key, 0);
    return allowed && changed;
}
