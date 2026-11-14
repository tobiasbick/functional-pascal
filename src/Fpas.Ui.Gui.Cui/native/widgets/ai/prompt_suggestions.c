#include "../../fpas_cui.h"

int fpas_cui_prompt_suggestions(cui_ctx *ui, const char *id, const cui_prompt_suggestion *items,
    int count, int columns, int flags, int focus, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < count) ui->focus_next_request = fpas_well_part(key ^ 0x9A0u, focus);
    const cui_prompt_suggestions_opts opts = { .columns = columns };
    cui_push_disabled(ui, !enabled);
    const int picked = cui_prompt_suggestions_ex(ui, id, items, count, &opts);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) parts[i] = fpas_find(ui, fpas_well_part(key ^ 0x9A0u, i));
    fpas_creative_node(ui, root, key);
    return enabled ? picked : -1;
}
