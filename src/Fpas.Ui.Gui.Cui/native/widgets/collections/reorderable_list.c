// [Documentation](../../../../../docs/ui/gui/data-display/reorderable-list.md)
#include "../../fpas_cui.h"

/** Opens the native list with an order copy for the current frame. */
int fpas_cui_reorder_begin(cui_ctx *ui, const char *id, const int *order, int count, int enabled, int *proposed) {
    const int node = ui->node_count;
    const size_t bytes = (size_t)count * sizeof *order;
    int *copy = count ? cui__frame_alloc(ui, bytes) : NULL;
    if (count) memcpy(copy, order, bytes);
    if (!enabled || cui_is_disabled(ui)) {
        cui__reorder_state *state = cui_state(ui, cui_make_id(ui, id), sizeof *state);
        state->from = -1;
        state->pending = false;
    }
    cui_reorder_begin(ui, id, copy, count);
    if (count) memcpy(proposed, copy, bytes);
    return node;
}

/** Opens a native row and attaches the row's spring to its tracked identity. */
int fpas_cui_reorder_item_begin(cui_ctx *ui, int position, int enabled) {
    const int node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    cui_reorder_item_begin(ui, position);
    cui_pop_disabled(ui);
    const cui_id key = ui->nodes[node].id;
    if (key) fpas_own(ui, key, key ^ 0x1u, 0, 0);
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
