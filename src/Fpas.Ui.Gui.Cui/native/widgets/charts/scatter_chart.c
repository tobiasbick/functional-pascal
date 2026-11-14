#include "../../fpas_cui.h"

int fpas_cui_scatter_chart(cui_ctx *ui, const char *id, const fpas_scatter_series *series, int count,
    int width_kind, float width, float height, const char *x_label, const char *y_label, const char *x_format,
    const char *y_format, int legend, int enabled, int *node) {
    cui_scatter_series *data = cui__frame_alloc(ui, sizeof *data * (size_t)(count > 0 ? count : 1));
    for (int i = 0; i < count; ++i)
        data[i] = (cui_scatter_series){ .name = fpas_viz_text(ui, series[i].name), .points = series[i].points,
            .sizes = series[i].sizes, .count = series[i].count, .color = fpas_chart_color(series[i].palette, series[i].rgba) };
    const cui_scatter_opts o = { .width = fpas_extent(width_kind, width), .height = height,
        .x_label = fpas_viz_text(ui, x_label), .y_label = fpas_viz_text(ui, y_label),
        .x_format = fpas_viz_text(ui, x_format), .y_format = fpas_viz_text(ui, y_format), .no_legend = !legend };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_scatter_chart_ex(ui, id, data, count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
