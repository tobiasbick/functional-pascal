#include "../fpas_cui.h"

int fpas_cui_segmented(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, int selected, int size, int full_width,
    int enabled, int focus, int *nodes) {
    const cui_id group = cui_make_id(ui, id);
    const char *captions[32], *glyphs[32];
    bool literal = false;
    for (int index = 0; index < count; ++index) {
        const bool quoted = strstr(labels[index], "##") != NULL;
        captions[index] = quoted ? "x" : labels[index];
        glyphs[index] = icons && icons[index] && icons[index][0] ? icons[index] : NULL;
        literal |= quoted;
    }
    fpas_indexed_focus(ui, group, count, enabled ? focus : 0);
    const cui_segmented_opts options = {
        .icons = glyphs, .size = (cui_control_size)size, .full_width = full_width != 0
    };
    int choice = selected;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_segmented_ex(ui, id, captions, count, &choice, &options);
    cui_pop_disabled(ui);
    nodes[0] = fpas_find(ui, group);
    for (int index = 0; index < count; ++index) {
        nodes[index + 1] = fpas_find(ui, cui__hash(&index, sizeof index, group));
        if (captions[index] != labels[index])
            fpas_literal(ui, nodes[index + 1], labels[index], CUI_C(ui, FOREGROUND));
    }
    /* Let native content measurement size literal captions, including leading ##. */
    if (literal) ui->nodes[nodes[0]].pref_hint = 0;
    if (!enabled) ui->nodes[nodes[0]].opacity = CUI__DISABLED;
    ui->last_node = nodes[0];
    ui->last_id = group;
    return changed ? choice : -1;
}
