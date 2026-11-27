// [Documentation](../../../../../docs/ui/gui/motion/spotlight-card.md)
#include "../../fpas_cui.h"

int fpas_cui_spotlight_begin(cui_ctx *ui, const char *id, int width_kind, float width,
    int height_kind, float height, float padding, float gap, int palette, uint32_t rgba, int flags) {
    const int node = ui->node_count;
    const cui_spotlight_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .pad = padding, .gap = gap,
        .color = fpas_chart_color(palette, rgba), .beam = (flags & 1) != 0 };
    const bool mouse_inside = ui->mouse_inside;
    const bool animating = ui->animating;
    /* [CUI-102](../../../../../docs/future/cui-upstream-bugs.md#cui-102): restrict hover to input-enabled cards. */
    ui->mouse_inside = mouse_inside && (flags & 2) && !cui_is_disabled(ui);
    cui_spotlight_begin(ui, id, &options);
    ui->mouse_inside = mouse_inside;
    /* [CUI-103](../../../../../docs/future/cui-upstream-bugs.md#cui-103): a settled glow needs no animation frames. */
    const cui__spot_paint *paint = ui->nodes[node].paint_data;
    if (!paint->beam && paint->hover >= 1.0f) ui->animating = animating;
    return node;
}

void fpas_cui_spotlight_end(cui_ctx *ui, int node) {
    cui_spotlight_end(ui);
    fpas_motion_node(ui, node, ui->nodes[node].id);
}
