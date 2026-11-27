// [Documentation](../../../../docs/ui/gui/cui.md#theme-and-resources)
#include "../fpas_cui.h"

/* A CUI token change requires updating the managed/shared palette contract. */
typedef char fpas_theme_token_count_must_match[(CUI_TOKEN_COUNT == 35) ? 1 : -1];

void fpas_cui_resolve_layout(cui_ctx *ui) { cui__layout(ui); }

int fpas_cui_system_dark(void) { return cui__plat_system_dark(); }

CUI_API void fpas_cui_follow_theme(cui_app *app, int follow) { cui_app_follow_system_theme(app, follow != 0); }

/** Transfers values without exposing the native theme structure across the ABI. */
static void fpas_theme_metrics(cui_theme *t, float *v, bool write) {
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
}

/** Reads or replaces the active metrics independently of the palette. */
CUI_API void fpas_cui_theme_metrics(cui_ctx *ui, float *v, int write) {
    cui_theme *t = cui_theme_get(ui);
    fpas_theme_metrics(t, v, write != 0);
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

/** Applies the model's target palette with CUI's native appearance transition. */
void fpas_cui_theme(cui_ctx *ui, int dark, int accent, const uint32_t *colors, float *metrics, int animate) {
    const cui_theme_mode mode = dark ? CUI_THEME_DARK : CUI_THEME_LIGHT;
    const bool changed = ui->mode != mode || ui->accent != (cui_accent)accent;
    cui_theme target;
    cui__theme_build(&target, mode, (cui_accent)accent);
    target.motion = ui->theme.motion;
    target.font_scale = ui->theme.font_scale;
    if (metrics) fpas_theme_metrics(&target, metrics, true);
    for (int token = 1; token < CUI_TOKEN_COUNT; token++) {
        uint32_t c = colors[token - 1];
        target.colors[token] = (cui_color){ (uint8_t)c, (uint8_t)(c >> 8), (uint8_t)(c >> 16), (uint8_t)(c >> 24) };
    }
    const bool continuing = ui->theme_t < 1 &&
        memcmp(target.colors, ui->target_theme.colors, sizeof target.colors) == 0;
    if (animate && target.motion > 0 && (changed || continuing)) {
        const cui_theme previous = ui->theme;
        if (changed) {
            ui->from_theme = previous;
            ui->theme_t = 0;
        }
        ui->theme = target;
        memcpy(ui->theme.colors, previous.colors, sizeof previous.colors);
        ui->theme.tone_sky = previous.tone_sky;
        ui->theme.tone_teal = previous.tone_teal;
        ui->theme.tone_green = previous.tone_green;
        ui->theme.tone_amber = previous.tone_amber;
        ui->theme.tone_rose = previous.tone_rose;
        ui->theme.tone_orchid = previous.tone_orchid;
        ui->theme.tone_violet = previous.tone_violet;
        ui->animating = true;
    } else {
        ui->from_theme = ui->theme = target;
        ui->theme_t = 1;
    }
    ui->target_theme = target;
    ui->mode = mode;
    ui->accent = (cui_accent)accent;
    ui->reduced_motion = target.motion == 0;
}
