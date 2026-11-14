#include "../fpas_cui.h"
#include "chrome_input.h"

float fpas_cui_split_view_begin(cui_ctx *ui, const char *id, float ratio, int vertical,
    float first, float second, int width_kind, float width, int height_kind, float height, int enabled) {
    cui_split_opts opts = { .vertical = vertical != 0, .min_first = first, .min_second = second,
        .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height) };
    /* CUI-040 (docs/future/cui-upstream-bugs.md#cui-040): proportional
       minimums keep the divider inside its parent when both minima cannot fit. */
    const cui_rect bounds = cui__prev_rect(ui, cui_make_id(ui, id));
    const float axis = vertical ? bounds.h : bounds.w;
    const cui_size extent = vertical ? opts.height : opts.width;
    const float planned = extent.kind == CUI_SIZE_FIXED ? extent.value : axis;
    const float available = planned > 0 ? CUI_MAX(0.002f, planned - 1.0f) : 0;
    const double min_first = first > 0 ? first : 120;
    const double min_second = second > 0 ? second : 120;
    const double total = min_first + min_second;
    if (available > 0 && total > available) {
        opts.min_first = CUI_MAX(0.001f, (float)(available * min_first / total));
        opts.min_second = CUI_MAX(0.001f, (float)(available * min_second / total));
    }

    float *value = cui__frame_alloc(ui, sizeof *value);
    *value = ratio;
    const bool allowed = enabled && !cui_is_disabled(ui) && axis > 1.0f && planned > 1.0f;
    fpas_chrome_input input;
    if (!allowed) fpas_chrome_pause(ui, &input);
    cui_push_disabled(ui, !allowed);
    cui_split_begin(ui, id, value, &opts);
    cui_pop_disabled(ui);
    if (!allowed) fpas_chrome_resume(ui, &input);
    return cui__clampf(*value, 0, 1);
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
