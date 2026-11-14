#ifndef FPAS_CUI_DOCKSPACE_H
#define FPAS_CUI_DOCKSPACE_H

CUI_API void fpas_cui_dockspace_begin(cui_ctx *, const char *, const char *, int, int, float, int, float, float, float, int);
CUI_API int fpas_cui_dock_panel_begin(cui_ctx *, const char *, const char *, int, int, int *, int *);
CUI_API void fpas_cui_dock_panel_end(cui_ctx *);
CUI_API void fpas_cui_dockspace_end(cui_ctx *);
CUI_API int fpas_cui_dockspace_load(cui_ctx *, const char *, const char *);
CUI_API size_t fpas_cui_dockspace_save(cui_ctx *, const char *, char *, size_t);

#endif
