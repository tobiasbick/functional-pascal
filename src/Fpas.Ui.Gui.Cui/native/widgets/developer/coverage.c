#include "../../fpas_cui.h"

void fpas_cui_coverage(cui_ctx *ui, const char *id, const cui_coverage_file * files, int files_count, float height, float low, float high, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_coverage_opts o = { .width = fpas_extent(width_kind, width), .height = height, .low = low, .high = high };
    const cui_id state_key = key ^ 0xC0E1u;
    fpas_own(ui, key, state_key, 0, 0);
    /* CUI caches path segments by reference. Refresh the tree from this call's
       paths without resetting sorting, folds or selection. */
    cui__cov_state *cache = cui__state_lazy(ui, state_key, sizeof *cache, false);
    if (cache && cache->init) {
        cache->init = false;
        cui__cov_build(ui, cache, files, files_count);
    }
    cui_push_disabled(ui, !enabled);
    cui_coverage_result r = cui_coverage_ex(ui,id,files,files_count,&o);
    cui_pop_disabled(ui);

    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->action=r.activated;
    out->x=r.lines;
}
