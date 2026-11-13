#ifndef FPAS_CUI_SCHEDULE_PICKER_H
#define FPAS_CUI_SCHEDULE_PICKER_H
CUI_API void fpas_cui_schedule_picker(cui_ctx *ui, const char *id, char *value, size_t capacity, int64_t now, int utc, int next_count, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
