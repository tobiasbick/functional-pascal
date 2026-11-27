// [Documentation](../../../../../docs/ui/gui/disclosure/hover-card.md)
#include "../../fpas_cui.h"

/** Opens the native hover card and resets its state when the trigger cannot accept input. */
int fpas_cui_hover_card_begin(cui_ctx *ui, const char *id, int placement,
    float width, float delay, int enabled, int *node, unsigned int *owner) {
    *node = -1;
    *owner = 0;
    const cui_id anchor = ui->last_id;
    const cui_id key = cui_make_id(ui, id) ^ 0x4C4Du;
    if (!enabled || !anchor || cui_is_disabled(ui)) {
        fpas_cui_forget(ui, key);
        return 0;
    }
    const cui_hover_card_opts options = {
        .placement = (cui_placement)placement, .width = width, .delay = delay
    };
    const float wakeup = ui->wakeup;
    const bool draw = cui_hover_card_begin(ui, id, &options);
    /* CUI-051 (docs/future/cui-upstream-bugs.md#cui-051): release wakes the host when
       a held pointer prevents the card from opening after its delay. */
    if (!draw && ui->mdown[CUI_MOUSE_LEFT] && ui->hot == anchor && ui->wakeup == 0)
        ui->wakeup = wakeup;
    const cui_popup *popup = cui__popup_get(ui, key, false);
    if (!popup) return 0;
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
    ui->last_node = base;
    ui->last_id = anchor;
}
