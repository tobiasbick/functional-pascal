#include "../fpas_cui.h"

void fpas_cui_regex_tester(cui_ctx *ui, const char *id, char *pattern, size_t pattern_cap, char *text, size_t text_cap, int options, float height, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_regex_tester_opts o = { .width=fpas_extent(width_kind,width), .height=height, .flags=&options };
    const cui_regex_tester_result r = cui_regex_tester_ex(ui,id,pattern,pattern_cap,text,text_cap,&o);
    out->index=options; out->count=r.matches; out->flags=r.valid;
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
