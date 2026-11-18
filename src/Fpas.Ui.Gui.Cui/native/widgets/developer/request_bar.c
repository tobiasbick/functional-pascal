#include "../../fpas_cui.h"

static void fpas_request_bar_retire(cui_ctx *ui, cui_id key) {
    const cui_id method = cui__hash_str("method", key);
    const cui_id field = cui__hash_str("url", key);
    fpas_forget_state(ui, method);
    fpas_forget_state(ui, (method ^ 0x5E1Eu) ^ 0x4E4156u);
    fpas_forget_state(ui, field);
}

void fpas_cui_request_bar(cui_ctx *ui, const char *id, int method, char *url, size_t capacity, int loading, const char *placeholder, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    const cui_id field = cui__hash_str("url", key);
    const cui_id popup = cui__hash_str("method", key) ^ 0x5E1Eu;
    const bool allowed = enabled && !cui_is_disabled(ui);
    *out = fpas_dev_empty();
    fpas_own(ui, key, fpas_dev_text_slot(field), 0, popup);
    fpas_on_forget(ui, key, fpas_request_bar_retire);
    if (!allowed) cui__popup_set(ui, popup, false);
    fpas_text_binding *binding = url && capacity ? fpas_dev_bind_text(ui, field, url, capacity, false) : NULL;
    cui_push_disabled(ui, !allowed);
    const cui_request_bar_opts o = { .width = fpas_extent(width_kind, width), .loading = loading,
        .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL };
    const cui_request_bar_result r = cui_request_bar_ex(ui,id,&method,url,capacity,&o);
    out->index=method; out->flags=(r.changed ? 1 : 0) | (r.send ? 2 : 0) | (r.cancel ? 4 : 0);
    cui_pop_disabled(ui);
    if (binding) fpas_text_store(ui, binding, url, strlen(url), capacity);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
