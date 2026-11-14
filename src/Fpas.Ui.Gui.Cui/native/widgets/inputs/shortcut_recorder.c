#include "../../fpas_cui.h"

void fpas_cui_shortcut_recorder(cui_ctx *ui, const char *id, int *key, int *mods,
    const char *placeholder, int width_kind, float width, int enabled, int focus, int *nodes) {
    const cui_id root = cui_make_id(ui, id);
    cui_push_id(ui, id);
    const cui_id clear = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "Clear"));
    cui_pop_id(ui);
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui__rec_state *state = cui_state(ui, root, sizeof *state);
    if (!allowed) {
        state->recording = false;
        if (ui->focus == root) ui->focus = 0;
    } else if (focus) {
        ui->focus_next_request = focus == 2 ? clear : root;
        ui->focus_visible = true;
    }
    const cui_shortcut_recorder_opts options = {
        .placeholder = placeholder ? cui__frame_strdup(ui, placeholder, -1) : NULL,
        .width = fpas_extent(width_kind, width) };
    cui_key_combo combo = { *mods, (cui_key)*key };
    cui_push_disabled(ui, !allowed);
    cui_shortcut_recorder_ex(ui, id, &combo, &options);
    cui_pop_disabled(ui);
    *key = combo.key; *mods = combo.mods;
    fpas_own(ui, root, clear, 0, 0);
    nodes[0] = fpas_find(ui, root); nodes[1] = fpas_find(ui, clear);
    ui->last_node = nodes[0]; ui->last_id = root;
}

void fpas_cui_shortcut_label(cui_ctx *ui, int key, int mods, char *text, size_t capacity) {
    snprintf(text, capacity, "%s", cui_shortcut_label(ui, (cui_key_combo){ mods, (cui_key)key }));
}

void fpas_cui_shortcut_display(cui_ctx *ui, int key, int mods, char *text, size_t capacity) {
    const char *label = cui_shortcut_label(ui, (cui_key_combo){ mods, (cui_key)key });
    cui_kbd(ui, label);
    snprintf(text, capacity, "%s", label);
}
