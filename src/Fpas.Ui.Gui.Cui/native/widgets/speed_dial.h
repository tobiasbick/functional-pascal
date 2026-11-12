#ifndef FPAS_SPEED_DIAL_H
#define FPAS_SPEED_DIAL_H

/** Draws a native speed dial and reports the chosen action.
 * @param ui Current drawing context.
 * @param id Stable dial identity.
 * @param labels Literal action captions.
 * @param icons Action glyphs.
 * @param count Number of actions, from one to 16.
 * @param icon Optional trigger glyph.
 * @param tooltip Optional trigger hint.
 * @param placement Native popup placement.
 * @param enabled Whether the dial accepts input.
 * @param focus Request focus on the trigger or one of the actions, encoded as one plus its node slot.
 * @param nodes Output trigger and visible action nodes.
 * @param open Whether the dial is open.
 * @return Chosen action index, or -1. */
CUI_API int fpas_cui_speed_dial(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, const char *icon, const char *tooltip,
    int placement, int enabled, int focus, int *nodes, int *open);
#endif
