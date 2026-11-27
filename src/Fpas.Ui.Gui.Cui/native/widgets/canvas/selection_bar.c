// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/selection-bar.md)
#include "../../fpas_cui.h"

/** Draws a selection bar and reports clear or action proposals while it owns input. */
int fpas_cui_selection_bar(cui_ctx *ui, const char *id, bool *selected, int item_count, int count,
    const cui_selection_action *actions, int action_count, int enabled, int no_escape, const char *count_format,
    const char *anchor_name) {
    const int anchor = ui->last_node;
    const cui_id previous = ui->last_id;
    const cui_id key = cui_make_id(ui, id);
    /* Generic FPAS containers can be anonymous. Assign an ID only when the anchor needs one. */
    if (!anchor_name && anchor >= 0 && !ui->nodes[anchor].id) ui->nodes[anchor].id = key ^ 0xBA4A11u;
    cui_id anchor_id = anchor >= 0 ? ui->nodes[anchor].id : 0;
    if (anchor_name) anchor_id = cui_make_id(ui, anchor_name);
    const bool accepts_input = enabled && !cui_is_disabled(ui);
    const cui_selection_bar_opts options = { .selected = selected, .item_count = item_count,
        .anchor = anchor_id, .no_escape = no_escape || !accepts_input, .count_format = count_format };
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): leave activation keys to the open popup. */
    const bool covered = accepts_input && cui__keys_covered(ui);
    const uint8_t enter = ui->kpressed[CUI_KEY_ENTER], space = ui->kpressed[CUI_KEY_SPACE];
    if (covered) ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    cui_push_disabled(ui, !accepts_input);
    const int event = cui_selection_bar_ex(ui, id, count, actions, action_count, &options);
    cui_pop_disabled(ui);
    if (covered) {
        ui->kpressed[CUI_KEY_ENTER] = enter;
        ui->kpressed[CUI_KEY_SPACE] = space;
    }
    ui->last_node = anchor;
    ui->last_id = previous;
    return accepts_input ? event : CUI_SELECTION_NONE;
}
