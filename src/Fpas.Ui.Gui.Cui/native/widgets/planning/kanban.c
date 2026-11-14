#include "../../fpas_cui.h"

int fpas_cui_kanban_begin(cui_ctx *ui, const char *id, int enabled) {
    const cui_id key = cui_make_id(ui, id);
    cui_push_disabled(ui, !enabled);
    if (cui_is_disabled(ui)) {
        cui__kb_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) { state->drag_card = -1; state->pending = state->kbd = state->kdrop = false; }
    }
    cui_kanban_begin(ui, id);
    return cui__current(ui);
}

int fpas_cui_kanban_column_begin(cui_ctx *ui, const char *title, int count, int palette, uint32_t color, int limit, float width, int *parts) {
    const cui_kanban_column_opts options = { .dot = fpas_chart_color(palette, color), .limit = limit, .width = width };
    const bool clicked = cui_kanban_column_begin_ex(ui, title, count, &options);
    parts[0] = cui__current(ui);
    parts[1] = ui->last_node;
    return clicked && !cui_is_disabled(ui);
}

int fpas_cui_kanban_card_begin(cui_ctx *ui, int key, int focus, int *node) {
    const struct cui_kanban_build *board = ui->building.kanban;
    if (focus && board && !cui_is_disabled(ui)) ui->focus_next_request = fpas_well_part(board->id ^ 0xCA2Du, key);
    const bool clicked = cui_kanban_card_begin(ui, key);
    *node = cui__current(ui);
    return clicked && !cui_is_disabled(ui);
}

void fpas_cui_kanban_card_end(cui_ctx *ui) { cui_kanban_card_end(ui); }
void fpas_cui_kanban_column_end(cui_ctx *ui) { cui_kanban_column_end(ui); }

int fpas_cui_kanban_end(cui_ctx *ui, int root, cui_kanban_move *move) {
    const bool enabled = !cui_is_disabled(ui);
    const bool changed = cui_kanban_end(ui, move);
    cui_pop_disabled(ui);
    const cui_id key = ui->nodes[root].id;
    fpas_own(ui, key, key ^ 0x5C2u, 0, 0);
    fpas_creative_node(ui, root, key);
    return enabled && changed;
}
