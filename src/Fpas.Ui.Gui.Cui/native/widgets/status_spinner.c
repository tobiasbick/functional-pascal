#include "../fpas_cui.h"

int fpas_cui_status_spinner(cui_ctx *ui, const char *id, int status, float size,
    int palette, uint32_t rgba, int keep) {
    const int first = ui->node_count;
    const cui_status_spinner_opts options = { .size = size,
        .color = fpas_chart_color(palette, rgba), .keep = keep != 0 };
    const bool visible = cui_status_spinner_ex(ui, id, status, &options);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
    return visible;
}
