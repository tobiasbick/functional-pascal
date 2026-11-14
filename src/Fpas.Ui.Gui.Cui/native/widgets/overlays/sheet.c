#include "../../fpas_cui.h"

int fpas_cui_sheet_begin(cui_ctx *ui, const char *id, const char *title, const char *description,
                         int side, float size, int open, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id) ^ 0x5EE7u;
    int panel;
    return fpas_modal_begin(ui, key, title, description, size, 1, (cui_side)side,
        open != 0, enabled != 0, false, true, focus != 0, &panel);
}

void fpas_cui_sheet_end(cui_ctx *ui, int base) {
    fpas_modal_end(ui, 1, base);
}
