#include "../fpas_cui.h"

int fpas_cui_aspect_picker(cui_ctx *ui, const char *id,
    const char *label, int *value, const int *widths, const int *heights, const char *const *labels, int count, const char *custom, int base, const char *display, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label), row = key ^ 0xA5EC7u;
    cui_aspect ratios[CUI__ASPECT_MAX];
    for (int i = 0; i < count; ++i)
        ratios[i] = (cui_aspect){ widths[i], heights[i], labels[i][0] ? labels[i] : NULL };
    const cui_aspect_picker_opts options = { .ratios = count ? ratios : NULL, .count = count,
        .custom = (flags & 2) != 0, .custom_label = custom, .base = base,
        .value = display, .plain = (flags & 8) != 0 };
    const int total = (count ? count : CUI_COUNTOF(cui__aspect_photo)) + options.custom;
    fpas_well_focus(ui, enabled, row, key, total, false, focus);
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_aspect_picker_ex(ui, label, value, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < total; ++i) parts[i] = fpas_find(ui, fpas_well_part(row, i));
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled && changed;
}
