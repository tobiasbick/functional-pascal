// [Documentation](../../../../../docs/ui/gui/developer-tools/key-value-editor.md)
#include "../../fpas_cui.h"

/** Draws the editor while preserving the editable layout in disabled scopes. */
void fpas_cui_key_value_editor(cui_ctx *ui, const char *id, cui_kv_pair *pairs, int *count, int capacity, const char *key_label, const char *value_label, int toggles, int secrets, int read_only, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_kv_editor_opts o = {
        .key_placeholder = key_label ? cui__frame_strdup(ui, key_label, -1) : NULL,
        .value_placeholder = value_label ? cui__frame_strdup(ui, value_label, -1) : NULL,
        .no_toggles=toggles, .no_secrets=secrets, .read_only=read_only, .width=fpas_extent(width_kind,width) };
    const bool changed = cui_kv_editor_ex(ui,id,pairs,count,capacity,&o);
    out->flags = allowed && !o.read_only && changed;
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
