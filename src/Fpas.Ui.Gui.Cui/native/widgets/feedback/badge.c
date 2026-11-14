#include "../../fpas_cui.h"

void fpas_cui_badge(cui_ctx *ui, const char *text, int tone, int dot, int solid, const char *icon, int size) {
    const int node = ui->node_count;
    const cui_badge_opts options = { .tone = (cui_tone)tone, .dot = dot != 0, .solid = solid != 0,
        .icon = icon, .size = (cui_control_size)size };
    cui_badge_ex(ui, text, &options);
    ui->last_node = node;
}
