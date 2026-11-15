#include "../../fpas_cui.h"

void fpas_cui_query_builder(cui_ctx *ui, const char *id, const cui_query_field *fields,
    int field_count, cui_query_node *rules, int *count, int capacity, int max_depth,
    int read_only, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_query_opts options = { .width = fpas_extent(width_kind, width),
        .max_depth = max_depth, .read_only = read_only != 0 };
    *out = fpas_dev_empty();

    cui_push_disabled(ui, !allowed);
    const bool changed = cui_query_builder_ex(ui, id, fields, field_count, rules, count, capacity, &options);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->flags = allowed && !options.read_only && changed;
}
