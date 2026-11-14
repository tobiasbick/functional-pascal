#include "../../fpas_cui.h"

int fpas_cui_waffle(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    float cell, int columns, int rows, const char *format, int enabled, int *node) {
    const cui_waffle_opts o = { .cell = cell, .columns = columns, .rows = rows, .format = fpas_viz_text(ui, format) };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_waffle_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
