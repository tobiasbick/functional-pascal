#ifndef FPAS_CUI_TICKER_H
#define FPAS_CUI_TICKER_H
typedef struct { const char *label, *value; float delta; int lower_is_better; } fpas_ticker_item;
CUI_API void fpas_cui_ticker(cui_ctx *ui, const char *id, const fpas_ticker_item *items, int count,
    float speed, float height);
#endif
