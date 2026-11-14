#ifndef FPAS_CUI_QUERY_BUILDER_H
#define FPAS_CUI_QUERY_BUILDER_H
CUI_API void fpas_cui_query_builder(cui_ctx *ui, const char *id, const cui_query_field *fields, int field_count, cui_query_node *rules, int *count, int capacity, int max_depth, int read_only, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
