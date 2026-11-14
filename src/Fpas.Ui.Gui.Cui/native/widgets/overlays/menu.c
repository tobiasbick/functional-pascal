#include "../../fpas_cui.h"

static int fpas_menu_rows(cui_ctx *ui, cui_id popup, bool visible,
    const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const int *flags, int count, int *nodes, int *highlighted) {
    for (int i = 1; i <= count; ++i) nodes[i] = -1;
    *highlighted = -1;
    if (!visible) return -1;
    int chosen = -1, ordinal = 0;
    const cui__menu_nav *nav = cui_state(ui, popup ^ 0x4E4156u, sizeof *nav);
    for (int i = 0; i < count; ++i) {
        if ((flags[i] & 3) == 1) cui_menu_label(ui, labels[i]);
        else if ((flags[i] & 3) == 2) cui_menu_separator(ui);
        else {
            const cui_menu_item_opts item = { .icon = icons[i][0] ? icons[i] : NULL,
                .shortcut = shortcuts[i][0] ? shortcuts[i] : NULL,
                .disabled = (flags[i] & 4) != 0, .danger = (flags[i] & 8) != 0,
                .checked = (flags[i] & 16) != 0 };
            cui_push_id(ui, keys[i]);
            if (cui_menu_item_ex(ui, "x", &item)) chosen = i;
            /* Menu items do not set last_node; find their stable keyed node. */
            const int command = fpas_find(ui, cui_make_id(ui, "x"));
            fpas_literal(ui, command, labels[i], CUI_C(ui, FOREGROUND));
            nodes[i + 1] = command;
            cui_pop_id(ui);
            if (nav->highlight == ordinal++) *highlighted = i;
        }
    }
    return chosen;
}

static void fpas_menu_snapshot(cui_ctx *ui, int enabled, int trigger, cui_id anchor, cui_id pid,
    int count, int *nodes, int *open, int *highlighted) {
    const cui_popup *popup = cui__popup_get(ui, pid, false);
    *open = enabled && popup && popup->open;
    if (!*open) {
        *highlighted = -1;
        for (int i = 1; i <= count; ++i) nodes[i] = -1;
    }
    nodes[0] = trigger;
    ui->last_node = trigger;
    ui->last_id = anchor;
}

int fpas_cui_menu(cui_ctx *ui, const char *id, const char *label, const char *icon,
    const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const int *flags, int count, int variant, int placement,
    float width, int icon_only, int enabled, int focus, int context_menu, int *nodes,
    int *open, int *highlighted, unsigned int *owner) {
    const char *caption = cui__frame_printf(ui, "x%s", id);
    const cui_id key = cui_make_id(ui, context_menu ? id : caption), pid = key ^ (context_menu ? 0xC7Cu : 0x3E40u);
    const char *glyph = icon ? icon : CUI_ICON_ELLIPSIS;
    int trigger = context_menu ? ui->last_node : -1;
    if (context_menu) {
        if (!ui->nodes[trigger].id) ui->nodes[trigger].id = key;
        cui__mark_interactive(ui, trigger, 0);
        ui->last_id = ui->nodes[trigger].id;
    }
    const cui_id anchor = context_menu ? ui->last_id : icon_only ? cui__hash_str(glyph, key) : key;
    *owner = context_menu ? pid : anchor;
    if (enabled && focus) ui->focus_next_request = anchor;
    if (!enabled) cui__popup_set(ui, pid, false);
    const cui_menu_opts options = { .icon = icon, .variant = fpas_button_variant(variant),
        .placement = (cui_placement)placement, .width = width, .icon_only = icon_only != 0 };
    cui_push_disabled(ui, !enabled);
    const bool visible = context_menu ? enabled && cui_context_menu_begin(ui, id) : cui_menu_begin(ui, caption, &options);
    if (!context_menu) {
        trigger = fpas_find(ui, anchor);
        if (!icon_only) fpas_literal(ui, trigger, label, CUI_C(ui, FOREGROUND));
        else {
            ui->last_node = trigger; ui->last_id = anchor;
            cui_tooltip(ui, label);
        }
    }
    const int chosen = fpas_menu_rows(ui, pid, visible, keys, labels, icons, shortcuts,
        flags, count, nodes, highlighted);
    if (visible) {
        if (context_menu) cui_context_menu_end(ui); else cui_menu_end(ui);
    }
    cui_pop_disabled(ui);
    fpas_menu_snapshot(ui, enabled, trigger, anchor, pid, count, nodes, open, highlighted);
    fpas_own(ui, *owner, pid ^ 0x4E4156u, 0, pid);
    return enabled ? chosen : -1;
}
