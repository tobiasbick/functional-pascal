#include "../fpas_cui.h"

int fpas_cui_hold_button(cui_ctx *ui, const char *id, const char *label, const char *icon,
                         int duration_ms, int variant, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id);
    if (enabled && focus) ui->focus_next_request = key;
    if (!enabled) {
        /* CUI samples the held state before constructing the disabled button. */
        if (ui->active == key) ui->active = 0;
        if (ui->focus == key) ui->focus = 0;
        cui__hold_state *state = cui_state(ui, key ^ 0x401Du, sizeof *state);
        memset(state, 0, sizeof *state);
    }
    const cui_hold_opts options = {
        .duration = (float)duration_ms / 1000.0f, .icon = icon, .variant = fpas_button_variant(variant)
    };
    cui_push_disabled(ui, !enabled);
    const bool confirmed = cui_hold_button_ex(ui, *label ? cui__frame_printf(ui, "x%s", id) : id, &options);
    cui_pop_disabled(ui);
    if (*label) fpas_literal(ui, ui->last_node, label, CUI_C(ui, FOREGROUND));
    return enabled && confirmed ? 1 : 0;
}
