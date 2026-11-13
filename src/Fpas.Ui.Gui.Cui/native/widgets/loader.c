#include "../fpas_cui.h"

void fpas_cui_loader(cui_ctx *ui, const char *id, int style, float size, int palette, uint32_t rgba) {
    const int node = ui->node_count;
    const cui_loader_opts options = { .style = style, .size = size, .color = fpas_chart_color(palette, rgba) };
    cui_loader_ex(ui, &options);
    fpas_motion_node(ui, node, cui_make_id(ui, id));
}
