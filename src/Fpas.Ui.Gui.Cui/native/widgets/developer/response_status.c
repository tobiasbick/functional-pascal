#include "../../fpas_cui.h"

void fpas_cui_response_status(cui_ctx *ui, const char *id, int status, double seconds, double bytes, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui, !enabled);
    cui_response_status(ui,status,seconds,bytes);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    ui->nodes[root].w = fpas_extent(width_kind, width);
    fpas_creative_node(ui, root, key);
}
