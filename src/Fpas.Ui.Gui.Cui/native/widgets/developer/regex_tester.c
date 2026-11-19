// [Documentation](../../../../../docs/ui/gui/more-developer-tools/regex-tester.md)
#include "../../fpas_cui.h"

static void fpas_regex_tester_retire(cui_ctx *ui, cui_id key) {
    const cui_id pattern = cui__hash_str("pattern", key);
    const cui_id sample = cui__hash_str("text", key);
    fpas_forget_state(ui, fpas_dev_text_slot(pattern));
    fpas_forget_state(ui, pattern);
    fpas_forget_state(ui, sample ^ CUI__ED_SALT);
    fpas_forget_state(ui, sample ^ 0xED15C011u);
    fpas_forget_state(ui, sample ^ 0xED1B0D1u);
    fpas_forget_state(ui, cui__hash_str("find", sample));
    fpas_forget_state(ui, cui__hash_str("goto", sample));
}

void fpas_cui_regex_tester(cui_ctx *ui, const char *id, char *pattern, size_t pattern_cap, char *text, size_t text_cap, int options, float height, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    const cui_id pattern_field = cui__hash_str("pattern", key);
    const cui_id sample_field = cui__hash_str("text", key);
    *out = fpas_dev_empty();
    fpas_own(ui, key, key ^ 0x2E6Eu, fpas_dev_text_slot(sample_field), 0);
    fpas_on_forget(ui, key, fpas_regex_tester_retire);
    fpas_text_binding *pattern_binding = NULL, *sample_binding = NULL;
    if (pattern && pattern_cap && text && text_cap) {
        pattern_binding = fpas_dev_bind_text(ui, pattern_field, pattern, pattern_cap, false);
        sample_binding = fpas_dev_bind_text(ui, sample_field, text, text_cap, true);
    }
    cui_push_disabled(ui,!enabled);
    const cui_regex_tester_opts o = { .width=fpas_extent(width_kind,width), .height=height, .flags=&options };
    const cui_regex_tester_result r = cui_regex_tester_ex(ui,id,pattern,pattern_cap,text,text_cap,&o);
    out->index=options; out->count=r.matches; out->flags=r.valid;
    cui_pop_disabled(ui);
    if (pattern_binding) {
        fpas_text_store(ui, pattern_binding, pattern, strlen(pattern), pattern_cap);
        fpas_text_store(ui, sample_binding, text, strlen(text), text_cap);
    }
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
