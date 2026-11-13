#include "../fpas_cui.h"

void fpas_cui_counter(cui_ctx *ui, const char *id, double value, const char *format,
    float size, int palette, uint32_t rgba, int group) {
    const int first = ui->node_count;
    const cui_counter_opts options = { .format = format, .size = size,
        .color = fpas_chart_color(palette, rgba), .group_thousands = group != 0 };
    cui_counter_ex(ui, id, value, &options);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
