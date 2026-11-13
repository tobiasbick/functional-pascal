#ifndef FPAS_CUI_STATUS_TAG_H
#define FPAS_CUI_STATUS_TAG_H
CUI_API int fpas_cui_status_tag(cui_ctx *ui, const char *id,
    const char *label, int status, const char *icon, int tone, int flags, int focus, int *parts);
#endif
