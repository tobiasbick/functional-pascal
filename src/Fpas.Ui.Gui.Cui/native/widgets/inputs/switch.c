#include "../../fpas_cui.h"

int fpas_cui_switch(cui_ctx *ui, const char *id, const char *label,
    const char *description, int on, int enabled, int focus) {
    if (enabled && focus) {
        const cui_id key = cui_make_id(ui, id) ^ 0x5D17C002u;
        ui->focus_next_request = key;
    }
    bool value = on != 0;
    const cui_check_opts options = {
        .description = description, .disabled = !enabled
    };
    const char *caption = cui__frame_printf(ui, "x%s", id);
    const bool changed = cui_switch_ex(ui, caption, &value, &options);
    const int row = fpas_find(ui, ui->last_id);
    fpas_literal(ui, row, label, CUI_C(ui, FOREGROUND));
    return changed;
}
