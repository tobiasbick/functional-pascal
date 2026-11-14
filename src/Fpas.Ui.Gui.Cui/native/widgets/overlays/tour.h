#ifndef FPAS_TOUR_H
#define FPAS_TOUR_H

/** Draws the native guided tour with borrowed step arrays copied into frame storage.
 * Targets are native keys, placements are cui_placement values. Returns the native
 * navigation event and proposed step. node/key identify the explanation card and
 * retained tour state. parts receives close, Back and Next/Done nodes, or -1.
 * width/padding zero select native defaults; block_target gates spotlight input.
 * focus is zero for no request or one plus the close/Back/Next slot.
 * Restores the base view's last widget so surrounding layout remains unchanged. */
CUI_API int fpas_cui_tour(cui_ctx *ui, const char *id, const unsigned int *targets,
    const char *const *titles, const char *const *bodies, const int *placements,
    int count, int *step, float width, float padding, int block_target,
    int focus, int *node, unsigned int *key, int *parts);

#endif
