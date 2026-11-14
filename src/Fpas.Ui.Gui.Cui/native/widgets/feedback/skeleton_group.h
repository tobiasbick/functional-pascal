#ifndef FPAS_CUI_SKELETON_GROUP_H
#define FPAS_CUI_SKELETON_GROUP_H
CUI_API int fpas_cui_skeleton_group_begin(cui_ctx *ui, const char *id, int width_kind, float width, float gap, int dither);
CUI_API void fpas_cui_skeleton_group_end(cui_ctx *ui, int node);
#endif
