/* Common modal header and layout identity handling for native dialogs and sheets. */
static int fpas_modal_begin(cui_ctx *ui, cui_id key, const char *title, const char *description,
    float width, int kind, cui_side side, bool open, bool enabled, bool persistent,
    bool close_button, bool focus, int *panel) {
    cui_id close = 0;
    if (close_button) {
        cui_push_id_int(ui, (int)key);
        close = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "Close"));
        cui_pop_id(ui);
        if (enabled && focus) ui->focus_next_request = close;
    }

    bool visible = open;
    const bool blocked = visible && !enabled;
    if (blocked) cui_push_disabled(ui, true);
    const bool draw = cui__modal_begin(ui, key, title, &visible, description, width,
        persistent || !enabled, close_button, kind, side);
    if (blocked) cui_pop_disabled(ui);
    *panel = -1;
    if (draw) {
        const int current = cui__current(ui);
        *panel = kind == 1 ? ui->nodes[current].parent : current;
        if (strstr(title, "##"))
            fpas_literal(ui, ui->nodes[*panel].first, title, CUI_C(ui, FOREGROUND));
        ui->last_node = close_button ? fpas_find(ui, close) : *panel;
        ui->last_id = close_button ? close : ui->nodes[*panel].id;
    }
    return (draw ? 1 : 0) | (open && !visible ? 2 : 0);
}

static void fpas_modal_end(cui_ctx *ui, int kind, int base) {
    cui__modal_end(ui, kind);
    ui->last_node = base;
    ui->last_id = base >= 0 ? ui->nodes[base].id : 0;
}
