#ifndef FPAS_CUI_PROMPT_SUGGESTIONS_H
#define FPAS_CUI_PROMPT_SUGGESTIONS_H
CUI_API int fpas_cui_prompt_suggestions(cui_ctx *ui, const char *id, const cui_prompt_suggestion *items,
    int count, int columns, int flags, int focus, int *parts);
#endif
