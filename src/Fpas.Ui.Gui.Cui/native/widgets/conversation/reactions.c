// [Documentation](../../../../../docs/ui/gui/actions/reactions.md)
#include "../../fpas_cui.h"

/** Draws the native group and copies only the selected reaction back. */
int fpas_cui_reactions(cui_ctx *ui, const char *id, const char *const *icons,
    int *counts, int *active, int count, int enabled, int focus, int *nodes) {
    const cui_id group = cui_make_id(ui, id);
    cui_reaction *items = count ? cui__frame_alloc(ui, (size_t)count * sizeof *items) : NULL;
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
