#include "../fpas_cui.h"

void fpas_cui_metric(cui_ctx *ui, const char *label, const char *value, float delta,
    const char *delta_label, const float *spark, int count, const char *icon,
    int width_kind, float width, int lower_is_better) {
    const int node = ui->node_count;
    const cui_metric_opts options = {
        .delta = delta, .delta_label = delta_label, .spark = spark, .spark_count = count,
        .icon = icon, .width = fpas_extent(width_kind, width), .lower_is_better = lower_is_better != 0
    };
    cui_metric_ex(ui, label, value, &options);
    ui->last_node = node;
}
