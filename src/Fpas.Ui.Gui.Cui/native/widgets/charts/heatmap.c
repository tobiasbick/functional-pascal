#include "../../fpas_cui.h"

int fpas_cui_heatmap(cui_ctx *ui, const char *id, const float *values, int rows, int columns,
    const char *const *row_labels, const char *const *column_labels, float cell, float minimum, float maximum,
    int palette, uint32_t rgba, const char *format, int width_kind, float width, int enabled, int *node) {
    const cui_heatmap_opts o = { .row_labels = fpas_viz_labels(ui, row_labels, rows),
        .col_labels = fpas_viz_labels(ui, column_labels, columns), .cell = cell, .min = minimum, .max = maximum,
        .color = fpas_chart_color(palette, rgba), .format = fpas_viz_text(ui, format), .width = fpas_extent(width_kind, width) };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_heatmap_ex(ui, id, values, rows, columns, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
