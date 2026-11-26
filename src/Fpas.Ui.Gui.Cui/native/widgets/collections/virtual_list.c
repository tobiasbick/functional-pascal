// [Documentation](../../../../../docs/ui/gui/data-display/virtual-list.md)
#include "../../fpas_cui.h"

/** Opens the native viewport and returns the rows this frame needs. */
int fpas_cui_virtual_list_begin(cui_ctx *ui, const char *id, int count, float row_height,
    int kind, float height, int enabled, float restore_offset, int *first, int *last) {
    const int list_capacity = (int)(sizeof ui->vl_stack / sizeof ui->vl_stack[0]);
    /* CUI opens a viewport and a row column. FPAS adds a list scope and an index scope. */
    if (ui->parent_depth > CUI_PARENT_STACK - 2 || ui->id_depth > CUI_ID_STACK - 2 ||
        ui->vl_depth >= list_capacity) {
        *first = *last = 0;
        return -1;
    }
    if (restore_offset >= 0) {
        cui_scroll_state *state = cui_state(ui, cui_make_id(ui, id), sizeof *state);
        state->y = state->ty = restore_offset;
    }
    const int node = ui->node_count;
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
