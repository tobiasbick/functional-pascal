#include "../fpas_cui.h"

void fpas_cui_text_reveal(cui_ctx *ui, const char *id, const char *text, int replay, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_motion_replay(ui, key, replay);
    const cui_text_fx_opts options = fpas_motion_options(o);
    cui_text_reveal_ex(ui, id, text, &options);
    fpas_motion_node(ui, node, key);
}
