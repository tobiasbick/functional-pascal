#include "../fpas_cui.h"

int fpas_cui_palette_strip(cui_ctx *ui, const char *id,
    const char *label, uint32_t *colors, int *locked, int count, uint32_t *seed, const char *display, float height, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label), row = key ^ 0x9A11u;
    cui_color swatches[CUI__PAL_MAX];
    bool locks[CUI__PAL_MAX];
    for (int i = 0; i < count; ++i) {
        swatches[i] = fpas_chart_color(-1, colors[i]);
        locks[i] = locked[i] != 0;
    }
    const cui_palette_strip_opts options = { .seed = seed, .value = display,
        .no_shuffle = (flags & 2) != 0, .height = height, .plain = (flags & 8) != 0 };
    fpas_well_focus(ui, enabled, row, key, count, !options.no_shuffle, focus);
    cui_push_disabled(ui, !enabled);
    const int result = cui_palette_strip_ex(ui, label, swatches, locks, count, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) {
        parts[i] = fpas_find(ui, fpas_well_part(row, i));
        colors[i] = fpas_well_rgba(swatches[i]);
        locked[i] = locks[i];
    }
    parts[count] = options.no_shuffle ? -1 : fpas_find(ui, key ^ CUI__WELL_TOOL_SALT);
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
