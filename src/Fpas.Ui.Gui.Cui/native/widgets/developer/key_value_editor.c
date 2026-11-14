#include "../../fpas_cui.h"

void fpas_cui_key_value_editor(cui_ctx *ui, const char *id, cui_kv_pair *pairs, int *count, int capacity, const char *key_label, const char *value_label, int toggles, int secrets, int read_only, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_kv_editor_opts o = { .key_placeholder=key_label, .value_placeholder=value_label,
        .no_toggles=toggles, .no_secrets=secrets, .read_only=read_only || !enabled, .width=fpas_extent(width_kind,width) };
    out->flags = cui_kv_editor_ex(ui,id,pairs,count,capacity,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
