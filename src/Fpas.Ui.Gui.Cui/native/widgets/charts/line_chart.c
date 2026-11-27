// [Documentation](../../../../../docs/ui/gui/charts/line-chart.md)
#include "../../fpas_cui.h"

/* CUI-028 (docs/future/cui-upstream-bugs.md#cui-028): native painting leaves
   the crosshair unset for one sample. Use the plot origin from native layout. */
static void fpas_paint_single_line(cui_ctx *ui, cui_node *node, void *data) {
    cui__chart_paint *chart = data;
    const float offset = chart->st->plot_x;
    chart->hx = node->rect.x + offset;
    cui__paint_xy_chart(ui, node, data);
    if (chart->hover >= 0 && chart->st->plot_x != offset) ui->animating = true;
}

int fpas_cui_line_chart(cui_ctx *ui, const char *id, const fpas_chart_series *series, int count, const fpas_chart_options *options) {
    const int node = ui->node_count;
    const cui_chart_opts o = fpas_chart_options_of(ui, options);
    cui_line_chart_ex(ui, id, fpas_chart_series_of(ui, series, count), count, &o);
    const int plot = ui->nodes[node].last;
    const cui_id key = ui->nodes[node].id, plot_key = ui->nodes[plot].id;
    fpas_own(ui, key, plot_key, 0, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->hover = plot_key;

    int samples = 0;
    for (int i = 0; i < count; ++i) samples = CUI_MAX(samples, series[i].count);
    if (samples == 1) ui->nodes[plot].paint = fpas_paint_single_line;

    ui->last_node = node;
    return node;
}
