#include "../fpas_cui.h"

int fpas_cui_sheet_begin(cui_ctx *ui, const char *id, const char *title, const char *description,
                         int side, float size, int open, int enabled, int focus) {
    cui_id key = cui_make_id(ui, id) ^ 0x5EE7u;
    cui_push_id_int(ui, (int)key);
    cui_id close = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "Close"));
    cui_pop_id(ui);
    if (enabled && focus) ui->focus_next_request = close;

    bool visible = open != 0;
    bool blocked = visible && !enabled;
    if (blocked) cui_push_disabled(ui, true);
    bool draw = cui__modal_begin(ui, key, title, &visible, description, size,
                                !enabled, true, 1, (cui_side)side);
    if (blocked) cui_pop_disabled(ui);
    if (draw) {
        if (strstr(title, "##")) {
            int panel = ui->nodes[cui__current(ui)].parent;
            fpas_literal(ui, ui->nodes[panel].first, title, CUI_C(ui, FOREGROUND));
        }
        ui->last_node = fpas_find(ui, close);
        ui->last_id = close;
    }
    return (draw ? 1 : 0) | (open && !visible ? 2 : 0);
}

void fpas_cui_sheet_end(cui_ctx *ui, int base) {
    cui_sheet_end(ui);
    ui->last_node = base;
    ui->last_id = base >= 0 ? ui->nodes[base].id : 0;
}
