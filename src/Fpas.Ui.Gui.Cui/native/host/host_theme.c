#include "../fpas_cui.h"

/* A CUI token change requires updating the managed/shared palette contract. */
typedef char fpas_theme_token_count_must_match[(CUI_TOKEN_COUNT == 35) ? 1 : -1];

void fpas_cui_resolve_layout(cui_ctx *ui) { cui__layout(ui); }

void fpas_cui_theme_read(cui_ctx *ui, uint32_t *colors) {
    for (int token = 1; token < CUI_TOKEN_COUNT; token++) {
        cui_color c = ui->theme.colors[token];
        colors[token - 1] = (uint32_t)c.r | ((uint32_t)c.g << 8) | ((uint32_t)c.b << 16) | ((uint32_t)c.a << 24);
    }
}

void fpas_cui_theme(cui_ctx *ui, int dark, int accent, const uint32_t *colors) {
    float motion = ui->theme.motion;
    float font_scale = ui->theme.font_scale;
    cui__theme_init(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT, (cui_accent)accent);
    ui->theme.motion = motion;
    ui->theme.font_scale = font_scale;
    for (int token = 1; token < CUI_TOKEN_COUNT; token++) {
        uint32_t c = colors[token - 1];
        ui->theme.colors[token] = (cui_color){ (uint8_t)c, (uint8_t)(c >> 8), (uint8_t)(c >> 16), (uint8_t)(c >> 24) };
    }
    ui->target_theme = ui->theme;
    ui->from_theme = ui->theme;
}
