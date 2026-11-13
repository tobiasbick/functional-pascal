#include "../fpas_cui.h"

void fpas_cui_typing_indicator(cui_ctx *ui, const char *id, const char *name) {
    const int first = ui->node_count;
    const bool animating = ui->animating;
    cui_typing_indicator(ui, name);
    /* CUI-033 (docs/future/cui-upstream-bugs.md#cui-033): keep the native
       paint at a fixed phase without waking a reduced-motion window. */
    if (ui->theme.motion <= 0) {
        const int bubble = ui->nodes[ui->nodes[first].first].next;
        const int dots = ui->nodes[bubble].first;
        ((cui__dots_paint *)ui->nodes[dots].paint_data)->time = 0;
        ui->animating = animating;
    }
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
