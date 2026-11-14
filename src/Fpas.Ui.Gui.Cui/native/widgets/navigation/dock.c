#include "../../fpas_cui.h"

int fpas_cui_dock(cui_ctx *ui, const char *id, const char *const *icons, const char *const *labels,
    const int *palettes, const uint32_t *colors, const int *badges, const int *kinds, int count,
    float size, float magnify, int enabled, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_dock_opts options = { .size = size, .magnify = magnify };
    int chosen = -1;
    cui_push_disabled(ui, !enabled);
    cui_dock_begin(ui, id, &options);
    for (int i = 0, slot = 0; i < count; ++i) {
        parts[i] = -1;
        if (!kinds[i]) { cui_dock_separator(ui); continue; }
        const cui_id part = cui__hash(&slot, sizeof slot, key ^ 0xD0Cu);
        ++slot;
        if (focus == i && enabled && !cui_is_disabled(ui)) ui->focus_next_request = part;
        const cui_dock_item_opts entry = { .color = fpas_chart_color(palettes[i], colors[i]),
            .running = kinds[i] == 2, .badge = badges[i] };
        if (cui_dock_item_ex(ui, cui__frame_strdup(ui, icons[i], -1),
            cui__frame_strdup(ui, labels[i], -1), &entry)) chosen = i;
        parts[i] = fpas_find(ui, part);
    }
    cui_dock_end(ui);
    cui_pop_disabled(ui);
    fpas_own(ui, key, key ^ 0xD0C5u, 0, 0);
    fpas_motion_node(ui, node, key);
    return chosen;
}
