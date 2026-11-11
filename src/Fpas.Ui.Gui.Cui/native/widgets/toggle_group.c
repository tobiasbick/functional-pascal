#include "../fpas_cui.h"

uint32_t fpas_cui_toggle_group(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, uint32_t selected, int size, int exclusive,
    int icon_only, int enabled, int focus, int *nodes) {
    const cui_id group = cui_make_id(ui, id);
    const char *captions[32], *glyphs[32];
    bool values[32];
    for (int index = 0; index < count; ++index) {
        glyphs[index] = icons && icons[index] && icons[index][0] ? icons[index] : NULL;
        captions[index] = labels[index];
        if (strstr(labels[index], "##"))
            captions[index] = icon_only && glyphs[index] ? NULL : "x";
        values[index] = (selected & (UINT32_C(1) << index)) != 0;
    }
    fpas_indexed_focus(ui, group, count, enabled ? focus : 0);
    const cui_toggle_group_opts options = {
        .icons = glyphs, .exclusive = exclusive != 0, .icon_only = icon_only != 0,
        .size = (cui_control_size)size
    };
    cui_push_disabled(ui, !enabled);
    cui_toggle_group_ex(ui, id, captions, count, values, &options);
    cui_pop_disabled(ui);
    nodes[0] = ui->last_node;

    uint32_t proposal = 0;
    for (int index = 0; index < count; ++index) {
        const cui_id key = cui__hash(&index, sizeof index, group);
        const int node = fpas_find(ui, key);
        nodes[index + 1] = node;
        if (captions[index] != labels[index]) {
            if (captions[index]) {
                fpas_literal(ui, node, labels[index], CUI_C(ui, FOREGROUND));
            } else if (enabled) {
                ui->last_id = key;
                cui_tooltip(ui, labels[index]);
            }
        }
        if (values[index]) proposal |= UINT32_C(1) << index;
    }
    if (!enabled) ui->nodes[nodes[0]].opacity = CUI__DISABLED;
    ui->last_node = nodes[0];
    ui->last_id = group;
    return proposal;
}
