#include "../fpas_cui.h"

int fpas_cui_radar_chart(cui_ctx *ui, const char *id, const fpas_chart_series *series, int count,
    const char *const *axes, int axis_count, float size, float maximum, const char *format, int legend,
    int enabled, int *node) {
    cui_series *copy = fpas_chart_series_of(ui, series, count);
    for (int i = 0; i < count; ++i) copy[i].name = fpas_viz_text(ui, copy[i].name);
    const cui_radar_opts o = { .size = size, .max = maximum, .format = fpas_viz_text(ui, format), .no_legend = !legend };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const int clicked = cui_radar_chart_ex(ui, id, copy, count, fpas_viz_labels(ui, axes, axis_count), axis_count, &o);
    cui_pop_disabled(ui); ui->last_node = *node;
    return clicked;
}
