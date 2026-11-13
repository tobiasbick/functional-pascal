#ifndef FPAS_CUI_COMPOSER_H
#define FPAS_CUI_COMPOSER_H
CUI_API int fpas_cui_composer(cui_ctx *ui, const char *id, char *draft, size_t capacity, const char *placeholder,
    const char *const *attachments, int attachment_count, const cui_model *models, int model_count, int *selected,
    const float *levels, int level_count, float seconds, int max_lines, int width_kind, float width,
    int flags, int focus, int *removed, int *nodes);
#endif
