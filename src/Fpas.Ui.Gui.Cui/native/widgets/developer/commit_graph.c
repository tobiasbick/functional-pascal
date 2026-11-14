#include "../../fpas_cui.h"

void fpas_cui_commit_graph(cui_ctx *ui, const char *id, const cui_commit * commits, int commits_count, int selection, float height, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_commit_graph_opts o = { .width = fpas_extent(width_kind, width), .height = height, .selected = &selection };
    cui_push_disabled(ui, !enabled);
    cui_commit_graph_result r = cui_commit_graph_ex(ui,id,commits,commits_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=selection;
    out->action=r.activated;
    out->hover=r.hovered;
    out->count=r.clicked;
}
