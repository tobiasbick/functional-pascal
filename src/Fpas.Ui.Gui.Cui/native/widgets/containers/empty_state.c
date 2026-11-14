#include "../../fpas_cui.h"

int fpas_cui_empty_state(cui_ctx *ui, const char *id, const char *title, const char *icon,
    const char *description, const char *action, int enabled, int focus, int *action_node) {
    const int node = ui->node_count;
    cui_push_id(ui, id);
    const cui_empty_opts options = {
        .icon = icon, .description = description, .action = action ? "x###action" : NULL
    };
    if (action && enabled && focus && !cui_is_disabled(ui)) ui->focus_next_request = cui_make_id(ui, "###action");
    cui_push_disabled(ui, !enabled);
    const bool clicked = cui_empty_state_ex(ui, title, &options);
    cui_pop_disabled(ui);
    cui_pop_id(ui);
    *action_node = action ? ui->last_node : -1;
    if (action) fpas_literal(ui, *action_node, action, CUI_C(ui, FOREGROUND));
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
    return clicked;
}
