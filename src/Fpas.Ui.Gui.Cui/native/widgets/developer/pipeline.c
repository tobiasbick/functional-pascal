#include "../../fpas_cui.h"

void fpas_cui_pipeline(cui_ctx *ui, const char *id, const char *const * stages, int stages_count, const cui_pipeline_job * jobs, int jobs_count, int selection, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_pipeline_opts o = { .width = fpas_extent(width_kind, width), .selected = &selection };
    cui_push_disabled(ui, !enabled);
    cui_pipeline_result r = cui_pipeline_ex(ui,id,stages,stages_count,jobs,jobs_count,&o);
    cui_pop_disabled(ui);
    fpas_dev_reduce_spinners(ui, root);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=selection;
    out->action=r.clicked;
    out->hover=r.hovered;
}
