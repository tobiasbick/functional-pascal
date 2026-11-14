#include "../../fpas_cui.h"

void fpas_cui_dot_display(cui_ctx *ui, const char *id, const char *text, float pitch,
    int palette, uint32_t rgba, int align, int width_kind, float width, int flags) {
    const int node = ui->node_count;
    const cui_dot_display_opts options = { .pitch = pitch, .color = fpas_chart_color(palette, rgba),
        .accent = (flags & 1) != 0, .no_dim = (flags & 2) != 0, .marquee = (flags & 4) != 0,
        .align = align, .width = fpas_extent(width_kind, width) };
    cui_push_id(ui, id);
    cui_dot_display_ex(ui, text, &options);
    cui_pop_id(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}
