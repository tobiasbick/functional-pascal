#ifndef FPAS_CUI_UPTIME_H
#define FPAS_CUI_UPTIME_H
CUI_API void fpas_cui_uptime(cui_ctx *ui, const char *id, const cui_uptime_service * services, int services_count, int year, int month, int day, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
