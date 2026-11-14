#include "../../fpas_cui.h"

int fpas_cui_password_strength(cui_ctx *ui, const char *id, const char *password) {
    const cui_id key = cui_make_id(ui, id);
    const int row = ui->node_count;
    cui_push_id(ui, id);
    const int strength = cui_password_strength(ui, password);
    cui_pop_id(ui);
    ui->nodes[row].id = key;
    ui->last_node = row;
    ui->last_id = key;
    return strength;
}
