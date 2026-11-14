#include "../../fpas_cui.h"

void fpas_cui_keymap(cui_ctx *ui, const char *id, const cui_keymap_binding * bindings, int bindings_count, const char *const * categories, int categories_count, int modifiers, int hide_function_row, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    const cui_keymap_opts o = { .width = fpas_extent(width_kind, width), .categories = categories, .category_count = categories_count, .mods = &modifiers, .no_function_row = hide_function_row };
    /* Keycap paint uses action labels after the bridge call returns. */
    cui_keymap_binding *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)CUI_MAX(1, bindings_count));
    for (int i = 0; i < bindings_count; ++i) {
        copy[i] = bindings[i];
        if (copy[i].action) copy[i].action = cui__frame_strdup(ui, copy[i].action, -1);
    }
    cui_push_disabled(ui, !enabled);
    cui_keymap_result r = cui_keymap_ex(ui,id,copy,bindings_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=modifiers;
    out->action=r.clicked;
    out->count=r.conflicts;
}
