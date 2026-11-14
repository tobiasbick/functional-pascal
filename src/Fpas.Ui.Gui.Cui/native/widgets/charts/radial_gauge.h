#ifndef FPAS_CUI_RADIAL_GAUGE_H
#define FPAS_CUI_RADIAL_GAUGE_H
#include "more_charts.h"
/* Native rendering with call-scoped input and host-owned identity. */
CUI_API int fpas_cui_radial_gauge(cui_ctx *ui, const char *id, float value, float minimum, float maximum,
    const fpas_radial_options *options);
#endif
