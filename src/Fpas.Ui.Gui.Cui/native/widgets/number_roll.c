#include "../fpas_cui.h"

static bool fpas_roll_fits(const char *text) {
    const size_t length = strlen(text);
    if (length >= sizeof ((cui__nr_state *)0)->shown) return false;
    size_t first = 0, end = length;
    while (first < length && (text[first] < '0' || text[first] > '9')) ++first;
    while (end > first && (text[end - 1] < '0' || text[end - 1] > '9')) --end;
    if (first > 0 && text[first - 1] == '-') --first;
    if (first >= sizeof ((cui__nr_state *)0)->prefix ||
        length - end >= sizeof ((cui__nr_state *)0)->suffix) return false;
    int slots = 0;
    for (size_t i = first; i < end; ++i)
        if (((unsigned char)text[i] & 0xC0u) != 0x80u) ++slots;
    return slots <= CUI__NR_MAX;
}

void fpas_cui_number_roll(cui_ctx *ui, const char *id, const char *text, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_text_fx_opts options = fpas_motion_options(o);
    /* CUI-030 (docs/future/cui-upstream-bugs.md#cui-030): keep the complete
       literal visible when it exceeds the native rolling buffers. */
    if (fpas_roll_fits(text)) {
        cui_number_roll_ex(ui, id, text, &options);
    } else {
        fpas_forget_state(ui, key);
        cui_text_ex(ui, text, &(cui_text_opts){
            .font = options.font, .size = options.size, .color = options.color,
            .tabular = true, .line_height = 1.3f
        });
    }
    fpas_motion_node(ui, node, key);
}
