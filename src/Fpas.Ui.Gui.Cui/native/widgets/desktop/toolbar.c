// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/toolbar.md)
#include "../../fpas_cui.h"

void fpas_cui_toolbar_begin(cui_ctx *ui, const char *id, int style, int no_border, int enabled) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id overflow = key ^ 0x0F10Fu;
    fpas_own(ui, key, 0, 0, overflow);
    const cui_toolbar_opts options = { .style = (cui_toolbar_style)style, .no_border = no_border != 0 };
    cui_push_disabled(ui, !enabled);
    cui_toolbar_begin(ui, id, &options);
    if (cui_is_disabled(ui)) {
        /* CUI-168 (docs/future/cui-upstream-bugs.md#cui-168): discard a queued overflow choice. */
        cui__slots(ui)->tb->st->fire = 0;
        if (cui__popup_get(ui, overflow, false)) cui__popup_set(ui, overflow, false);
    }
}

void fpas_cui_toolbar_end(cui_ctx *ui) {
    cui__toolbar_build *bar = cui__slots(ui)->tb;
    const cui_id key = bar ? bar->id : 0;
    const cui_popup *overflow = bar ? cui__popup_get(ui, key ^ 0x0F10Fu, false) : NULL;
    const bool covered = !(overflow && overflow->open) && cui__keys_covered(ui);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    const uint8_t down = ui->kpressed[CUI_KEY_DOWN];
    if (covered) {
        fpas_well_set_keys(ui, (fpas_well_keys){0});
        ui->kpressed[CUI_KEY_DOWN] = 0;
    }
    cui_toolbar_end(ui);
    if (covered) {
        fpas_well_set_keys(ui, keys);
        ui->kpressed[CUI_KEY_DOWN] = down;
    }
    cui_pop_disabled(ui);
    ui->last_id = key;
    ui->last_node = fpas_find(ui, key);
}

int fpas_cui_toolbar_button(cui_ctx *ui, const char *icon, const char *label, const char *tooltip,
    const char *shortcut, int toggle, int tone, int flags) {
    /* CUI's overflow menu is built at EndToolbar and borrows its icons. */
    if (icon) icon = cui__frame_strdup(ui, icon, -1);
    bool on = toggle > 0;
    const cui_tool_opts options = { .label = label, .tooltip = tooltip, .shortcut = shortcut,
        .toggle = toggle >= 0 ? &on : NULL, .tone = (cui_tone)tone, .disabled = (flags & 1) != 0,
        .flyout = (flags & 2) != 0 };
    const bool primary = (flags & 4) != 0;
    /* CUI-131 (docs/future/cui-upstream-bugs.md#cui-131): leave activation keys to the popup. */
    const bool covered = cui__keys_covered(ui);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){0});
    const bool clicked = primary ? cui_toolbar_primary_ex(ui, icon, &options)
        : cui_toolbar_button_ex(ui, icon, &options);
    if (covered) fpas_well_set_keys(ui, keys);
    return clicked ? (!primary && on ? 1 : 0) : -1;
}

int fpas_cui_toolbar_radio(cui_ctx *ui, const char *id, const char *const *icons,
    const char *const *labels, int count, int value) {
    if (count <= 0) return -1;
    if (count > 32) count = 32;
    const char **copy = cui__frame_alloc(ui, (size_t)count * sizeof *copy);
    for (int i = 0; i < count; ++i)
        copy[i] = icons[i] ? cui__frame_strdup(ui, icons[i], -1) : NULL;
    const cui_toolbar_radio_opts options = { .labels = labels };
    const fpas_well_keys keys = fpas_well_pressed(ui);
    /* CUI-156 (docs/future/cui-upstream-bugs.md#cui-156): follow the displayed order in RTL. */
    const cui_id row = cui_make_id(ui, id);
    const bool rtl = (keys.left || keys.right) && count > 1 &&
        cui__prev_rect(ui, fpas_well_part(row, 0)).x > cui__prev_rect(ui, fpas_well_part(row, 1)).x;
    const bool covered = cui__keys_covered(ui);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){0});
    else if (rtl) fpas_well_set_keys(ui, (fpas_well_keys){keys.right, keys.left, keys.enter, keys.space});
    const int start = ui->node_count;
    const bool changed = cui_toolbar_radio_ex(ui, id, copy, count, &value, &options);
    if (rtl || covered) fpas_well_set_keys(ui, keys);
    /* CUI-157 (docs/future/cui-upstream-bugs.md#cui-157): paint the ring on the final focus target. */
    for (int i = start; i < ui->node_count; ++i)
        if (ui->nodes[i].flags & CUI_NF_FOCUSABLE) fpas_well_choice_focus(ui, i);
    return changed && !cui_is_disabled(ui) ? value : -1;
}

int fpas_cui_toolbar_cycle(cui_ctx *ui, const char *id, const char *const *labels,
    int count, int value, const char *tooltip, const char *shortcut, int tone, int flags) {
    const cui_tool_opts options = { .tooltip = tooltip, .shortcut = shortcut,
        .tone = (cui_tone)tone, .disabled = (flags & 1) != 0, .flyout = (flags & 2) != 0 };
    const bool covered = cui__keys_covered(ui);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){0});
    const bool changed = cui_toolbar_cycle_ex(ui, id, labels, count, &value, &options);
    if (covered) fpas_well_set_keys(ui, keys);
    /* CUI-167 (docs/future/cui-upstream-bugs.md#cui-167): a singleton has no next choice. */
    return changed && count > 1 ? value : -1;
}
