#include "../../fpas_cui.h"

void fpas_cui_stepper(cui_ctx *ui, const char *const *steps, int count, int current) {
    const int row = ui->node_count;
    cui_stepper(ui, steps, count, current);
    ui->last_node = row;
    ui->last_id = ui->nodes[row].id;
}
