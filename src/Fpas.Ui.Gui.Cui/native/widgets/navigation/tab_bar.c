// [Documentation](../../../../../docs/ui/gui/motion/tab-bar.md)
#include "../../fpas_cui.h"

static cui_id fpas_tab_bar_part(cui_id key, int index) {
    return cui__hash(&index, sizeof index, key ^ 0x7ABu);
}

int fpas_cui_tab_bar(cui_ctx *ui, const char *id, const char *const *icons, const char *const *labels,
    const int *badges, int count, int *selected, int width_kind, float width, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_tab_bar_item *items = cui__frame_alloc(ui, (size_t)count * sizeof *items);
    for (int i = 0; i < count; ++i)
        items[i] = (cui_tab_bar_item){ icons[i][0] ? icons[i] : NULL, labels[i], badges[i] };
    const cui_tab_bar_opts options = { .width = fpas_extent(width_kind, width),
        .labels_on_selected_only = (flags & 1) != 0 };
    const bool enabled = (flags & 2) != 0;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus >= 0)
        ui->focus_next_request = fpas_tab_bar_part(key, focus);

    cui_push_disabled(ui, !enabled);
    const bool changed = cui_tab_bar_ex(ui, id, items, count, selected, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) parts[i] = fpas_find(ui, fpas_tab_bar_part(key, i));
    fpas_motion_node(ui, node, key);
    return changed;
}
