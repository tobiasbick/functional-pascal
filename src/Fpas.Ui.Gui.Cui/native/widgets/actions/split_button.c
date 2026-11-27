// [Documentation](../../../../../docs/ui/gui/actions/split-button.md)
#include "../../fpas_cui.h"

int fpas_cui_split_button(cui_ctx *ui, const char *id, const char *label, const char *icon,
                          const char *const *labels, const char *const *icons, int count,
                          int variant, int size, int enabled, int focus, int *nodes,
                          int *open, int *highlighted) {
    const cui_id gid = cui_make_id(ui, id) ^ 0x5B17u;
    const cui_id pid = gid ^ 0x3E40u;
    const char *keyed_label = cui__frame_printf(ui, "x%s", id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) {
        cui_push_id(ui, keyed_label);
        ui->focus_next_request = focus == 1 ? cui_make_id(ui, keyed_label) :
            cui__hash_str(CUI_ICON_CHEVRON_DOWN, cui_make_id(ui, "More options"));
        cui_pop_id(ui);
    }
    const cui_popup *popup = cui__popup_get(ui, pid, false);
    const bool was_open = popup && popup->open;
    /* CUI-034 (docs/future/cui-upstream-bugs.md#cui-034): disabling the split
       must also close the menu so its highlighted command cannot activate. */
    if (!allowed && was_open) cui__popup_set(ui, pid, false);

    /* Reserve text slots while keeping CUI's visible-label syntax out of the captions. */
    const char **slots = cui__frame_alloc(ui, (size_t)count * sizeof *slots);
    const char **glyphs = cui__frame_alloc(ui, (size_t)count * sizeof *glyphs);
    for (int i = 0; i < count; ++i) {
        slots[i] = "x";
        glyphs[i] = icons[i][0] ? icons[i] : NULL;
        nodes[2 + i] = -1;
    }
    const cui_split_button_opts options = {
        .variant = fpas_button_variant(variant), .size = (cui_control_size)size,
        .icon = icon, .item_icons = glyphs, .disabled = !allowed
    };
    cui_push_disabled(ui, !allowed);
    const int selected = cui_split_button_ex(ui, keyed_label, slots, count, &options);
    cui_pop_disabled(ui);

    const int group = ui->last_node;
    nodes[0] = ui->nodes[group].first;
    nodes[1] = ui->nodes[nodes[0]].next;
    fpas_literal(ui, nodes[0], label, CUI_C(ui, FOREGROUND));

    popup = cui__popup_get(ui, pid, false);
    *open = allowed && popup && popup->open;
    /* CUI-033 (docs/future/cui-upstream-bugs.md#cui-033): an outside close on
       the chevron press must not reopen the menu on release. */
    if (allowed && was_open && !*open && ui->mpressed[CUI_MOUSE_LEFT] &&
        ui->active == ui->nodes[nodes[1]].id) ui->active = 0;
    *highlighted = -1;
    if (fpas_find(ui, pid) >= 0) {
        cui_push_id_int(ui, (int)pid);
        for (int i = 0; i < count; ++i) {
            cui_push_id_int(ui, i);
            const int item = fpas_find(ui, cui_make_id(ui, slots[i]));
            cui_pop_id(ui);
            if (item >= 0) {
                fpas_literal(ui, item, labels[i], CUI_C(ui, FOREGROUND));
                if (*open) nodes[i + 2] = item;
            }
        }
        cui_pop_id(ui);
        if (*open) {
            const cui__menu_nav *navigation = cui_state(ui, pid ^ 0x4E4156u, sizeof *navigation);
            *highlighted = navigation->highlight;
        }
    }
    ui->last_node = group;
    ui->last_id = gid;
    fpas_own(ui, ui->nodes[nodes[0]].id, gid, pid ^ 0x4E4156u, pid);
    return allowed ? selected : -1;
}
