#include "../../fpas_cui.h"

void fpas_cui_problems(cui_ctx *ui, const char *id, const cui_diagnostic * diagnostics, int diagnostics_count, float height, int hide_toolbar, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_problems_opts o = { .width = fpas_extent(width_kind, width), .height = height, .no_toolbar = hide_toolbar };
    cui_push_disabled(ui, !enabled);
    cui_problems_result r = cui_problems_ex(ui,id,diagnostics,diagnostics_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=r.selected;
    out->action=r.activated;
    out->count=r.shown;
}
