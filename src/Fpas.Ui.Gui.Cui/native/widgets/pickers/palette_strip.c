// [Documentation](../../../../../docs/ui/gui/creative-controls/palette-strip.md)
#include "../../fpas_cui.h"

/** Draws the native swatch with the final keyboard focus for this frame. */
static void fpas_paint_palette_swatch(cui_ctx *ui, cui_node *node, void *data) {
    cui__pal_paint *paint = data;
    paint->focus = cui_focus_visible(ui, node->id) ? 1.0f : 0.0f;
    cui__paint_swatch(ui, node, data);
}

/** Draws controlled swatches and writes lock or shuffle proposals. */
int fpas_cui_palette_strip(cui_ctx *ui, const char *id,
    const char *label, uint32_t *colors, int *locked, int count, uint32_t *seed, const char *display, float height, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool locking = locked && !(flags & 4);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label), row = key ^ 0x9A11u;
    const cui_id shuffle = key ^ CUI__WELL_TOOL_SALT;
    cui_color swatches[CUI__PAL_MAX];
    bool locks[CUI__PAL_MAX];
    for (int i = 0; i < count; ++i) {
        swatches[i] = fpas_chart_color(-1, colors[i]);
        locks[i] = locking && locked[i] != 0;
    }
    const cui_palette_strip_opts options = { .seed = seed, .value = display,
        .no_shuffle = (flags & 2) != 0, .height = height, .plain = (flags & 8) != 0 };
    /* CUI-049 (docs/future/cui-upstream-bugs.md#cui-049): release hidden swatches and Shuffle. */
    if (options.no_shuffle) fpas_clear_interaction(ui, shuffle);
    for (int i = count; i < CUI__PAL_MAX; ++i) fpas_clear_interaction(ui, fpas_well_part(row, i));
    fpas_well_focus(ui, enabled, row, key, count, !options.no_shuffle, focus);
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): block covered activation.
     * CUI-131 (docs/future/cui-upstream-bugs.md#cui-131): block covered arrow movement. */
    const bool covered = enabled && cui__keys_covered(ui);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    cui_push_disabled(ui, !enabled);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){0});
    const int result = cui_palette_strip_ex(ui, label, swatches, locking ? locks : NULL, count, &options);
    if (covered) fpas_well_set_keys(ui, keys);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) {
        parts[i] = fpas_find(ui, fpas_well_part(row, i));
        /* CUI-157 (docs/future/cui-upstream-bugs.md#cui-157): the arrow can move focus after the input pass. */
        if (enabled && parts[i] >= 0) ui->nodes[parts[i]].paint = fpas_paint_palette_swatch;
        colors[i] = fpas_well_rgba(swatches[i]);
        if (locking) locked[i] = locks[i];
    }
    parts[count] = options.no_shuffle ? -1 : fpas_find(ui, shuffle);
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
