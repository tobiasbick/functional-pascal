// [Documentation](../../../../../docs/ui/gui/smart-inputs/money-field.md)
#include "../../fpas_cui.h"

int fpas_cui_money_field(cui_ctx *ui, const char *id, char *text, size_t capacity,
    const char *label, const char *symbol, const char *code, const char *hint, const char *error,
    int width_kind, float width, int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && ui->focus == key) ui->focus = 0;
    const cui_input_opts options = { .label=label, .prefix=symbol, .suffix=code,
        .hint=hint, .error=error, .placeholder="0", .width=fpas_extent(width_kind,width) };
    int nodes[3], revealed;
    const int flags = fpas_draw_text_editor(ui,id,text,capacity,options,false,allowed,focus,
        nodes,&revealed,cui_input_ex);
    *node = nodes[0];
    return flags | (allowed && ui->focus == key ? 4 : 0);
}
