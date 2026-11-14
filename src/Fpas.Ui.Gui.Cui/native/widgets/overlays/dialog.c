#include "../../fpas_cui.h"

int fpas_cui_dialog_begin(cui_ctx *ui, const char *id, const char *title, const char *description,
    float width, int open, int enabled, int persistent, int close_button, int focus, int *panel) {
    const cui_id key = cui_make_id(ui, id) ^ 0xD1A106u;
    return fpas_modal_begin(ui, key, title, description, width, 0, CUI_SIDE_RIGHT,
        open != 0, enabled != 0, persistent != 0, close_button != 0, focus != 0, panel);
}

void fpas_cui_dialog_end(cui_ctx *ui, int base) {
    fpas_modal_end(ui, 0, base);
}
