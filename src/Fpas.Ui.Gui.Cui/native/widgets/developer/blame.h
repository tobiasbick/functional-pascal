#ifndef FPAS_CUI_BLAME_H
#define FPAS_CUI_BLAME_H
CUI_API void fpas_cui_blame(cui_ctx *ui, const char *id, const char *const * lines, int lines_count, const int * line_commits, int line_commits_count, const cui_blame_commit * revisions, int revisions_count, float height, const char * language, int64_t now, int first_line, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
