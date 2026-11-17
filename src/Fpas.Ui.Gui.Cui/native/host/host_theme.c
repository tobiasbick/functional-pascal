#include "../fpas_cui.h"

/* A CUI token change requires updating the managed/shared palette contract. */
typedef char fpas_theme_token_count_must_match[(CUI_TOKEN_COUNT == 35) ? 1 : -1];

void fpas_cui_resolve_layout(cui_ctx *ui) { cui__layout(ui); }

int fpas_cui_system_dark(void) { return cui__plat_system_dark(); }

CUI_API void fpas_cui_follow_theme(cui_app *app, int follow) { cui_app_follow_system_theme(app, follow != 0); }

/* Explicit field marshalling keeps the ABI independent of cui_theme layout. */
CUI_API void fpas_cui_theme_metrics(cui_ctx *ui, float *v, int write) {
    cui_theme *t = cui_theme_get(ui);
    float *fields[] = { &t->radius_control, &t->radius_panel, &t->radius_surface,
        &t->control_sm, &t->control_md, &t->control_lg,
        &t->text_xs, &t->text_sm, &t->text_body, &t->text_lg,
        &t->heading[0], &t->heading[1], &t->heading[2], &t->heading[3], &t->motion, &t->font_scale };
    for (int i = 0; i < 16; i++) {
        if (write) { if (v[i] >= 0) *fields[i] = v[i]; }
        else v[i] = *fields[i];
    }
    cui_shadow *layers[] = { &t->shadow_resting[0], &t->shadow_resting[1],
        &t->shadow_raised[0], &t->shadow_raised[1], &t->shadow_floating[0], &t->shadow_floating[1] };
    for (int i = 0; i < 6; i++) {
        float *s = v + 16 + i * 4;
        if (write) { if (s[1] >= 0) *layers[i] = (cui_shadow){s[0], s[1], s[2], s[3]}; }
        else { s[0] = layers[i]->y; s[1] = layers[i]->blur; s[2] = layers[i]->spread; s[3] = layers[i]->alpha; }
    }
    if (write) {
        if (v[14] >= 0) ui->reduced_motion = v[14] == 0;
        ui->target_theme = ui->from_theme = *t;
        ui->theme_t = 1;
    }
}

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
