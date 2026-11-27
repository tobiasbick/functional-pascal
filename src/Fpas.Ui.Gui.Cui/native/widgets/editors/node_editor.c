// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/node-editor.md)
#include "../../fpas_cui.h"

int fpas_cui_node_editor_begin(cui_ctx *ui, const char *id, cui_node_view view,
    int width_kind, float width, int height_kind, float height, float minimum, float maximum, int flags) {
    cui_node_view *current = cui__frame_alloc(ui, sizeof *current);
    *current = view;
    const cui_node_editor_opts options = { .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height),
        .min_zoom = minimum, .max_zoom = maximum, .no_grid = (flags & 2) != 0, .minimap = (flags & 4) != 0 };
    cui_push_disabled(ui, !(flags & 1));
    /* CUI-100: apply movement queued with the pan release before CUI ends the gesture. */
    if (!cui_is_disabled(ui) && !cui__ne_cur(ui) && !ui->mdown[CUI_MOUSE_MIDDLE] && !ui->mdown[CUI_MOUSE_LEFT]) {
        const bool middle = ui->mreleased[CUI_MOUSE_MIDDLE];
        if (middle || ui->mreleased[CUI_MOUSE_LEFT]) {
            const cui_id key = cui_make_id(ui, id ? id : "node-editor");
            const cui__ne_state *state = cui__state_lazy(ui, key, sizeof *state, false);
            if (state && state->mode == NE_PAN) {
                current->x += (middle ? ui->mx : ui->release_x) - state->last_x;
                current->y += (middle ? ui->my : ui->release_y) - state->last_y;
            }
        }
    }
    const bool began = cui_node_editor_begin_ex(ui, id, current, &options);
    if (began) return 1;
    cui_pop_disabled(ui);
    return 0;
}

void fpas_cui_node_editor_end(cui_ctx *ui, cui_node_view *result) {
    const cui__ne_build *editor = cui__ne_cur(ui);
    if (editor) {
        *result = *editor->view;
        /* CUI-170: a later node can change the selection after an earlier card is built. */
        const cui__ne_state *state = editor->st;
        const cui__ne_set *current = &state->set[state->cur];
        const int count = CUI_MIN(current->node_n, editor->card_n);
        for (int i = 0; i < count; ++i) {
            cui__ne_card *card = ui->nodes[editor->cards[i]].paint_data;
            const bool selected = cui__ne_has(&state->sel, current->nodes[i].id);
            card->selected = editor->raise[i] = selected;
            card->lift = selected && state->lift.x > .01f ? state->lift.x : 0;
        }
    }
    cui_node_editor_end(ui);
    cui_pop_disabled(ui);
}

int fpas_cui_node_begin(cui_ctx *ui, int id, float *x, float *y, const char *title, float width, const char *icon) {
    const cui_node_opts options = { .width = width, .icon = icon ? cui__frame_strdup(ui, icon, -1) : NULL };
    return cui_node_begin_ex(ui, id, x, y, title ? cui__frame_strdup(ui, title, -1) : NULL, &options);
}

int fpas_cui_node_pin(cui_ctx *ui, int id, const char *label, int output, int type) {
    if (label) label = cui__frame_strdup(ui, label, -1);
    const cui_pin_opts options = { .type = type };
    return output ? cui_node_output_ex(ui, id, label, &options) : cui_node_input_ex(ui, id, label, &options);
}

int fpas_cui_node_selected(cui_ctx *ui, int id) {
    const cui__ne_build *editor = cui__ne_cur(ui);
    return editor && cui__ne_has(&editor->st->sel, id);
}

int fpas_cui_node_event(cui_ctx *ui, int kind, int *first, int *second) {
    if (kind == 0) return cui_node_link_created(ui, first, second);
    if (kind == 1) return cui_node_link_removed(ui, first);
    return cui_node_deleted(ui, first);
}
