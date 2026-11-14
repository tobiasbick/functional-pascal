#include "../../fpas_cui.h"

int fpas_cui_tree_begin(cui_ctx *ui, const char *id, const char *label, const char *icon,
    int leaf, int selected, int enabled, int focus, int *open, int *node) {
    const cui_id key = cui_make_id(ui, id) ^ 0x7EEE0007u;
    if (focus && enabled && !cui_is_disabled(ui)) ui->focus_next_request = key;
    const cui_tree_opts options = { .icon = icon, .leaf = leaf != 0, .selected = selected != 0 };
    bool expanded = *open != 0;
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    cui_tree_begin(ui, id, &expanded, &options);
    cui_pop_disabled(ui);
    fpas_text(ui, fpas_label_node(ui, *node), label);
    ui->last_node = *node;
    *open = expanded && !leaf;
    return cui_tree_clicked(ui);
}

void fpas_cui_tree_end(cui_ctx *ui, int node) {
    cui_tree_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}
