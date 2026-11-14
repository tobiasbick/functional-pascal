#include "../../fpas_cui.h"

void fpas_cui_uptime(cui_ctx *ui, const char *id, const cui_uptime_service * services, int services_count, int year, int month, int day, int width_kind, float width, int enabled, fpas_dev_result * out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    *out = fpas_dev_empty();
    cui_uptime_service *safe = cui__frame_alloc(ui, sizeof *safe * (size_t)CUI_MAX(1, services_count));
    for (int i = 0; i < services_count; ++i) {
        safe[i] = services[i];
        if (safe[i].day_count <= 0) continue;
        const size_t size = sizeof(float) * (size_t)safe[i].day_count;
        float *days = cui__frame_alloc(ui, size);
        memcpy(days, safe[i].days, size);
        safe[i].days = days;
    }
    services = safe;
    const cui_uptime_opts o = { .width = fpas_extent(width_kind, width), .last_day = ((cui_date){year,month,day}) };
    cui_push_disabled(ui, !enabled);
    cui_uptime_result r = cui_uptime_ex(ui,id,services,services_count,&o);
    cui_pop_disabled(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_creative_node(ui, root, key);
    out->index=r.service;
    out->action=r.day;
}
