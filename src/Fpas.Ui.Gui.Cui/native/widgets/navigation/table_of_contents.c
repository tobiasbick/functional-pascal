#include "../../fpas_cui.h"

int fpas_cui_table_of_contents(cui_ctx *ui, const char *scroll_id,
    const char *const *titles, const char *const *anchors, const int *levels,
    int count, int enabled, int focus, int *node, int *rows) {
    cui_toc_item items[64];
    if (count < 1 || count > 64) return -1;
    for (int index = 0; index < count; ++index)
        items[index] = (cui_toc_item){ titles[index], anchors[index], levels[index] };

    if (enabled && !cui_is_disabled(ui) && focus > 0 && focus <= count) {
        const cui_id key = cui_make_id(ui, scroll_id) ^ 0x70CCu;
        const int index = focus - 1;
        ui->focus_next_request = cui__hash(&index, sizeof index, key);
    }

    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int active = cui_toc(ui, scroll_id, items, count);
    cui_pop_disabled(ui);
    if (rows) {
        const cui_id key = ui->nodes[*node].id;
        for (int index = 0; index < count; ++index)
            rows[index] = fpas_find(ui, cui__hash(&index, sizeof index, key));
    }
    ui->last_node = *node;
    return active;
}
