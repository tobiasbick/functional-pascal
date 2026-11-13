#include "../fpas_cui.h"

void fpas_cui_api_explorer(cui_ctx *ui, const char *id, const cui_api_endpoint * endpoints, int endpoints_count, int selection, float height, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_api_explorer_opts o = { .width = fpas_extent(width_kind, width), .height = height, .selected = &selection };
    cui_push_disabled(ui, !enabled);
    cui_api_explorer_result r = cui_api_explorer_ex(ui,id,endpoints,endpoints_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=r.selected;
    out->action=r.try_it;
}
