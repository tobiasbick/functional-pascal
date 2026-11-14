#include "../../fpas_cui.h"

void fpas_cui_chat_divider(cui_ctx *ui, const char *id, const char *label) {
    const int first = ui->node_count;
    cui_chat_divider(ui, label);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
