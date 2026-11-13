#ifndef FPAS_CUI_COMMAND_PALETTE_H
#define FPAS_CUI_COMMAND_PALETTE_H

CUI_API int fpas_cui_command_palette(cui_ctx *ui, const char *id, int *open, int enabled,
    const char *const *titles, const char *const *icons, const char *const *groups,
    const char *const *shortcuts, int count, cui_id *owner, int *node, int *query, int *rows);
CUI_API int fpas_cui_palette_shortcut(cui_ctx *ui);

#endif
