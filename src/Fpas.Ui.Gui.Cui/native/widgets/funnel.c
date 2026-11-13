#include "../fpas_cui.h"

int fpas_cui_funnel(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, int palette, uint32_t rgba, const char *format, int enabled, int *node) {
    const cui_funnel_opts o = { .width = fpas_extent(width_kind, width),
        .color = fpas_chart_color(palette, rgba), .format = fpas_viz_text(ui, format) };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_funnel_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
