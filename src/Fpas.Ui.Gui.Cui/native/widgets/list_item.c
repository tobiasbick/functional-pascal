#include "../fpas_cui.h"

int fpas_cui_list_item(cui_ctx *ui, const char *id, const char *title, const char *subtitle,
    const char *trailing, const char *avatar, const char *icon, int selected, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id) ^ 0x115E0008u;
    if (focus && enabled && !cui_is_disabled(ui)) ui->focus_next_request = key;
    const cui_list_item_opts options = {
        .subtitle = subtitle, .trailing = trailing, .avatar = avatar, .icon = icon,
        .selected = selected != 0
    };
    const int row = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const bool clicked = cui_list_item_ex(ui, id, &options);
    cui_pop_disabled(ui);
    int column = ui->nodes[row].first;
    if (avatar || icon) column = ui->nodes[column].next;
    fpas_text(ui, ui->nodes[column].first, title);
    ui->last_node = row;
    return clicked;
}
