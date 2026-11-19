// [Documentation](../../../../../docs/ui/gui/creative-controls/aspect-ratio-picker.md)
#ifndef FPAS_CUI_ASPECT_PICKER_H
#define FPAS_CUI_ASPECT_PICKER_H
/** Draws an aspect-ratio well and proposes a selected choice.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param label Literal well caption.
 * @param value Current zero-based choice; receives the proposal.
 * @param widths Relative widths, aligned with heights and labels.
 * @param heights Positive relative heights.
 * @param labels Choice captions; empty entries use native captions.
 * @param count Choice count, up to 32; zero uses six native photo ratios.
 * @param custom Optional custom-choice caption.
 * @param base Short-side pixel count, or zero for the native default.
 * @param display Optional value caption.
 * @param flags Bit 0 enables input, bit 1 adds Custom, bit 3 selects plain styling.
 * @param focus Requested choice index, or minus one.
 * @param parts Receives one node index per choice, including Custom, or minus one.
 * @return Nonzero when enabled input changes the selection. */
CUI_API int fpas_cui_aspect_picker(cui_ctx *ui, const char *id,
    const char *label, int *value, const int *widths, const int *heights, const char *const *labels, int count, const char *custom, int base, const char *display, int flags, int focus, int *parts);
#endif
