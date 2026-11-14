#include "../fpas_cui.h"

void fpas_cui_status_bar_begin(cui_ctx *ui, const char *id, int border) {
    const cui_status_bar_opts opts = { .no_border = !border };
    cui_status_bar_begin(ui, id, &opts);
}

void fpas_cui_status_bar_end(cui_ctx *ui) {
    const int node = cui__current(ui);
    cui_status_bar_end(ui);
    const cui_id key = ui->nodes[node].id;
    fpas_own(ui, key, key ^ 0x5BA8u, 0, 0);
    ui->last_node = node;
    ui->last_id = key;
}
void fpas_cui_status_bar_spacer(cui_ctx *ui) { cui_status_bar_spacer(ui); }

void fpas_cui_status_bar_text(cui_ctx *ui, const char *text, const char *icon, int tone) {
    const cui_status_item_opts opts = { .icon = icon, .tone = (cui_tone)tone };
    cui_status_bar_text_ex(ui, text, &opts);
}

int fpas_cui_status_bar_item(cui_ctx *ui, const char *title, const char *icon, int tone,
    const char *tooltip, int enabled, int focus, int *node) {
    const cui_status_item_opts opts = { .icon = icon, .tone = (cui_tone)tone,
        .tooltip = tooltip, .disabled = !enabled };
    const cui_id key = cui_make_id(ui, title);
    const cui_id previous = ui->focus_next_request;
    if (enabled && focus && !cui_is_disabled(ui)) ui->focus_next_request = key;
    const bool clicked = cui_status_bar_item_ex(ui, title, &opts);
    *node = fpas_find(ui, key);
    if (*node < 0 && ui->focus_next_request == key) ui->focus_next_request = previous;
    return clicked;
}

void fpas_cui_status_bar_progress(cui_ctx *ui, float value, const char *label, float width, int indeterminate) {
    const cui_status_progress_opts opts = { .label = label, .width = width, .indeterminate = indeterminate != 0 };
    cui_status_bar_progress_ex(ui, value, &opts);
}
