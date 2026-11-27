// [Documentation](../../../../../docs/ui/gui/selection/checkbox.md)
#include "../../fpas_cui.h"

int fpas_cui_checkbox(cui_ctx *ui, const char *id, const char *label,
    const char *description, int checked_value, int enabled, int focus, int indeterminate, int hide_label) {
    if (enabled && focus) {
        const cui_id key = cui_make_id(ui, id) ^ 0xC0EC0001u;
        ui->focus_next_request = key;
    }
    bool value = checked_value != 0;
    const cui_check_opts options = {
        .description = description, .disabled = !enabled, .indeterminate = indeterminate != 0
    };
    const char *caption = hide_label ? id : cui__frame_printf(ui, "x%s", id);
    const bool changed = cui_checkbox_ex(ui, caption, &value, &options);
    const int row = fpas_find(ui, ui->last_id);
    if (!hide_label) fpas_literal(ui, row, label, CUI_C(ui, FOREGROUND));
    ui->last_node = row;
    return changed;
}
