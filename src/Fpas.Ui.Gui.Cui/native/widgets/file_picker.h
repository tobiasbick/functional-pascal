#ifndef FPAS_CUI_FILE_PICKER_H
#define FPAS_CUI_FILE_PICKER_H
CUI_API int fpas_cui_file_picker(cui_ctx *ui, const char *id, int open, int enabled, int mode, const char *title,
    const char *start, const char *name, const char *filter, const char *filter_label,
    cui_file_list_fn source, void *user, char *path, int capacity, int *parts);
#endif
