#include "../fpas_cui.h"

int fpas_cui_user_menu(cui_ctx *ui, const char *id, const char *name,
    const char *detail, const char *status, const char *const *keys,
    const char *const *labels, const char *const *icons, const char *const *shortcuts,
    const int *flags, int count, int placement, float width, int compact, int enabled,
    int focus, int *nodes, int *open, int *highlighted) {
    cui_push_id(ui, id);
    const cui_id trigger = cui_make_id(ui, name) ^ 0x05E4u;
    const cui_id popup = trigger ^ 0x3E40u;
    if (enabled && focus) ui->focus_next_request = trigger;
    if (!enabled) cui__popup_set(ui, popup, false);
    const cui_user_menu_opts options = { .detail = detail, .status = status,
        .compact = compact != 0, .placement = (cui_placement)placement, .width = width };
    cui_push_disabled(ui, !enabled);
    const bool visible = cui_user_menu_begin(ui, name, &options);
    const int chosen = fpas_menu_rows(ui, popup, visible, keys, labels, icons, shortcuts,
        flags, count, nodes, highlighted);
    if (visible) cui_user_menu_end(ui);
    cui_pop_disabled(ui);
    cui_pop_id(ui);

    fpas_menu_snapshot(ui, enabled, fpas_find(ui, trigger), trigger, popup,
        count, nodes, open, highlighted);
    fpas_own(ui, trigger, popup ^ 0x4E4156u, 0, popup);
    return enabled ? chosen : -1;
}
