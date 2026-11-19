// [Documentation](../../../../docs/ui/gui/cui.md#application-shortcuts)
#include "../fpas_cui.h"

int fpas_cui_shortcut_owners(cui_ctx *ui) {
    int owners = cui_wants_keyboard(ui) ? 1 : 0;
    if (ui->modal_layer || ui->prev_modal_layer) owners |= 2;
    for (int index = 0; index < ui->popup_count; index++)
        if (ui->popups[index].open) owners |= 2;
    if (ui->focus) owners |= 4;
    return owners;
}

int fpas_cui_take_shortcut(cui_ctx *ui, int mods, int key) {
    if (!cui_shortcut(ui, mods, (cui_key)key)) return 0;
    ui->kpressed[key] = 0;
    return 1;
}

void fpas_cui_shortcut_keys(cui_ctx *ui, unsigned char *keys, int count) {
    for (int key = 0; key < count; key++) keys[key] = cui_key_down(ui, (cui_key)key) ? 1 : 0;
}
