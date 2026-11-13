#include "../fpas_cui.h"

int fpas_cui_node_editor_begin(cui_ctx *ui, const char *id, cui_node_view view,
    int width_kind, float width, int height_kind, float height, float minimum, float maximum, int flags) {
    cui_node_view *current = cui__frame_alloc(ui, sizeof *current);
    *current = view;
    const cui_node_editor_opts options = { .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height),
        .min_zoom = minimum, .max_zoom = maximum, .no_grid = (flags & 2) != 0, .minimap = (flags & 4) != 0 };
    cui_push_disabled(ui, !(flags & 1));
    const bool interactive = !cui_is_disabled(ui);
    const cui_id key = cui_make_id(ui, id), scroll = ui->scroll_target;
    const cui_id hot = ui->hot, focus = ui->focus;
    if (!interactive) {
        /* CUI-037 (docs/future/cui-upstream-bugs.md#cui-037): the editor handles wheel and gestures outside cui__interact. */
        if (scroll == cui__ne_scroll_id(key)) ui->scroll_target = 0;
        ui->hot = ui->focus = 0;
        cui__ne_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->mode = NE_IDLE;
    }
    const bool began = cui_node_editor_begin_ex(ui, id, current, &options);
    if (!interactive) {
        ui->scroll_target = scroll;
        ui->hot = hot;
        ui->focus = focus;
    }
    if (began) return 1;
    cui_pop_disabled(ui);
    return 0;
}

void fpas_cui_node_editor_end(cui_ctx *ui, cui_node_view *result) {
    const cui__ne_build *editor = cui__ne_cur(ui);
    if (editor) *result = *editor->view;
    cui_node_editor_end(ui);
    cui_pop_disabled(ui);
}

void fpas_cui_node_begin(cui_ctx *ui, int id, float *x, float *y, const char *title, float width, const char *icon) {
    const cui_node_opts options = { .width = width, .icon = icon ? cui__frame_strdup(ui, icon, -1) : NULL };
    cui_node_begin_ex(ui, id, x, y, title ? cui__frame_strdup(ui, title, -1) : NULL, &options);
}

void fpas_cui_node_pin(cui_ctx *ui, int id, const char *label, int output, int type) {
    if (label) label = cui__frame_strdup(ui, label, -1);
    const cui_pin_opts options = { .type = type };
    if (output) cui_node_output_ex(ui, id, label, &options);
    else cui_node_input_ex(ui, id, label, &options);
}

int fpas_cui_node_event(cui_ctx *ui, int kind, int *first, int *second) {
    if (kind == 0) return cui_node_link_created(ui, first, second);
    if (kind == 1) return cui_node_link_removed(ui, first);
    return cui_node_deleted(ui, first);
}
