#ifndef FPAS_COLOR_PICKER_H
#define FPAS_COLOR_PICKER_H

/** Draws CUI's color picker.
 * @param ui Current drawing context.
 * @param id Stable trigger identity.
 * @param rgba Model color with red in the low byte.
 * @param enabled Whether input is accepted.
 * @param focus Whether to request trigger focus.
 * @param nodes Optional array for the trigger, saturation/value, hue and eight presets.
 * @param open Receives whether the popup is open.
 * @return Proposed color with the original alpha channel. */
CUI_API uint32_t fpas_cui_color_picker(cui_ctx *ui, const char *id, uint32_t rgba,
    int enabled, int focus, int *nodes, int *open);
#endif
