#ifndef FPAS_CUI_PIPELINE_H
#define FPAS_CUI_PIPELINE_H
CUI_API void fpas_cui_pipeline(cui_ctx *ui, const char *id, const char *const * stages, int stages_count, const cui_pipeline_job * jobs, int jobs_count, int selection, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
