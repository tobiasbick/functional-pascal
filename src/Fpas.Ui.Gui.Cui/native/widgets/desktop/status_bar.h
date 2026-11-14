#ifndef FPAS_CUI_STATUS_BAR_H
#define FPAS_CUI_STATUS_BAR_H

CUI_API void fpas_cui_status_bar_begin(cui_ctx *, const char *, int);
CUI_API void fpas_cui_status_bar_end(cui_ctx *);
CUI_API void fpas_cui_status_bar_spacer(cui_ctx *);
CUI_API void fpas_cui_status_bar_text(cui_ctx *, const char *, const char *, int);
CUI_API int fpas_cui_status_bar_item(cui_ctx *, const char *, const char *, int, const char *, int, int, int *);
CUI_API void fpas_cui_status_bar_progress(cui_ctx *, float, const char *, float, int);

#endif
