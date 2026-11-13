#ifndef FPAS_CUI_CONTEXT_METER_H
#define FPAS_CUI_CONTEXT_METER_H
CUI_API void fpas_cui_context_meter(cui_ctx *ui, const char *id, float used, float limit,
    const fpas_viz_item *parts, int count, const char *unit, int flags);
#endif
