#ifndef FPAS_CUI_RESPONSE_STATUS_H
#define FPAS_CUI_RESPONSE_STATUS_H
CUI_API void fpas_cui_response_status(cui_ctx *ui, const char *id, int status, double seconds, double bytes, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
