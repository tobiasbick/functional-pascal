#include "../fpas_cui.h"

int fpas_cui_source_list(cui_ctx *ui, const char *id, const cui_source *sources, int count, int flags, int focus, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < count) ui->focus_next_request = fpas_well_part(key ^ 0x50Cu, focus);
    cui_push_disabled(ui, !enabled);
    const int picked = cui_source_list(ui, id, sources, count);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) parts[i] = fpas_find(ui, fpas_well_part(key ^ 0x50Cu, i));
    fpas_creative_node(ui, root, key);
    return enabled ? picked : -1;
}
