#include "../../fpas_cui.h"

void fpas_cui_word_rotate(cui_ctx *ui, const char *id, const char *const *words, int count,
    float interval, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const char **copy = cui__frame_alloc(ui, (size_t)(count ? count : 1) * sizeof *copy);
    for (int i = 0; i < count; ++i) copy[i] = cui__frame_strdup(ui, words[i], -1);
    const cui_text_fx_opts options = fpas_motion_options(o);
    cui_word_rotate_ex(ui, id, copy, count, interval, &options);
    fpas_motion_node(ui, node, cui_make_id(ui, id));
}
