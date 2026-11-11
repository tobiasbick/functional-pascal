#include "../fpas_cui.h"

int fpas_cui_action_footer_begin(cui_ctx *ui) {
    cui_dialog_footer_begin(ui);
    return cui__current(ui);
}

void fpas_cui_action_footer_end(cui_ctx *ui, int footer) {
    cui_dialog_footer_end(ui);
    ui->last_node = footer;
    ui->last_id = ui->nodes[footer].id;
}
