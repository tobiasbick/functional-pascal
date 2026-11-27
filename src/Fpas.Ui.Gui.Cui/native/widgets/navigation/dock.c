// [Documentation](../../../../../docs/ui/gui/motion/dock.md)
#include "../../fpas_cui.h"

static cui_id fpas_dock_part(cui_id dock, int slot) {
    return cui__hash(&slot, sizeof slot, dock ^ 0xD0Cu);
}

static void fpas_dock_forget(cui_ctx *ui, cui_id dock) {
    const cui__dock_count *count = cui__state_lazy(ui, dock ^ 0xD0C5u, sizeof *count, false);
    if (count) for (int slot = 0; slot < count->items; ++slot) fpas_forget_state(ui, fpas_dock_part(dock, slot));
}

int fpas_cui_dock(cui_ctx *ui, const char *id, const char *const *icons, const char *const *labels,
    const int *palettes, const uint32_t *colors, const int *badges, const int *kinds, int count,
    float size, float magnify, int enabled, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_dock_opts options = { .size = size, .magnify = magnify };
    const cui_id count_key = key ^ 0xD0C5u;
    cui__dock_count *previous = cui_state(ui, count_key, sizeof *previous);
    int item_count = 0;
    for (int i = 0; i < count; ++i) if (kinds[i]) ++item_count;
    for (int slot = item_count; slot < previous->items; ++slot) fpas_forget_state(ui, fpas_dock_part(key, slot));
    // CUI-105 (docs/future/cui-upstream-bugs.md#cui-105): use the known count on the first frame.
    *previous = (cui__dock_count){ .items = item_count, .seps = count - item_count };
    int chosen = -1;
    cui_push_disabled(ui, !enabled);
    cui_dock_begin(ui, id, &options);
    struct cui_dock_build *dock = ui->building.dock;
    // CUI-106 (docs/future/cui-upstream-bugs.md#cui-106): a fixed icon size has no magnification to update.
    if (dock->mag == 1) dock->hot = false;
    for (int i = 0, slot = 0; i < count; ++i) {
        parts[i] = -1;
        if (!kinds[i]) { cui_dock_separator(ui); continue; }
        const cui_id part = fpas_dock_part(key, slot++);
        if (focus == i && enabled && !cui_is_disabled(ui)) ui->focus_next_request = part;
        const cui_dock_item_opts entry = { .color = fpas_chart_color(palettes[i], colors[i]),
            .running = kinds[i] == 2, .badge = badges[i] };
        if (cui_dock_item_ex(ui, cui__frame_strdup(ui, icons[i], -1),
            cui__frame_strdup(ui, labels[i], -1), &entry)) chosen = i;
        parts[i] = fpas_find(ui, part);
    }
    cui_dock_end(ui);
    cui_pop_disabled(ui);
    fpas_own(ui, key, count_key, 0, 0);
    fpas_on_forget(ui, key, fpas_dock_forget);
    fpas_motion_node(ui, node, key);
    return chosen;
}
