#include "../../fpas_cui.h"

void fpas_cui_selection_ring(cui_ctx *ui, int tone, uint32_t rgba, const char *name, float radius) {
    const cui_selection_ring_opts options = { .name = name, .radius = radius };
    cui_selection_ring_ex(ui, rgba ? fpas_chart_color(-1, rgba) :
        tone ? cui__tone_color(ui, (cui_tone)tone) : (cui_color){0}, &options);
}
