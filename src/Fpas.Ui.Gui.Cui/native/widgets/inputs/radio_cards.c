#include "../../fpas_cui.h"

int fpas_cui_radio_cards(cui_ctx *ui, const char *id, const char *const *titles,
    const char *const *descriptions, const char *const *icons, const char *const *badges,
    int count, int selected, int columns, int enabled, int focus, int *nodes) {
    const cui_id group = cui_make_id(ui, id);
    cui_choice *choices = cui__frame_alloc(ui, (size_t)count * sizeof *choices);
    for (int i = 0; i < count; ++i) {
        choices[i] = (cui_choice){
            .title = titles[i],
            .description = descriptions[i][0] ? descriptions[i] : NULL,
            .icon = icons[i][0] ? icons[i] : NULL,
            .badge = badges[i][0] ? badges[i] : NULL
        };
    }
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) {
        const int index = focus - 1;
        ui->focus_next_request = cui__hash(&index, sizeof index, group);
    }
    int choice = selected;
    const cui_radio_cards_opts options = { .columns = columns };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_radio_cards_ex(ui, id, choices, count, &choice, &options);
    cui_pop_disabled(ui);
    nodes[0] = fpas_find(ui, group);
    for (int i = 0; i < count; ++i)
        nodes[i + 1] = fpas_find(ui, cui__hash(&i, sizeof i, group));
    if (!enabled) ui->nodes[nodes[0]].opacity = CUI__DISABLED;
    ui->last_node = nodes[0];
    ui->last_id = group;
    return allowed && changed ? choice : -1;
}
