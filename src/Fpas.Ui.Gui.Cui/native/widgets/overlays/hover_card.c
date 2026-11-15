#include "../../fpas_cui.h"

int fpas_cui_hover_card_begin(cui_ctx *ui, const char *id, int placement,
    float width, float delay, int enabled, int *node, unsigned int *owner) {
    *node = -1;
    *owner = 0;
    const cui_id anchor = ui->last_id;
    if (!enabled || !anchor || cui_is_disabled(ui)) return 0;

    const cui_id key = cui_make_id(ui, id) ^ 0x4C4Du;
    const cui_hover_card_opts options = {
        .placement = (cui_placement)placement, .width = width, .delay = delay
    };
    const bool draw = cui_hover_card_begin(ui, id, &options);
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
    fpas_popup_inherit_focus(ui, key);
    ui->last_node = base;
    ui->last_id = anchor;
}
