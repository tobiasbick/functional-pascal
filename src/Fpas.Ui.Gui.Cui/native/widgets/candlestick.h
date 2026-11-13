#ifndef FPAS_CUI_CANDLESTICK_H
#define FPAS_CUI_CANDLESTICK_H
#include "more_charts.h"
/* Native rendering with call-scoped input and host-owned identity. */
CUI_API int fpas_cui_candlestick(cui_ctx *ui, const char *id, const cui_candle *candles, int count,
    const fpas_candle_options *options, int enabled, int focus, int *node);
#endif
