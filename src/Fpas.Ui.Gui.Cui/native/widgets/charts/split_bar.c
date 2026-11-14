#include "../../fpas_cui.h"

int fpas_cui_split_bar(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, float height, const char *format, int legend, int enabled, int *node) {
    const cui_split_bar_opts o = { .width = fpas_extent(width_kind, width), .height = height,
        .format = fpas_viz_text(ui, format), .no_legend = !legend };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_split_bar_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
