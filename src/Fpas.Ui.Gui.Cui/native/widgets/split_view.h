#ifndef FPAS_CUI_SPLIT_VIEW_H
#define FPAS_CUI_SPLIT_VIEW_H

CUI_API float fpas_cui_split_view_begin(cui_ctx *, const char *, float, int, float, float, int, float, int, float, int);
CUI_API void fpas_cui_split_view_next(cui_ctx *, int);
CUI_API void fpas_cui_split_view_end(cui_ctx *);

#endif
