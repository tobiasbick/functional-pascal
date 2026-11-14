#ifndef FPAS_CUI_TREE_H
#define FPAS_CUI_TREE_H

CUI_API int fpas_cui_tree_begin(cui_ctx *ui, const char *id, const char *label, const char *icon, int leaf, int selected, int enabled, int focus, int *open, int *node);
CUI_API void fpas_cui_tree_end(cui_ctx *ui, int node);

#endif
