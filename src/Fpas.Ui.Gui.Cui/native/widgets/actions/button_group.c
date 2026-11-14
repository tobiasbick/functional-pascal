#include "../../fpas_cui.h"

int fpas_cui_button_group_begin(cui_ctx *ui) {
    cui_button_group_begin(ui);
    return cui__current(ui);
}

void fpas_cui_button_group_end(cui_ctx *ui, int node) {
    cui_button_group_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}
