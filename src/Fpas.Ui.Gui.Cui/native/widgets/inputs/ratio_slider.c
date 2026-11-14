#include "../../fpas_cui.h"

int fpas_cui_ratio_slider(cui_ctx *ui, const char *id,
    const char *first, const char *second, float *value, int tone, float step, float height, int width_kind, float width, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    cui_push_id(ui, id);
    first = cui__frame_strdup(ui, first, -1);
    second = cui__frame_strdup(ui, second, -1);
    const cui_id key = cui__hash_str(second, cui_make_id(ui, first));
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus == 0) ui->focus_next_request = key;
    const cui_ratio_slider_opts options = { .tone = tone, .step = step, .height = height,
        .width = fpas_extent(width_kind, width), .disabled = !enabled };
    const bool changed = cui_ratio_slider_ex(ui, first, second, value, &options);
    cui_pop_id(ui);
    cui__ratio_paint *paint = ui->nodes[node].paint_data;
    paint->alen = (int)strlen(first); paint->blen = (int)strlen(second);
    parts[0] = node;
    fpas_creative_node(ui, node, key);
    return enabled && changed;
}
