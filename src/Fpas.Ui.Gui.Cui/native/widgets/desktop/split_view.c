// [Documentation](../../../../../docs/ui/gui/app-chrome/split-view.md)
#include "../../fpas_cui.h"
#include "chrome_input.h"

/** Opens a native split and its first pane if the parent stack can hold both. */
float fpas_cui_split_view_begin(cui_ctx *ui, const char *id, float ratio, int vertical,
    float first, float second, int width_kind, float width, int height_kind, float height, int enabled) {
    if (ui->parent_depth > CUI_PARENT_STACK - 2) return -1;
    cui_split_opts opts = { .vertical = vertical != 0, .min_first = first, .min_second = second,
        .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height) };
    float *value = cui__frame_alloc(ui, sizeof *value);
    *value = ratio;
    const bool allowed = enabled && !cui_is_disabled(ui);
    fpas_chrome_input input;
    if (!allowed) fpas_chrome_pause(ui, &input);
    cui_push_disabled(ui, !allowed);
    cui_split_begin(ui, id, value, &opts);
    cui_pop_disabled(ui);
    if (!allowed) fpas_chrome_resume(ui, &input);
    return *value;
}

/** Advances only when the first pane is the active layout. */
int fpas_cui_split_view_next(cui_ctx *ui, int enabled) {
    struct cui_split_build *split = ui->building.split;
    const int node = cui__current(ui);
    if (!split || node < 0 || node != ui->nodes[split->container].first) return 0;
    cui_split_next(ui);
    if (!enabled) {
        const int handle = fpas_find(ui, split->id ^ 0x77u);
        if (handle >= 0) ui->nodes[handle].flags &= ~(uint32_t)CUI_NF_INTERACTIVE;
    }
    return 1;
}

/** Ends only an active second pane and restores the split's identity. */
int fpas_cui_split_view_end(cui_ctx *ui) {
    struct cui_split_build *split = ui->building.split;
    const int node = cui__current(ui);
    if (!split || node < 0 || node != ui->nodes[split->container].last ||
        node == ui->nodes[split->container].first) return 0;
    cui_split_end(ui);
    fpas_own(ui, split->id, split->id ^ 0x77u, 0, 0);
    ui->last_node = split->container;
    ui->last_id = split->id;
    return 1;
}
