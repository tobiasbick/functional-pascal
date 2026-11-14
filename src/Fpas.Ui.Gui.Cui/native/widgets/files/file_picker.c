#include "../../fpas_cui.h"

int fpas_cui_file_picker(cui_ctx *ui, const char *id, int open, int enabled, int mode, const char *title,
    const char *start, const char *name, const char *filter, const char *filter_label,
    cui_file_list_fn source, void *user, char *path, int capacity, int *parts) {
    const int first = ui->node_count;
    bool visible = open != 0;
    const cui_id key = cui_make_id(ui, id);
    const char *caption = title ? title : mode == 1 ? "Save" : mode == 2 ? "Choose a folder" : "Open";
    cui_push_id(ui, id);
    const cui_id dialog = cui_make_id(ui, caption) ^ 0xD1A106u;
    cui_push_id_int(ui, (int)dialog);
    const cui_id browser = cui_make_id(ui, "browser");
    const cui_id cancel = cui_make_id(ui, "Cancel");
    const cui_id confirm = cui_make_id(ui, mode == 1 ? "Save" : mode == 2 ? "Choose" : "Open");
    const cui_id field = cui_make_id(ui, "name");
    cui_pop_id(ui);
    cui_pop_id(ui);
    const cui_file_picker_opts options = { .title = title, .save = mode == 1, .folders = mode == 2,
        .start = start, .name = name, .filter = filter, .filter_label = filter_label, .list = source, .list_user = user };
    const uint8_t escape = ui->kpressed[CUI_KEY_ESCAPE];
    if (!enabled) ui->kpressed[CUI_KEY_ESCAPE] = 0;
    cui_push_disabled(ui, !enabled);
    const bool chosen = cui_file_picker_ex(ui, id, &visible, path, (size_t)capacity, &options);
    cui_pop_disabled(ui);
    if (!enabled) ui->kpressed[CUI_KEY_ESCAPE] = escape;
    parts[0] = fpas_find(ui, dialog ^ 0x9A4Eu);
    parts[1] = fpas_find(ui, cancel);
    parts[2] = fpas_find(ui, confirm);
    parts[3] = mode == 1 ? fpas_find(ui, field) : -1;
    parts[4] = fpas_find(ui, browser ^ 0x115Du);
    for (int i = 0; i < 5; ++i) if (parts[i] < first) parts[i] = -1;
    ui->last_node = parts[0];
    ui->last_id = key;
    return enabled && open && !visible ? (chosen ? 2 : 1) : 0;
}
