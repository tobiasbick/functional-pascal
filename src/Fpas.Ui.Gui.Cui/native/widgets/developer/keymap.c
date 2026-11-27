// [Documentation](../../../../../docs/ui/gui/developer-tools/keymap.md)
#include "../../fpas_cui.h"

void fpas_cui_keymap(cui_ctx *ui, const char *id, const cui_keymap_binding *bindings, int bindings_count,
    const char *const *categories, int categories_count, int modifiers, int hide_function_row,
    int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_id state_key = key ^ 0x4B3Bu;
    const cui__km_state *state = cui_state(ui, state_key, sizeof *state);
    const bool controlled = modifiers >= 0;
    const int before = controlled ? modifiers : state->mods;
    cui_push_id(ui, id);
    const cui_id group = cui_make_id(ui, "mods");
    cui_pop_id(ui);
    int selection = 0;
    for (int i = 0; i < (int)CUI_COUNTOF(cui__km_sets); ++i)
        if (cui__km_sets[i] == before) { selection = i; break; }
    const cui_id focus = cui__hash(&selection, sizeof selection, group);
    if (enabled && (flags & 2) && !cui__keys_covered(ui)) ui->focus_next_request = focus;
    const cui_id previously_focused = fpas_dev_focused_item(ui, group, (int)CUI_COUNTOF(cui__km_sets));
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = enabled && previously_focused && fpas_dev_save_keys(ui, previously_focused, keys);
    *out = fpas_dev_empty();
    const cui_keymap_opts o = { .width = fpas_extent(width_kind, width), .categories = categories,
        .category_count = categories_count, .mods = controlled ? &modifiers : NULL, .no_function_row = hide_function_row };
    /* Keycap paint uses action labels after the bridge call returns. */
    cui_keymap_binding *copy = cui__frame_alloc(ui, sizeof *copy * (size_t)CUI_MAX(1, bindings_count));
    for (int i = 0; i < bindings_count; ++i) {
        copy[i] = bindings[i];
        if (copy[i].action) copy[i].action = cui__frame_strdup(ui, copy[i].action, -1);
    }
    cui_push_disabled(ui, !enabled);
    cui_keymap_result r = cui_keymap_ex(ui,id,copy,bindings_count,&o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_dev_watch_width(ui, root, key ^ 0x4B3Au);
    fpas_creative_node(ui, root, key);
    fpas_own(ui, key, group, state_key, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    const cui_id focused = fpas_dev_focused_item(ui, group, (int)CUI_COUNTOF(cui__km_sets));
    owner->focus = focused ? focused : focus;
    out->index = controlled ? modifiers : state->mods;
    out->flags = out->index != before;
    out->action = r.clicked;
    out->count = r.conflicts;
}
