#include "../../fpas_cui.h"

int fpas_cui_line_chart(cui_ctx *ui, const char *id, const fpas_chart_series *series, int count, const fpas_chart_options *options) {
    const int node = ui->node_count;
    const cui_chart_opts o = fpas_chart_options_of(ui, options);
    cui_line_chart_ex(ui, id, fpas_chart_series_of(ui, series, count), count, &o);
    ui->last_node = node;
    return node;
}
