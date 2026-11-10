#include "../fpas_cui.h"

void fpas_cui_sidebar_clear(cui_ctx *ui, const char *group) {
    cui__nav_group *state = cui__state_lazy(ui, cui_make_id(ui, group), sizeof *state, false);
    if (state) {
        state->has_target = false;
        state->init = false;
    }
}

int fpas_cui_sidebar_item(cui_ctx *ui, const char *group, const char *id, const char *text,
                         const char *icon, const char *badge, int enabled, int selected, int focus) {
    cui_id key = cui_make_id(ui, id) ^ 0x4A570005u;
    if (focus && enabled) ui->focus_next_request = key;
    const cui_nav_opts options = { .group = group, .icon = icon, .badge = badge, .selected = selected != 0 };
    cui_push_disabled(ui, !enabled);
    bool activated = cui_nav_item_ex(ui, id, &options);
    cui_pop_disabled(ui);
    int node = fpas_find(ui, key);
    if (node >= 0) {
        fpas_literal(ui, node, text, CUI_C(ui, FOREGROUND));
        const cui__nav_group *state = cui_state(ui, cui_make_id(ui, group), sizeof *state);
        /* The animated highlight needs prior geometry. Paint the active row on its first frame. */
        if (selected && !state->init) ui->nodes[node].bg = CUI_C(ui, ACTIVE);
    }
    return activated ? 1 : 0;
}
