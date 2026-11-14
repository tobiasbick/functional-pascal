#include "../../fpas_cui.h"

int fpas_cui_virtual_list_begin(cui_ctx *ui, const char *id, int count, float row_height,
    int kind, float height, int enabled, int *first, int *last) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_scroll_state *state = cui_state(ui, key, sizeof *state);
    /* CUI-023 (docs/future/cui-upstream-bugs.md#cui-023): clamp before CUI calculates the realized range. */
    state->content_h = (float)count * row_height;
    const float maximum = CUI_MAX(0, state->content_h - state->view_h);
    state->y = cui__clampf(state->y, 0, maximum);
    state->ty = cui__clampf(state->ty, 0, maximum);
    const float dx = ui->scroll_dx, dy = ui->scroll_dy;
    ui->scroll_dx = 0;
    if (!enabled) ui->scroll_dy = 0;
    cui_push_disabled(ui, !enabled);
    cui_virtual_list_begin(ui, id, count, row_height, fpas_extent(kind, height), first, last);
    cui_pop_disabled(ui);
    ui->scroll_dx = dx;
    if (!enabled) ui->scroll_dy = dy;
    cui_push_id(ui, id);
    return node;
}

void fpas_cui_virtual_list_end(cui_ctx *ui, int node) {
    cui_pop_id(ui);
    cui_virtual_list_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

void fpas_cui_virtual_row_begin(cui_ctx *ui, int index) { cui_push_id_int(ui, index); }
void fpas_cui_virtual_row_end(cui_ctx *ui) { cui_pop_id(ui); }
