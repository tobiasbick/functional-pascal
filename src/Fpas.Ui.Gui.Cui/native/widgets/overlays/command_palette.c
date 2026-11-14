#include "../../fpas_cui.h"

int fpas_cui_command_palette(cui_ctx *ui, const char *id, int *open, int enabled,
    const char *const *titles, const char *const *icons, const char *const *groups,
    const char *const *shortcuts, int count, cui_id *owner, int *node, int *query, int *rows) {
    *node = *query = -1;
    const int previous_node = ui->last_node;
    const cui_id previous_id = ui->last_id;
    cui_push_id(ui, id);
    const cui_id key = cui_make_id(ui, "#command-palette");
    *owner = key;
    cui_push_id(ui, "#palette");
    const cui_id query_key = cui_make_id(ui, "#query");
    const cui_id list_key = cui_make_id(ui, "#list");
    cui_pop_id(ui);
    fpas_own(ui, key, query_key, list_key, 0);
    cui_command *commands = cui__frame_alloc(ui, (size_t)(count ? count : 1) * sizeof *commands);
    for (int index = 0; index < count; ++index) {
        commands[index] = (cui_command){
            .title = titles[index], .icon = *icons[index] ? icons[index] : NULL,
            .group = *groups[index] ? groups[index] : NULL,
            .shortcut = *shortcuts[index] ? shortcuts[index] : NULL
        };
    }
    const bool was_open = *open != 0;
    bool visible = was_open;
    const bool interactive = enabled && was_open && !cui_is_disabled(ui);
    const cui_key keys[] = { CUI_KEY_ESCAPE, CUI_KEY_ENTER, CUI_KEY_UP, CUI_KEY_DOWN };
    uint8_t presses[sizeof keys / sizeof *keys];
    const cui_id focus = ui->focus;
    const cui_id modal_layer = ui->modal_layer;
    const int modal_z = ui->modal_z;
    /* CUI-024 (docs/future/cui-upstream-bugs.md#cui-024): the exit animation still handles keys. */
    if (!interactive) {
        for (size_t index = 0; index < sizeof presses; ++index) {
            presses[index] = ui->kpressed[keys[index]];
            ui->kpressed[keys[index]] = 0;
        }
    }
    cui_push_disabled(ui, !interactive);
    const int selected = cui_command_palette(ui, &visible, commands, count);
    cui_pop_disabled(ui);
    if (!interactive) {
        for (size_t index = 0; index < sizeof presses; ++index)
            ui->kpressed[keys[index]] = presses[index];
        ui->focus = focus;
        ui->modal_layer = modal_layer;
        ui->modal_z = modal_z;
    }
    if (was_open && rows) {
        *node = fpas_find(ui, key ^ 0x9A4Eu);
        *query = fpas_find(ui, query_key);
        for (int index = 0; index < count; ++index)
            rows[index] = fpas_find(ui, cui__hash(&index, sizeof index, key));
    }
    cui_pop_id(ui);
    ui->last_node = previous_node;
    ui->last_id = previous_id;
    if (interactive) *open = visible;
    return interactive ? selected : -1;
}

int fpas_cui_palette_shortcut(cui_ctx *ui) {
    return !cui_wants_keyboard(ui) && cui_shortcut(ui, CUI_MOD_PRIMARY, CUI_KEY_K);
}
