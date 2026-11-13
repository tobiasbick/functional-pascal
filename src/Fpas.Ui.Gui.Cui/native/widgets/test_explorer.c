#include "../fpas_cui.h"

void fpas_cui_test_explorer(cui_ctx *ui, const char *id, const cui_test_item * entries, int entries_count, float height, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_test_explorer_opts o = { .width = fpas_extent(width_kind, width), .height = height };
    cui_push_disabled(ui, !enabled);
    cui_test_explorer_result r = cui_test_explorer_ex(ui,id,entries,entries_count,&o);
    cui_pop_disabled(ui);
    fpas_dev_reduce_spinners(ui, root);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=r.selected;
    out->action=r.activated;
    out->hover=r.run;
    out->flags=r.run_all;
    out->count=r.total;
    out->x=r.passed;
    out->y=r.failed;
    out->z=r.skipped;
}
