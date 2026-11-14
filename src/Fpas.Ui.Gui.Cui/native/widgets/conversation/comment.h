#ifndef FPAS_CUI_COMMENT_H
#define FPAS_CUI_COMMENT_H
CUI_API int fpas_cui_comment(cui_ctx *ui, const char *id, const char *author, const char *text,
    const char *time, int depth, const char *badge, int flags, int focus, int *likes, int *liked, int *parts);
#endif
