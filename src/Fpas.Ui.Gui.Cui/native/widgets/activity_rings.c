#include "../fpas_cui.h"

int fpas_cui_activity_rings(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    float size, float thickness, const char *center, const char *caption, int legend, int enabled, int *node) {
    const cui_rings_opts o = { .size = size, .thickness = thickness, .center = fpas_viz_text(ui, center),
        .caption = fpas_viz_text(ui, caption), .legend = legend != 0 };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_activity_rings_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
