#include "../fpas_cui.h"

void fpas_cui_gradient_text(cui_ctx *ui, const char *id, const char *text, const fpas_motion_text *o,
    int to_palette, uint32_t to_rgba, int animate) {
    const int node = ui->node_count;
    const cui_gradient_text_opts options = { .font = o->font, .size = o->size,
        .width = fpas_extent(o->width_kind, o->width), .from = fpas_chart_color(o->palette, o->rgba),
        .to = fpas_chart_color(to_palette, to_rgba), .animate = animate != 0 };
    cui_gradient_text_ex(ui, text, &options);
    fpas_motion_node(ui, node, cui_make_id(ui, id));
}
