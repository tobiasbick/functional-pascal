#include "../fpas_cui.h"

void fpas_cui_sparkline(cui_ctx *ui, const char *id, const float *values, int count,
    int width_kind, float width, float height, int palette, uint32_t rgba) {
    const int first = ui->node_count;
    cui_sparkline(ui, values, count, fpas_extent(width_kind, width), height,
        fpas_chart_color(palette, rgba));
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
