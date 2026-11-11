#include "../fpas_cui.h"

int fpas_cui_one_time_code(cui_ctx *ui, const char *id, char *code, int length, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id);
    if (enabled && !cui_is_disabled(ui) && focus) ui->focus_next_request = key;
    cui_push_disabled(ui, !enabled);
    cui_otp_input(ui, id, code, length);
    cui_pop_disabled(ui);

    /* Escape releases focus after CUI has marked this row as the active editor. */
    if (ui->focus != key && ui->edit == cui__state_lazy(ui, key, sizeof(cui__otp_state), false))
        ui->edit = NULL;
    const int row = fpas_find(ui, key);
    ui->last_node = row;
    ui->last_id = key;
    return row;
}
