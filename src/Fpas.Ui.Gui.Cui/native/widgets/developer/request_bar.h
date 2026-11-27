// [Documentation](../../../../../docs/ui/gui/more-developer-tools/http-request-bar.md)
#ifndef FPAS_CUI_REQUEST_BAR_H
#define FPAS_CUI_REQUEST_BAR_H
/** Draws a request bar. Flags enable input (1) and request URL focus (2). */
CUI_API void fpas_cui_request_bar(cui_ctx *ui, const char *id, int method, char *url, size_t capacity, int loading, const char *placeholder, int width_kind, float width, int flags, fpas_dev_result *out);
#endif
