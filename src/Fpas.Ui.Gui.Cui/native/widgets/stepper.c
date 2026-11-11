#include "../fpas_cui.h"

void fpas_cui_stepper(cui_ctx *ui, const char *id, const char *const *steps,
    int count, int current) {
    const int row = ui->node_count;
    cui_push_id(ui, id);
    cui_stepper(ui, steps, count, current);
    cui_pop_id(ui);
    ui->last_node = row;
    ui->last_id = ui->nodes[row].id;
}
