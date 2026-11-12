#include "../fpas_cui.h"

int fpas_cui_reactions(cui_ctx *ui, const char *id, const char *const *icons,
    int *counts, int *active, int count, int enabled, int focus, int *nodes) {
    if (count < 1) return -1;
    const cui_id group = cui_make_id(ui, id);
    cui_reaction *items = cui__frame_alloc(ui, (size_t)count * sizeof *items);
    for (int i = 0; i < count; ++i)
        items[i] = (cui_reaction){ icons[i], counts[i], active[i] != 0 };

    if (enabled && focus) {
        const int index = focus - 1;
        ui->focus_next_request = cui__hash(&index, sizeof index, group);
    }
    cui_push_disabled(ui, !enabled);
    const int changed = cui_reactions(ui, id, items, count);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i)
        nodes[i + 1] = fpas_find(ui, cui__hash(&i, sizeof i, group));
    nodes[0] = fpas_find(ui, group);
    ui->last_node = nodes[0];
    if (!enabled || changed < 0) return -1;
    counts[changed] = items[changed].count;
    active[changed] = items[changed].active;
    return changed;
}
