#ifndef FPAS_CUI_FEATURE_FLAGS_H
#define FPAS_CUI_FEATURE_FLAGS_H
CUI_API void fpas_cui_feature_flags(cui_ctx *ui, const char *id, cui_feature_flag *flags, int count, const char *const *environments, int environment_count, float height, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
