#include "../../fpas_cui.h"

int fpas_cui_gauge(cui_ctx *ui, const char *id, const char *label, float value, float size) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    cui_gauge(ui, id, value, size);
    cui__gauge_paint *paint = ui->nodes[node].paint_data;
    paint->label = cui__frame_strdup(ui, label, -1);
    ui->nodes[node].id = key;
    ui->last_node = node; ui->last_id = key;
    return node;
}
