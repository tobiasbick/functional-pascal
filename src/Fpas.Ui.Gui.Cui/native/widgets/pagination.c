#include "../fpas_cui.h"

static cui_id fpas_pager_focus(cui_ctx *ui, const char *id, int focus) {
    cui_push_id(ui, id);
    cui_id key;
    if (focus >= 0) key = cui__hash(&focus, sizeof focus, cui_make_id(ui, "#page"));
    else if (focus == -1) key = cui__hash_str(CUI_ICON_CHEVRON_LEFT, cui_make_id(ui, "Previous page"));
    else key = cui__hash_str(CUI_ICON_CHEVRON_RIGHT, cui_make_id(ui, "Next page"));
    cui_pop_id(ui);
    return key;
}

int fpas_cui_pagination(cui_ctx *ui, const char *id, int page, int count,
    int enabled, int focus, int *parts, int *used) {
    const cui_id group = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_id key = allowed && focus != -3 ? fpas_pager_focus(ui, id, focus) : 0;
    if (key) ui->focus_next_request = key;
    int next = page;
    const int row = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_pagination(ui, id, &next, count);
    cui_pop_disabled(ui);
    if (key && ui->focus_next_request == key) ui->focus_next_request = 0;

    const int first = ui->nodes[row].first, last = ui->nodes[row].last;
    *used = 0;
    for (int child = first; child >= 0 && *used < 9; child = ui->nodes[child].next) {
        if (!ui->nodes[child].id) continue;   /* ellipses have no identity */
        const int index = child == first ? -1 : child == last ? -2
            : atoi(ui->nodes[fpas_label_node(ui, child)].text) - 1;
        parts[*used * 2] = index;
        parts[*used * 2 + 1] = child;
        ++*used;
    }
    cui_push_id(ui, id);
    const cui_id measurement = cui_make_id(ui, "#pager-width");
    cui_pop_id(ui);
    fpas_own(ui, group, measurement, 0, 0);
    ui->nodes[row].id = group;
    if (!enabled) ui->nodes[row].opacity = CUI__DISABLED;
    ui->last_node = row;
    ui->last_id = group;
    return allowed && changed ? next : -1;
}
