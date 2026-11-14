#ifndef FPAS_CUI_CHROME_INPUT_H
#define FPAS_CUI_CHROME_INPUT_H

/* Chrome components consult global shortcuts and held pointer buttons as well
   as cui__interact. Preserve the frame's events for the active view. */
typedef struct {
    uint8_t keys[CUI_KEY_COUNT];
    bool pressed[3], down[3], released[3];
    int mods;
    cui_id focus, active;
} fpas_chrome_input;

static void fpas_chrome_pause(cui_ctx *ui, fpas_chrome_input *input) {
    memcpy(input->keys, ui->kpressed, sizeof input->keys);
    memcpy(input->pressed, ui->mpressed, sizeof input->pressed);
    memcpy(input->down, ui->mdown, sizeof input->down);
    memcpy(input->released, ui->mreleased, sizeof input->released);
    input->mods = ui->mods;
    input->focus = ui->focus;
    input->active = ui->active;
    memset(ui->kpressed, 0, sizeof ui->kpressed);
    memset(ui->mpressed, 0, sizeof ui->mpressed);
    memset(ui->mdown, 0, sizeof ui->mdown);
    memset(ui->mreleased, 0, sizeof ui->mreleased);
    ui->mods = 0;
    ui->focus = ui->active = 0;
}

static void fpas_chrome_resume(cui_ctx *ui, const fpas_chrome_input *input) {
    memcpy(ui->kpressed, input->keys, sizeof input->keys);
    memcpy(ui->mpressed, input->pressed, sizeof input->pressed);
    memcpy(ui->mdown, input->down, sizeof input->down);
    memcpy(ui->mreleased, input->released, sizeof input->released);
    ui->mods = input->mods;
    ui->focus = input->focus;
    ui->active = input->active;
}

#endif
