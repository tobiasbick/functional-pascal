// [Documentation](../../../../../docs/ui/gui/data-grid/tree-table.md#context-menu)
#include "../../fpas_cui.h"

int fpas_cui_tree_table_menu_begin(cui_ctx *ui, const char *id, int enabled, int *node,
    unsigned int *popup, int *table) {
    *node = -1;
    /* The open popup pushes its own ID scope and layout node, so record both first. */
    *popup = cui_make_id(ui, id) ^ CUI__TT_MENU;
    *table = ui->last_node;
    return enabled && cui_tree_table_menu_begin(ui, id, node);
}

int fpas_cui_tree_table_menu_end(cui_ctx *ui, unsigned int popup, int table, int visible,
    const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const int *flags, int count, int *nodes,
    int *open, int *highlighted, unsigned int *owner) {
    const cui_id pid = popup;
    const int chosen = fpas_menu_rows(ui, pid, visible != 0, keys, labels, icons, shortcuts,
        flags, count, nodes, highlighted);
    if (visible) cui_tree_table_menu_end(ui);
    const cui_popup *state = cui__popup_get(ui, pid, false);
    *open = visible && state && state->open;
    if (!*open) {
        *highlighted = -1;
        for (int i = 1; i <= count; ++i) nodes[i] = -1;
    }
    nodes[0] = table;
    ui->last_node = table;
    ui->last_id = table >= 0 ? ui->nodes[table].id : 0;
    *owner = pid;
    fpas_own(ui, pid, pid ^ 0x4E4156u, 0, pid);
    return visible ? chosen : -1;
}
