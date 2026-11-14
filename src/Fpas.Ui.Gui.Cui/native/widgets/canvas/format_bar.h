#ifndef FPAS_CUI_FORMAT_BAR_H
#define FPAS_CUI_FORMAT_BAR_H
CUI_API int fpas_cui_format_bar(cui_ctx *ui, const char *id, cui_text_format *format, const cui_text_style *styles,
    int style_count, const cui_tone *colors, int color_count, const char *footer, const char *keys,
    const char *question, int flags);
#endif
