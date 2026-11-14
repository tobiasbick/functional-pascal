#include "../../fpas_cui.h"

int fpas_cui_format_bar(cui_ctx *ui, const char *id, cui_text_format *format, const cui_text_style *styles,
    int style_count, const cui_tone *colors, int color_count, const char *footer, const char *keys,
    const char *question, int flags) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0xF0F0u, 0, key ^ 0x0F10Fu);
    const cui_format_bar_opts options = { .styles = styles, .style_count = style_count,
        .colors = colors, .color_count = color_count, .footer = footer, .footer_keys = keys,
        .delete_question = question, .no_delete = (flags & 2) != 0, .no_more = (flags & 4) != 0 };
    cui_push_disabled(ui, !(flags & 1));
    int event = cui_format_bar_ex(ui, id, format, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, key);
    return flags & 1 ? event : 0;
}
