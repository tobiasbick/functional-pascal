#include "../../fpas_cui.h"

int fpas_cui_treemap(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, float height, const char *format, int selectable, int *selected, int enabled, int *node) {
    const cui_treemap_opts o = { .width = fpas_extent(width_kind, width), .height = height,
        .format = fpas_viz_text(ui, format), .selected = selectable ? selected : NULL };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_treemap_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
