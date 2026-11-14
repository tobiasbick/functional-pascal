#include "../../fpas_cui.h"

void fpas_cui_context_meter(cui_ctx *ui, const char *id, float used, float limit,
    const fpas_viz_item *parts, int count, const char *unit, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_viz_item *items = cui__frame_alloc(ui, sizeof *items * (size_t)CUI_MAX(1, count));
    for (int i = 0; i < count; ++i) {
        items[i] = (cui_viz_item){ cui__frame_strdup(ui, parts[i].label, -1), parts[i].value,
            fpas_chart_color(parts[i].palette, parts[i].rgba) };
    }
    const cui_context_opts opts = { .parts = items, .part_count = count, .unit = unit };
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    cui_push_disabled(ui, !enabled);
    cui_context_meter_ex(ui, id, used, limit, &opts);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
}
