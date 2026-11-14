#ifndef FPAS_CUI_VOICE_WAVE_H
#define FPAS_CUI_VOICE_WAVE_H
CUI_API int fpas_cui_voice_wave(cui_ctx *ui, const char *id, const float *levels, const uint8_t *owners, int count,
    float *progress, int width_kind, float width, float pitch, float step, int owner, int palette, uint32_t rgba, int flags);
#endif
