#include "../fpas_cui.h"

int fpas_cui_donut_chart(cui_ctx *ui, const char *id, const float *values, int count, const fpas_chart_options *options) {
    const int node = ui->node_count;
    const cui_chart_opts o = fpas_chart_options_of(ui, options);
    cui_donut_chart_ex(ui, id, values, options->label_count > 0 ? options->labels : NULL, count, &o);
    ui->last_node = node;
    return node;
}
