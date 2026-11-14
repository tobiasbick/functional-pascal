#include "../../fpas_cui.h"

void fpas_cui_schedule_picker(cui_ctx *ui, const char *id, char *value, size_t capacity, int64_t now, int utc, int next_count, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui,id);
    *out = fpas_dev_empty();
    cui_push_disabled(ui,!enabled);
    const cui_schedule_opts o = { .width=fpas_extent(width_kind,width), .now=now, .utc=utc, .next_count=next_count };
    const cui_schedule_result r = cui_schedule_picker_ex(ui,id,value,capacity,&o);
    out->flags = (r.changed ? 1 : 0) | (r.valid ? 2 : 0);
    out->x = (double)r.next;
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui,key,CUI_FIT,CUI_FIT);
    fpas_creative_node(ui,root,key);
}
