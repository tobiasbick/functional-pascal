#ifndef FPAS_CUI_RESULT_CARD_H
#define FPAS_CUI_RESULT_CARD_H
CUI_API int fpas_cui_result_card(cui_ctx *ui, const char *id, const fpas_media_source *source, const char *title,
    const char *caption, const char *badge, int tone, int width_kind, float width, float aspect, int enabled, int focus);
#endif
