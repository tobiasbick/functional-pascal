#include "../../fpas_cui.h"

void fpas_cui_ticker(cui_ctx *ui, const char *id, const fpas_ticker_item *items, int count,
    float speed, float height) {
    const int first = ui->node_count;
    cui_ticker_item *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_ticker_item *item = &items[i];
        native[i] = (cui_ticker_item){ item->label, item->value, item->delta, item->lower_is_better != 0 };
    }
    const cui_ticker_opts options = { .speed = speed, .height = height };
    cui_ticker_ex(ui, id, native, count, &options);
    fpas_creative_node(ui, first, cui_make_id(ui, id));
}
