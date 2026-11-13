#include "../fpas_cui.h"

void fpas_cui_request_bar(cui_ctx *ui, const char *id, int method, char *url, size_t capacity, int loading, const char *placeholder, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_request_bar_opts o = { .width=fpas_extent(width_kind,width), .loading=loading, .placeholder=placeholder };
    const cui_request_bar_result r = cui_request_bar_ex(ui,id,&method,url,capacity,&o);
    out->index=method; out->flags=(r.changed ? 1 : 0) | (r.send ? 2 : 0) | (r.cancel ? 4 : 0);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
