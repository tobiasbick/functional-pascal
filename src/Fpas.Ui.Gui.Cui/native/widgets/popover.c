#include "../fpas_cui.h"

static cui_popup *fpas_prepare_popover(cui_ctx *ui, cui_id key, bool open, unsigned int *owner) {
    cui_popup *popup = cui__popup_get(ui, key, open);
    *owner = popup ? key : 0;
    if (popup) {
        fpas_own(ui, key, key ^ 0x4E4156u, 0, key);
        if (open != popup->open) cui__popup_set(ui, key, open);
        popup->anchor = ui->last_id;
    }
    return popup;
}

static void fpas_return_popup_focus(cui_ctx *ui, cui_id popup, cui_id anchor) {
    for (int i = 0; i < ui->prev_hit_count; ++i) {
        if (ui->prev_hits[i].layer == popup && ui->prev_hits[i].id == ui->focus) {
            ui->focus = anchor;
            return;
        }
    }
}

static void fpas_popup_inherit_focus(cui_ctx *ui, cui_id popup) {
    /* A floating panel inside a modal still belongs to its parent's Tab order. */
    if (!ui->current_layer) return;
    for (int i = 0; i < ui->focus_count; ++i) {
        if (ui->focus_layers[i] == popup) ui->focus_layers[i] = ui->current_layer;
    }
}

int fpas_cui_popover_begin(cui_ctx *ui, const char *id, int placement,
    float width, float padding, float gap, int open, int enabled, int *node, unsigned int *owner) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id anchor = ui->last_id;
    const bool requested = open && anchor;
    *node = -1;
    cui_popup *popup = fpas_prepare_popover(ui, key, requested, owner);
    if (!popup) return 0;
    const cui_popover_opts options = {
        .placement = (cui_placement)placement, .width = width, .pad = padding, .gap = gap
    };
    const uint8_t escape = ui->kpressed[CUI_KEY_ESCAPE];
    const bool left = ui->mpressed[CUI_MOUSE_LEFT], right = ui->mpressed[CUI_MOUSE_RIGHT];
    if (!enabled) ui->kpressed[CUI_KEY_ESCAPE] = ui->mpressed[CUI_MOUSE_LEFT] = ui->mpressed[CUI_MOUSE_RIGHT] = false;
    const bool draw = cui_popover_begin(ui, id, &options);
    if (!enabled) {
        ui->kpressed[CUI_KEY_ESCAPE] = escape;
        ui->mpressed[CUI_MOUSE_LEFT] = left;
        ui->mpressed[CUI_MOUSE_RIGHT] = right;
    }
    if (popup->open && draw) *node = cui__current(ui);
    if (!popup->open && enabled) fpas_return_popup_focus(ui, key, anchor);
    return (draw ? 1 : 0) | (enabled && requested && !popup->open ? 2 : 0);
}

void fpas_cui_popover_end(cui_ctx *ui, int base) {
    const cui_id popup = ui->current_layer;
    cui_popover_end(ui);
    fpas_popup_inherit_focus(ui, popup);
    ui->last_node = base;
    ui->last_id = base >= 0 ? ui->nodes[base].id : 0;
}
