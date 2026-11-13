#ifndef FPAS_CUI_VISUALIZATIONS_H
#define FPAS_CUI_VISUALIZATIONS_H
#include "charts.h"
/* Input representation shared by the native visualization adapters. */
typedef struct { const char *label; float value; int palette; uint32_t rgba; } fpas_viz_item;
#endif
