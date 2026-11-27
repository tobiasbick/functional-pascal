// [Documentation](../../../../../docs/ui/gui/more-charts/heatmap.md)
#include "../../fpas_cui.h"

int fpas_cui_heatmap(cui_ctx *ui, const char *id, const float *values, int rows, int columns,
    const char *const *row_labels, const char *const *column_labels, float cell, float minimum, float maximum,
    int palette, uint32_t rgba, const char *format, int width_kind, float width, int enabled, int *node) {
    const cui_heatmap_opts o = { .row_labels = fpas_viz_labels(ui, row_labels, rows),
        .col_labels = fpas_viz_labels(ui, column_labels, columns), .cell = cell, .min = minimum, .max = maximum,
        .color = fpas_chart_color(palette, rgba), .format = fpas_viz_text(ui, format), .width = fpas_extent(width_kind, width) };
    const cui_id hot = ui->hot, key = cui_make_id(ui, id);
    // CUI-087 (docs/future/cui-upstream-bugs.md#cui-087): gaps belong to no cell.
    if (hot == key) {
        const cui__hm_geo g = cui__hm_layout(ui, cui__prev_rect(ui, key), rows, columns, &o);
        if (fpas_viz_cell_gap(ui, g.gx, g.gy, g.cell, g.gap)) ui->hot = 0;
    }
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_heatmap_ex(ui, id, values, rows, columns, &o);
    cui_pop_disabled(ui);
    ui->hot = hot;
    ui->last_node = *node;
    return clicked;
}
