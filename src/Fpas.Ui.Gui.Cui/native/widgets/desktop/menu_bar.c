#include "../../fpas_cui.h"
#include "chrome_input.h"

void fpas_cui_menu_bar_begin(cui_ctx *ui, const char *id, int border, int enabled) {
    const cui_menu_bar_opts opts = { .no_border = !border };
    const bool allowed = enabled && !cui_is_disabled(ui);
    fpas_chrome_input input;
    if (!allowed) {
        cui__menubar_state *state = cui_state(ui, cui_make_id(ui, id), sizeof *state);
        if (state->init && state->open >= 0) {
            cui__popup_set(ui, state->pid[state->open], false);
            state->open = -1;
        }
        cui__bar_release_keys(ui, state);
        state->alt_armed = state->alt_prev = false;
        fpas_chrome_pause(ui, &input);
    }
    cui_menu_bar_begin(ui, id, &opts);
    if (!allowed) fpas_chrome_resume(ui, &input);
    cui_push_disabled(ui, !allowed);
}

void fpas_cui_menu_bar_end(cui_ctx *ui) {
    cui__menubar_state *state = cui__slots(ui)->bar;
    if (state && state->open >= state->build) {
        cui__bar_open(ui, state, -1, false);
        cui__bar_release_keys(ui, state);
    }
    const int node = cui__current(ui);
    cui_menu_bar_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
    cui_pop_disabled(ui);
}

int fpas_cui_menu_bar_menu(cui_ctx *ui, const char *title, int mnemonic, float width,
    int enabled, const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const int *flags, int count, int *nodes, int *open, int *highlighted) {
    const char *caption = cui__frame_strdup(ui, title, -1);
    const cui_id key = cui_make_id(ui, caption), popup = key ^ 0x3E40B4u;
    const cui_menu_bar_menu_opts opts = { .mnemonic = (char)mnemonic, .width = width, .disabled = !enabled };
    const bool visible = cui_menu_bar_menu_begin(ui, caption, &opts);
    const int trigger = fpas_find(ui, key);
    const int chosen = fpas_menu_rows(ui, popup, visible, keys, labels, icons, shortcuts,
        flags, count, nodes, highlighted);
    if (visible) cui_menu_bar_menu_end(ui);
    fpas_menu_snapshot(ui, enabled && !cui_is_disabled(ui), trigger, key, popup,
        count, nodes, open, highlighted);
    fpas_own(ui, key, popup ^ 0x4E4156u, 0, popup);
    return enabled && !cui_is_disabled(ui) ? chosen : -1;
}
