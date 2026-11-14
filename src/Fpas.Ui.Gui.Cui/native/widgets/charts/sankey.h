#ifndef FPAS_CUI_SANKEY_H
#define FPAS_CUI_SANKEY_H
#include "more_charts.h"
/* Native rendering with call-scoped input and host-owned identity. */
CUI_API int fpas_cui_sankey(cui_ctx *ui, const char *id, const fpas_sankey_node *nodes, int count,
    const cui_sankey_link *links, int link_count, int width_kind, float width, float height,
    float node_width, const char *format, int enabled, int *node);
#endif
