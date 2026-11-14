#ifndef FPAS_CUI_SEED_FIELD_H
#define FPAS_CUI_SEED_FIELD_H
CUI_API int fpas_cui_seed_field(cui_ctx *ui, const char *id,
    const char *label, uint32_t *seed, int *locked, const char *display, const char *random, const char *fixed, int flags, int focus, int *parts);
#endif
