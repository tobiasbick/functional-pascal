#ifndef FPAS_CUI_MOTION_TEXT_H
#define FPAS_CUI_MOTION_TEXT_H
typedef struct {
    int font, width_kind, palette, flags;
    uint32_t rgba;
    float size, width, speed, delay;
} fpas_motion_text;
#endif
