#include "../fpas_cui.h"

int fpas_cui_selection_bar(cui_ctx *ui, const char *id, bool *selected, int count,
    const cui_selection_action *actions, int action_count, int enabled, int no_escape) {
    const int anchor = ui->last_node;
    const cui_id previous = ui->last_id;
    const cui_id key = cui_make_id(ui, id);
    /* Generic FPAS containers can be anonymous. Assign an ID only when the anchor needs one. */
    if (anchor >= 0 && !ui->nodes[anchor].id) ui->nodes[anchor].id = key ^ 0xBA4A11u;
    const cui_id anchor_id = anchor >= 0 ? ui->nodes[anchor].id : 0;
    const cui_selection_bar_opts options = { .selected = selected, .item_count = count,
        .anchor = anchor_id, .no_escape = no_escape || !enabled };
    cui_push_disabled(ui, !enabled);
    const int event = cui_selection_bar_ex(ui, id, 0, actions, action_count, &options);
    cui_pop_disabled(ui);
    ui->last_node = anchor;
    ui->last_id = previous;
    return enabled ? event : CUI_SELECTION_NONE;
}
