#include "../../fpas_cui.h"

void fpas_cui_network_waterfall(cui_ctx *ui, const char *id, const cui_net_request * requests, int requests_count, int selection, float height, double dom_loaded, double loaded, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_waterfall_opts o = { .width = fpas_extent(width_kind, width), .height = height, .dom_loaded = dom_loaded, .loaded = loaded, .selected = &selection };
    cui_push_disabled(ui, !enabled);
    cui_waterfall_result r = cui_waterfall_ex(ui,id,requests,requests_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=selection;
    out->action=r.activated;
    out->hover=r.hovered;
    out->count=r.clicked;
}
