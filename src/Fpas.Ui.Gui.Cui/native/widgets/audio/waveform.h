#ifndef FPAS_CUI_WAVEFORM_H
#define FPAS_CUI_WAVEFORM_H
CUI_API int fpas_cui_waveform(cui_ctx *ui, const char *id, const float *peaks, int count, float *progress,
    int width_kind, float width, float height, float bar, float gap, int palette, uint32_t rgba, int flags);
#endif
