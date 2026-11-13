#include "../fpas_cui.h"

int fpas_cui_spotlight_begin(cui_ctx *ui, const char *id, int width_kind, float width,
    int height_kind, float height, float padding, float gap, int palette, uint32_t rgba, int beam) {
    const int node = ui->node_count;
    const cui_spotlight_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .pad = padding, .gap = gap,
        .color = fpas_chart_color(palette, rgba), .beam = beam != 0 };
    cui_spotlight_begin(ui, id, &options);
    return node;
}

void fpas_cui_spotlight_end(cui_ctx *ui, int node) {
    cui_spotlight_end(ui);
    fpas_motion_node(ui, node, ui->nodes[node].id);
}
