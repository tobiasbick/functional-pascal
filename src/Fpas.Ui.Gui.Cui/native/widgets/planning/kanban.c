// [Documentation](../../../../../docs/ui/gui/planning/kanban.md)
#include "../../fpas_cui.h"

static int fpas_kanban_gesture_card(const cui__kb_state *state) {
    return state->drag_card >= 0 ? state->drag_card : state->pending ? state->press_card : -1;
}

static void fpas_kanban_cancel(cui_ctx *ui, cui_id key, cui__kb_state *state) {
    const int card = fpas_kanban_gesture_card(state);
    if (ui->active == key || (card >= 0 && ui->active == fpas_well_part(key ^ 0xCA2Du, card)))
        ui->active = 0;
    state->drag_card = -1;
    state->pending = state->kbd = state->kdrop = false;
}

int fpas_cui_kanban_begin(cui_ctx *ui, const char *id, int enabled) {
    const cui_id key = cui_make_id(ui, id);
    cui_push_disabled(ui, !enabled);
    cui__kb_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    if (state) {
        /* [CUI-085](../../../../../docs/future/cui-upstream-bugs.md#cui-085): a lift cannot outlive keyboard focus. */
        const bool lost_focus = state->kbd && state->drag_card >= 0 &&
            ui->focus != fpas_well_part(key ^ 0xCA2Du, state->drag_card);
        if (cui_is_disabled(ui) || lost_focus) fpas_kanban_cancel(ui, key, state);
    }
    cui_kanban_begin(ui, id);
    return cui__current(ui);
}

int fpas_cui_kanban_column_begin(cui_ctx *ui, const char *title, int count, int palette, uint32_t color, int limit, float width, int *parts) {
    const cui_kanban_column_opts options = { .dot = fpas_chart_color(palette, color), .limit = limit, .width = width };
    const bool clicked = cui_kanban_column_begin_ex(ui, title, count, &options);
    parts[0] = cui__current(ui);
    parts[1] = ui->last_node;
    if (strstr(title, "##")) fpas_text(ui, fpas_label_node(ui, parts[0]), title);
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
    const cui_id key = ui->nodes[root].id;
    const struct cui_kanban_build *board = ui->building.kanban;
    cui__kb_state *state = board->st;
    const int card = fpas_kanban_gesture_card(state);
    /* [CUI-082](../../../../../docs/future/cui-upstream-bugs.md#cui-082): cancel when the source changes. */
    if (card >= 0) {
        const int column = state->drag_card >= 0 ? state->from_col : state->press_col;
        const int node = fpas_find(ui, fpas_well_part(key ^ 0xCA2Du, card));
        const int parent = node > root ? ui->nodes[node].parent : -1;
        if (parent <= root || ui->nodes[parent].parent != root ||
            ui->nodes[parent].id != fpas_well_part(key ^ 0xC01u, column))
            fpas_kanban_cancel(ui, key, state);
    }
    const bool enabled = !cui_is_disabled(ui);
    const bool changed = cui_kanban_end(ui, move);
    cui_pop_disabled(ui);
    fpas_own(ui, key, key ^ 0x5C2u, 0, 0);
    fpas_creative_node(ui, root, key);
    return enabled && changed;
}
