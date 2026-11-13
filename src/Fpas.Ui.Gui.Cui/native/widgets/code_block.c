#include "../fpas_cui.h"

void fpas_cui_code_block(cui_ctx *ui, const char *id, const char *code, const char *language,
    const char *title, int first, int last, float height, float size, int flags, int *copy) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_code_opts options = { .lang = language, .title = title,
        .line_numbers = (flags & 1) != 0, .highlight_from = first, .highlight_to = last,
        .max_height = height, .size = size, .no_header = (flags & 2) != 0 };
    cui_push_id(ui, id);
    const cui_id state_key = cui_make_id(ui, "copy") ^ 0xC0B7u;
    cui_push_id(ui, "copy");
    const cui_id copy_key = cui_make_id(ui, "#copy-button");
    cui_pop_id(ui);
    cui_pop_id(ui);
    if ((flags & 12) == 12 && !cui_is_disabled(ui)) ui->focus_next_request = copy_key;
    cui_push_disabled(ui, (flags & 4) == 0);
    cui_code_block_ex(ui, id, code, &options);
    cui_pop_disabled(ui);
    *copy = fpas_find(ui, copy_key);
    fpas_own(ui, key, key ^ 0x5C0Du, state_key, 0);
    ui->last_node = node;
    ui->last_id = key;
}
