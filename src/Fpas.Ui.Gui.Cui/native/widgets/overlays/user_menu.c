#include "../../fpas_cui.h"

static void fpas_user_menu_name(cui_ctx *ui, int row, const char *name, float size,
    const char *status, bool show_name) {
    const int avatar = ui->nodes[row].first;
    const int next = ui->nodes[avatar].next;
    const int last = ui->nodes[row].last;
    if (show_name) fpas_text(ui, ui->nodes[next].first, name);

    /* Use CUI's own initials and color for the display name, in the original avatar slot. */
    const cui_avatar_opts options = { .size = size, .status = status };
    const int replacement = ui->node_count;
    cui__open(ui, row);
    cui_avatar_ex(ui, name, &options);
    cui__close(ui);
    /* The native avatar is appended before we replace the original first child. */
    if (next >= 0) {
        ui->nodes[last].next = -1;
        ui->nodes[row].last = last;
    }
    ui->nodes[row].first = replacement;
    ui->nodes[replacement].next = next;
    --ui->nodes[row].child_count;
}

int fpas_cui_user_menu(cui_ctx *ui, const char *id, const char *name,
    const char *detail, const char *status, const char *const *keys,
    const char *const *labels, const char *const *icons, const char *const *shortcuts,
    const int *flags, int count, int placement, float width, int compact, int enabled,
    int focus, int *nodes, int *open, int *highlighted) {
    const bool allowed = enabled && !cui_is_disabled(ui);
    const char *caption = "account";
    cui_push_id(ui, id);
    const cui_id trigger = cui_make_id(ui, caption) ^ 0x05E4u;
    const cui_id popup = trigger ^ 0x3E40u;
    if (allowed && focus) ui->focus_next_request = trigger;
    if (!allowed) cui__popup_set(ui, popup, false);
    const cui_user_menu_opts options = { .detail = detail, .status = status,
        .compact = compact != 0, .placement = (cui_placement)placement, .width = width };
    cui_push_disabled(ui, !allowed);
    const bool visible = cui_user_menu_begin(ui, caption, &options);
    const int row = fpas_find(ui, trigger);
    fpas_user_menu_name(ui, row, name, compact ? 28 : 32, status, !compact);
    if (visible) fpas_user_menu_name(ui, ui->nodes[cui__current(ui)].first, name, 36, NULL, true);
    const int chosen = fpas_menu_rows(ui, popup, visible, keys, labels, icons, shortcuts,
        flags, count, nodes, highlighted);
    if (visible) cui_user_menu_end(ui);
    cui_pop_disabled(ui);
    cui_pop_id(ui);

    fpas_menu_snapshot(ui, allowed, row, trigger, popup, count, nodes, open, highlighted);
    fpas_own(ui, trigger, popup ^ 0x4E4156u, 0, popup);
    return allowed ? chosen : -1;
}
