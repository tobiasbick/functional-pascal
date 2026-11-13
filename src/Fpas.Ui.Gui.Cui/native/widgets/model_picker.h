#ifndef FPAS_CUI_MODEL_PICKER_H
#define FPAS_CUI_MODEL_PICKER_H
CUI_API int fpas_cui_model_picker(cui_ctx *ui, const char *id, const cui_model *models, int count,
    int *selected, int placement, int flags);
#endif
