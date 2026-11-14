#include "../../fpas_cui.h"

int fpas_cui_reorder_begin(cui_ctx *ui, const char *id, const int *order, int count, int enabled, int *proposed) {
    const int node = ui->node_count;
    int *copy = cui__frame_alloc(ui, (size_t)count * sizeof *copy);
    memcpy(copy, order, (size_t)count * sizeof *copy);
    if (!enabled || cui_is_disabled(ui)) {
        cui__reorder_state *state = cui_state(ui, cui_make_id(ui, id), sizeof *state);
        state->from = -1;
        state->pending = false;
    }
    cui_reorder_begin(ui, id, copy, count);
    memcpy(proposed, copy, (size_t)count * sizeof *copy);
    return node;
}

int fpas_cui_reorder_item_begin(cui_ctx *ui, int position, int enabled) {
    const int node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    cui_reorder_item_begin(ui, position);
    cui_pop_disabled(ui);
    return node;
}

void fpas_cui_reorder_item_end(cui_ctx *ui, int node) {
    cui_reorder_item_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

int fpas_cui_reorder_end(cui_ctx *ui, int node) {
    const bool changed = cui_reorder_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
    return changed;
}
