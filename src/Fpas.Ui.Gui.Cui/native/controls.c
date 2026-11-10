#include "fpas_cui.h"

void fpas_cui_column_begin(cui_ctx *ui, float padding, float gap) {
    const cui_box options = { .pad = padding, .gap = gap, .width = CUI_FILL };
    (void)cui_column_begin(ui, &options);
}

void fpas_cui_row_begin(cui_ctx *ui, float padding, float gap) {
    const cui_box options = { .pad = padding, .gap = gap, .width = CUI_FILL };
    (void)cui_row_begin(ui, &options);
}

void fpas_cui_request_focus(cui_ctx *ui, const char *id) {
    ui->focus_next_request = cui_make_id(ui, id);
}

void fpas_cui_move_pointer(cui_ctx *ui, float x, float y) {
    cui_input_mouse_move(ui, x, y);
    /* The normal frame-start hit test ran before this injected move. */
    cui__compute_hot(ui);
}

void fpas_cui_scroll_begin(cui_ctx *ui, const char *id, float height, int horizontal) {
    const cui_scroll_opts options = {
        .id = id, .width = CUI_FILL, .height = height > 0 ? CUI_PX(height) : CUI_FILL,
        .horizontal = horizontal != 0
    };
    cui_scroll_begin(ui, &options);
}

int fpas_cui_list_item(cui_ctx *ui, const char *label, int selected) {
    const cui_list_item_opts options = { .selected = selected != 0 };
    return cui_list_item_ex(ui, label, &options);
}

int fpas_cui_tab_item(cui_ctx *ui, const char *label, int selected) {
    const cui_button_opts options = { .variant = CUI_BUTTON_GHOST, .selected = selected != 0 };
    return cui_button_ex(ui, label, &options);
}

int fpas_cui_table_begin(cui_ctx *ui, const char *id, const char *const *titles, int count) {
    if (count <= 0) return 0;
    /* Keep descriptors and their strings valid while CUI retains the columns. */
    cui_table_column *columns = cui__frame_alloc(ui, (size_t)count * sizeof *columns);
    for (int index = 0; index < count; ++index)
        columns[index].title = cui__frame_strdup(ui, titles[index], -1);
    const cui_table_opts options = { .selectable = true };
    return cui_table_begin_ex(ui, id, columns, count, &options);
}

int fpas_cui_table_row(cui_ctx *ui, int selected) {
    if (!ui->building.table) return 0;
    int activated = cui_table_row(ui, selected != 0);
    ui->last_id = cui__node(ui, cui__current(ui))->id;
    return activated;
}

int fpas_cui_dialog_begin(cui_ctx *ui, const char *title, int *open) {
    bool visible = *open != 0;
    bool draw = cui_dialog_begin(ui, title, &visible, NULL);
    *open = visible;
    return draw;
}
