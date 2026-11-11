#include "../fpas_cui.h"

int fpas_cui_radio_button(cui_ctx *ui, const char *id, const char *label,
    const char *description, int selected, int enabled, int focus) {
    const int option = 0;
    if (enabled && focus) {
        const cui_id key = cui_make_id(ui, id) ^ 0x4AD10003u;
        ui->focus_next_request = cui__hash(&option, sizeof option, key);
    }
    int value = selected ? option : -1;
    const cui_check_opts options = {
        .description = description, .disabled = !enabled
    };
    const char *caption = cui__frame_printf(ui, "x%s", id);
    const bool changed = cui_radio_ex(ui, caption, &value, option, &options);
    const int row = fpas_find(ui, ui->last_id);
    fpas_literal(ui, row, label, CUI_C(ui, FOREGROUND));
    return changed;
}
