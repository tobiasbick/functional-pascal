// [Documentation](../../../../docs/ui/gui/cui.md#theme-and-resources)
#include "../fpas_cui.h"

CUI_API uint32_t fpas_cui_color(int kind, float a, float b, float c, float d, uint32_t left, uint32_t right) {
    const cui_color value = kind == 0 ? cui_oklch(a,b,c,d) : kind == 1 ?
        cui_mix(fpas_chart_color(-1,left), fpas_chart_color(-1,right), a) : cui_alpha(fpas_chart_color(-1,left), a);
    return (uint32_t)value.r | ((uint32_t)value.g << 8) | ((uint32_t)value.b << 16) | ((uint32_t)value.a << 24);
}
