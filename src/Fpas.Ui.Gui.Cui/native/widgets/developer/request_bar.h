#ifndef FPAS_CUI_REQUEST_BAR_H
#define FPAS_CUI_REQUEST_BAR_H
CUI_API void fpas_cui_request_bar(cui_ctx *ui, const char *id, int method, char *url, size_t capacity, int loading, const char *placeholder, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
