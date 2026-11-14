#include "../../fpas_cui.h"

int fpas_cui_speed_dial(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, const char *icon, const char *tooltip,
    int placement, int enabled, int focus, int *nodes, int *open) {
    if (count < 1 || count > 16) return -1;
    if (ui->disabled_n > 30) return -2;
    const cui_id key = cui_make_id(ui, id);
    const cui_id actions_key = key ^ 0xA0u;
    cui_speed_action actions[16];
    for (int i = 0; i < count; ++i)
        actions[i] = (cui_speed_action){ icons[i], labels[i] };
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui__dial_state *state = cui_state(ui, key, sizeof *state);
    if (!allowed) state->open = false;
    const bool escape = state->open && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    bool action_focused = false;
    if (escape)
        for (int i = 0; i < count; ++i)
            action_focused |= ui->focus == cui__hash(&i, sizeof i, actions_key);
    if (allowed && focus == 1) ui->focus_next_request = key;
    if (allowed && state->open && focus > 1 && focus <= count + 1) {
        const int index = focus - 2;
        ui->focus_next_request = cui__hash(&index, sizeof index, actions_key);
    }
    const cui_speed_dial_opts opts = { .icon = icon, .tooltip = tooltip, .placement = (cui_placement)placement };
    const int scope = ui->disabled_n;
    cui_push_disabled(ui, !allowed);
    const int choice = cui_speed_dial_ex(ui, id, actions, count, &opts);
    /* CUI-008 (docs/future/cui-upstream-bugs.md#cui-008):
       Choosing an action can pop our temporary scope during drawing. */
    if (ui->disabled_n > scope) cui_pop_disabled(ui);
    nodes[0] = ui->last_node;
    *open = allowed && state->open;
    for (int i = 0; i < count; ++i)
        nodes[i + 1] = *open ? fpas_find(ui, cui__hash(&i, sizeof i, actions_key)) : -1;
    if (allowed && ui->focus_visible && (choice >= 0 || action_focused)) ui->focus = key;
    fpas_own(ui, key, key ^ 0x17E5u, key ^ 0x1C0u, 0);
    return allowed ? choice : -1;
}
