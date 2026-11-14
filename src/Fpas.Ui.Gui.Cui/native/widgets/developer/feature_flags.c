#include "../../fpas_cui.h"

void fpas_cui_feature_flags(cui_ctx *ui, const char *id, cui_feature_flag *flags, int count, const char *const *environments, int environment_count, float height, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_feature_flags_opts o = { .width=fpas_extent(width_kind,width), .height=height };
    const cui_feature_flags_result r = cui_feature_flags_ex(ui,id,flags,count,environments,environment_count,&o);
    out->index=r.flag; out->action=r.env;
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
