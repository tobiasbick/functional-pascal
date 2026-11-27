// [Documentation](../../../../../docs/ui/gui/motion/loader.md)
#include "../../fpas_cui.h"

/** Retains one native phase without consuming another widget's redraw request. */
static void fpas_freeze_loader(cui_ctx *ui, int node, bool animating) {
    // CUI-096 (docs/future/cui-upstream-bugs.md#cui-096): honor reduced motion.
    cui__ld_paint *paint = ui->nodes[node].paint_data;
    paint->t = 0;
    ui->animating = animating;
}

/** Draws a native loader and honors the theme's motion preference. */
void fpas_cui_loader(cui_ctx *ui, const char *id, int style, float size, int palette, uint32_t rgba) {
    const int node = ui->node_count;
    const cui_loader_opts options = { .style = style, .size = size, .color = fpas_chart_color(palette, rgba) };
    const bool animating = ui->animating;
    cui_loader_ex(ui, &options);
    if (cui__still(ui)) fpas_freeze_loader(ui, node, animating);
    fpas_motion_node(ui, node, cui_make_id(ui, id));
}
