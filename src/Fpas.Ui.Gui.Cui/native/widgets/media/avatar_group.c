#include "../../fpas_cui.h"

void fpas_cui_avatar_group(cui_ctx *ui, const char *const *names, int count, int maximum, float size) {
    const int node = ui->node_count;
    cui_avatar_group(ui, names, count, maximum, size);
    ui->last_node = node;
}
