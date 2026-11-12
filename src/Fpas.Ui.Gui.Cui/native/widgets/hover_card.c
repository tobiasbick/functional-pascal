#include "../fpas_cui.h"

int fpas_cui_hover_card_begin(cui_ctx *ui, const char *id, int placement,
    float width, float delay, int enabled, int *node, unsigned int *owner) {
    *node = -1;
    *owner = 0;
    const cui_id anchor = ui->last_id;
    if (!enabled || !anchor || cui_is_disabled(ui)) return 0;

    const cui_id key = cui_make_id(ui, id) ^ 0x4C4Du;
    bool *wait_for_leave = cui_state(ui, key, sizeof *wait_for_leave);
    const cui_id hot = ui->hot;
    if (hot != anchor) *wait_for_leave = false;
    const cui_popup *previous = cui__popup_get(ui, key, false);
    const bool was_open = previous && previous->open;
    const cui_hover_card_opts options = {
        .placement = (cui_placement)placement, .width = width, .delay = delay
    };
    /* CUI-016 (docs/future/cui-upstream-bugs.md#cui-016): let native dismissal
       finish before admitting another hover on the same trigger. */
    const bool suppress = *wait_for_leave;
    if (suppress) ui->hot = 0;
    const bool draw = cui_hover_card_begin(ui, id, &options);
    if (suppress) ui->hot = hot;
    const cui_popup *popup = cui__popup_get(ui, key, false);
    if (!popup) return 0;
    if (was_open && !popup->open && hot == anchor) *wait_for_leave = true;
    fpas_own(ui, key, key ^ 1u, key ^ 0x4E4156u, key);
    *owner = key;
    if (popup->open && draw) *node = cui__current(ui);
    if (!popup->open) fpas_return_popup_focus(ui, key, anchor);
    return draw;
}

void fpas_cui_hover_card_end(cui_ctx *ui, int base) {
    const cui_id key = ui->current_layer;
    const cui_popup *popup = cui__popup_get(ui, key, false);
    const cui_id anchor = popup ? popup->anchor : 0;
    cui_hover_card_end(ui);
    fpas_popup_inherit_focus(ui, key);
    ui->last_node = base;
    ui->last_id = anchor;
}
