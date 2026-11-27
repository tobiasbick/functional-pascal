// [Documentation](../../../../../docs/ui/gui/motion/tab-bar.md)
#include "../../fpas_cui.h"

static cui_id fpas_tab_bar_part(cui_id key, int index) {
    return cui__hash(&index, sizeof index, key ^ 0x7ABu);
}

static cui_id fpas_tab_bar_count(cui_id key) { return key ^ 0x7AB5u; }

static void fpas_tab_bar_forget(cui_ctx *ui, cui_id key) {
    const int *count = cui__state_lazy(ui, fpas_tab_bar_count(key), sizeof *count, false);
    if (count) for (int i = 0; i < *count; ++i) fpas_forget_state(ui, fpas_tab_bar_part(key, i));
}

typedef struct { cui__tb_paint *native; int selected; cui_rect previous; } fpas_tab_bar_paint;

static void fpas_paint_tab_bar(cui_ctx *ui, cui_node *node, void *data) {
    const fpas_tab_bar_paint *paint = data;
    const cui_rect current = ui->nodes[paint->selected].rect;
    // CUI-104 (docs/future/cui-upstream-bugs.md#cui-104): reconcile the pill after layout changes.
    if (current.x != paint->previous.x || current.w != paint->previous.w) cui_request_redraw(ui);
    cui__paint_tab_bar(ui, node, paint->native);
}

int fpas_cui_tab_bar(cui_ctx *ui, const char *id, const char *const *icons, const char *const *labels,
    const int *badges, int count, int *selected, int width_kind, float width, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id count_key = fpas_tab_bar_count(key);
    int *previous_count = cui_state(ui, count_key, sizeof *previous_count);
    for (int i = count; i < *previous_count; ++i) fpas_forget_state(ui, fpas_tab_bar_part(key, i));
    *previous_count = count;
    cui_tab_bar_item *items = cui__frame_alloc(ui, (size_t)count * sizeof *items);
    for (int i = 0; i < count; ++i)
        items[i] = (cui_tab_bar_item){ icons[i], labels[i], badges[i] };
    const cui_tab_bar_opts options = { .width = fpas_extent(width_kind, width),
        .labels_on_selected_only = (flags & 1) != 0 };
    const bool enabled = (flags & 2) != 0;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus >= 0 && focus < count)
        ui->focus_next_request = fpas_tab_bar_part(key, focus);

    cui_push_disabled(ui, !enabled);
    const bool changed = cui_tab_bar_ex(ui, id, items, count, selected, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) parts[i] = fpas_find(ui, fpas_tab_bar_part(key, i));
    int current = selected ? *selected : 0;
    if (current >= 0 && current < count && parts[current] >= 0) {
        fpas_tab_bar_paint *paint = cui__frame_alloc(ui, sizeof *paint);
        *paint = (fpas_tab_bar_paint){ ui->nodes[node].paint_data, parts[current],
            cui__prev_rect(ui, fpas_tab_bar_part(key, current)) };
        ui->nodes[node].paint = fpas_paint_tab_bar;
        ui->nodes[node].paint_data = paint;
    }
    fpas_own(ui, key, count_key, 0, 0);
    fpas_on_forget(ui, key, fpas_tab_bar_forget);
    fpas_motion_node(ui, node, key);
    return changed;
}
