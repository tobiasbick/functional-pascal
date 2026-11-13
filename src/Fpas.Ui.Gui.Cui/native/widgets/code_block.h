#ifndef FPAS_CUI_CODE_BLOCK_H
#define FPAS_CUI_CODE_BLOCK_H

CUI_API void fpas_cui_code_block(cui_ctx *ui, const char *id, const char *code, const char *language,
    const char *title, int first, int last, float height, float size, int flags, int *copy);

#endif
