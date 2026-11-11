#include "../fpas_cui.h"

void fpas_cui_tag_input(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, int max_tags, int enabled, int focus, int count, int *nodes) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id editor = key ^ 0x7A6u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    fpas_own(ui, editor, key, 0, 0);
    /* CUI checks Backspace before the draft editor clears disabled focus. */
    if (!allowed && ui->focus == editor) ui->focus = 0;
    if (allowed && focus) ui->focus_next_request = editor;
    if (placeholder) placeholder = cui__frame_strdup(ui, placeholder, (int)strlen(placeholder));

    const cui_tag_opts options = { .label = label, .placeholder = placeholder, .max_tags = max_tags };
    cui_push_disabled(ui, !enabled);
    cui_tag_input_ex(ui, id, value, capacity, &options);
    cui_pop_disabled(ui);

    nodes[0] = fpas_find(ui, editor);
    cui_push_id(ui, id);
    for (int i = 0; i < count; ++i) {
        cui_push_id_int(ui, i);
        nodes[i + 1] = fpas_find(ui, cui_make_id(ui, CUI_ICON_X));
        cui_pop_id(ui);
    }
    cui_pop_id(ui);
    const int box = fpas_find(ui, key);
    const int root = label ? ui->nodes[box].parent : box;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root;
    ui->last_id = key;
}
