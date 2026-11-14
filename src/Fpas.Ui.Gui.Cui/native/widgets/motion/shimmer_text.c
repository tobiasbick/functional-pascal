#include "../../fpas_cui.h"

void fpas_cui_shimmer_text(cui_ctx *ui, const char *id, const char *text, const fpas_motion_text *o) {
    const int node = ui->node_count;
    const cui_text_fx_opts options = fpas_motion_options(o);
    cui_shimmer_text_ex(ui, text, &options);
    fpas_motion_node(ui, node, cui_make_id(ui, id));
}
