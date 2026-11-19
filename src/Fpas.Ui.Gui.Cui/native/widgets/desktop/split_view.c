// [Documentation](../../../../../docs/ui/gui/app-chrome/split-view.md)
#include "../../fpas_cui.h"
#include "chrome_input.h"

float fpas_cui_split_view_begin(cui_ctx *ui, const char *id, float ratio, int vertical,
    float first, float second, int width_kind, float width, int height_kind, float height, int enabled) {
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

void fpas_cui_split_view_next(cui_ctx *ui, int enabled) {
    struct cui_split_build *split = ui->building.split;
    cui_split_next(ui);
    if (!enabled && split) {
        const int handle = fpas_find(ui, split->id ^ 0x77u);
        if (handle >= 0) ui->nodes[handle].flags &= ~(uint32_t)CUI_NF_INTERACTIVE;
    }
}

void fpas_cui_split_view_end(cui_ctx *ui) {
    struct cui_split_build *split = ui->building.split;
    if (!split) return;
    cui_split_end(ui);
    fpas_own(ui, split->id, split->id ^ 0x77u, 0, 0);
    ui->last_node = split->container;
    ui->last_id = split->id;
}
