#include "../fpas_cui.h"

int fpas_cui_skeleton_group_begin(cui_ctx *ui, const char *id, int width_kind, float width, float gap, int dither) {
    const int first = ui->node_count;
    const cui_skeleton_group_opts options = { .width = fpas_extent(width_kind, width),
        .gap = gap, .dither = dither != 0 };
    cui_skeleton_group_begin_ex(ui, &options);
    ui->nodes[first].id = cui_make_id(ui, id);
    return first;
}

void fpas_cui_skeleton_group_end(cui_ctx *ui, int node) {
    cui_skeleton_group_end(ui);
    fpas_creative_node(ui, node, ui->nodes[node].id);
}
