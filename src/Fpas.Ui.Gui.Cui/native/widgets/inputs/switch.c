// [Documentation](../../../../../docs/ui/gui/selection/switch.md)
#include "../../fpas_cui.h"

int fpas_cui_switch(cui_ctx *ui, const char *id, const char *label,
    const char *description, int on, int enabled, int focus, int hide_label) {
    if (enabled && focus) {
        const cui_id key = cui_make_id(ui, id) ^ 0x5D17C002u;
        ui->focus_next_request = key;
    }
    bool value = on != 0;
    const cui_check_opts options = {
        .description = description, .disabled = !enabled
    };
    const char *caption = hide_label ? id : cui__frame_printf(ui, "x%s", id);
    const bool changed = cui_switch_ex(ui, caption, &value, &options);
    const int row = fpas_find(ui, ui->last_id);
    if (!hide_label) fpas_literal(ui, row, label, CUI_C(ui, FOREGROUND));
    ui->last_node = row;
    return changed;
}
