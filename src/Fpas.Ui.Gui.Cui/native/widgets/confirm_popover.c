#include "../fpas_cui.h"

int fpas_cui_confirm_popover(cui_ctx *ui, const char *id, const char *title,
    const char *description, const char *confirm, const char *cancel, int placement,
    int danger, int open, int enabled, int *nodes, unsigned int *owner) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id anchor = ui->last_id;
    const int base = ui->last_node;
    const bool requested = open && enabled && anchor;
    for (int i = 0; i < 3; ++i) nodes[i] = -1;
    cui_popup *popup = fpas_prepare_popover(ui, key, requested, owner);
    if (!popup) return 0;

    /* Stable button IDs keep literal captions independent of CUI's label syntax. */
    const char *cancel_slot = "Cancel###cancel";
    const char *confirm_slot = "Confirm###confirm";
    const cui_confirm_popover_opts options = {
        .title = title, .description = description, .confirm_label = confirm_slot, .cancel_label = cancel_slot,
        .danger = danger != 0, .placement = (cui_placement)placement
    };
    cui_push_id_int(ui, (int)key);
    const cui_id cancel_key = cui_make_id(ui, cancel_slot);
    const cui_id confirm_key = cui_make_id(ui, confirm_slot);
    cui_pop_id(ui);

    const bool confirmed = cui_confirm_popover_ex(ui, id, &options);
    fpas_popup_inherit_focus(ui, key);
    const int cancel_node = fpas_find(ui, cancel_key);
    const int confirm_node = fpas_find(ui, confirm_key);
    if (cancel_node >= 0) fpas_literal(ui, cancel_node, cancel, CUI_C(ui, FOREGROUND));
    if (confirm_node >= 0) fpas_literal(ui, confirm_node, confirm, ui->nodes[confirm_node].text_color);

    if (popup->open) {
        nodes[0] = fpas_find(ui, key);
        nodes[1] = cancel_node;
        nodes[2] = confirm_node;
    } else if (enabled) {
        fpas_return_popup_focus(ui, key, anchor);
    }
    ui->last_node = base;
    ui->last_id = anchor;
    return confirmed ? 1 : requested && !popup->open ? 2 : popup->open ? 4 : 0;
}
