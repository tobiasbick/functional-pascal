/* Strict widget parity against the pinned CUI implementation, including input. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct { cui_ctx *ui; cui_sw_renderer *sw; unsigned char *pixels; } sample;
static int failures, comparisons;
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "line %d: %s\n", __LINE__, #x); failures++; } } while (0)
static sample create(int width, bool dark) {
    sample s = {0};
    s.sw = cui_sw_create(); s.ui = cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(s.sw)});
    cui_set_viewport(s.ui, width, 400, 1); cui_set_reduced_motion(s.ui, true);
    cui_set_theme(s.ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
    cui_input_mouse_leave(s.ui); s.pixels = malloc((size_t)width * 400 * 4);
    return s;
}
static void destroy(sample s) { free(s.pixels); cui_destroy(s.ui); cui_sw_destroy(s.sw); }

static int draw(sample *s, bool adapter, int kind, int width, bool gray) {
    float a[] = {18,22,21,27,31,29}, b[] = {12,13,15,14}, shares[] = {46,27,17,10};
    const char *labels[] = {"Jan","Feb","Mar","Apr","May","Jun"};
    const char *names[] = {"Windows","macOS","Linux","Other"};
    cui_series series[] = {{"Revenue",a,6,CUI_COLOR_SERIES(3)},{"Costs",b,4,{0}}};
    fpas_chart_series bridge[] = {{"Revenue",a,6,4,0},{"Costs",b,4,0,0}};
    cui_chart_annotation annotations[] = {{.kind=CUI_ANNOTATION_Y,.y=20,.label="Target",.detail="Threshold",.accent=true},
        {.kind=CUI_ANNOTATION_POINT,.x=2,.y=21,.label="Point"}};
    fpas_chart_annotation ba[] = {{1,0,20,"Target","Threshold",1},{2,2,21,"Point","",0}};
    cui_chart_opts o = {.labels=labels,.label_count=6,.height=220,.format="$%.0fk"};
    fpas_chart_options bo = {.width_kind=1,.labels=labels,.label_count=6,.height=220,.format="$%.0fk"};
    int shown = -1, node; unsigned int key;
    cui_begin_frame(s->ui,.04f);
    cui_column(s->ui,.pad=16,.gap=12) {
        switch(kind) {
        case 0: case 1: case 2: case 3:
            if (kind==1) { o.area=true; o.smooth=true; bo.flags=1|16; }
            if (kind==2) { o.dither=true; o.stacked=true; o.smooth=true; bo.flags=32|8|16; }
            if (kind==3) { o.min=0;o.max=50;o.no_grid=true;o.no_legend=true;
                o.annotations=annotations;o.annotation_count=2;
                bo.minimum=0;bo.maximum=50;bo.flags=2|4;bo.annotations=ba;bo.annotation_count=2; }
            if (adapter) fpas_cui_line_chart(s->ui,"line",bridge,2,&bo);
            else cui_line_chart_ex(s->ui,"line",series,2,&o);
            break;
        case 4: case 5:
            o.stacked=kind==5; bo.flags=kind==5?8:0;
            if (adapter) fpas_cui_bar_chart(s->ui,"bar",bridge,2,&bo);
            else cui_bar_chart_ex(s->ui,"bar",series,2,&o);
            break;
        case 6:
            o=(cui_chart_opts){.height=180,.format="%.0f%%"};
            bo=(fpas_chart_options){.height=180,.format="%.0f%%",.labels=names,.label_count=4};
            if (adapter) fpas_cui_donut_chart(s->ui,"donut",shares,4,&bo);
            else cui_donut_chart_ex(s->ui,"donut",shares,names,4,&o);
            break;
        case 7:
            if (adapter) fpas_cui_gauge(s->ui,"gauge","CPU ### literal",.42f,130);
            else cui_gauge(s->ui,"CPU ### literal",.42f,130);
            break;
        case 8: {
            cui_meter_part parts[]={{"Photos",81},{"Video",58},{"Apps",41},{"Docs",32}};
            fpas_meter_part bp[]={{"Photos",81},{"Video",58},{"Apps",41},{"Docs",32}};
            cui_pattern_meter_opts po={.total=256,.label="Storage",.unit="GB",.grayscale=gray};
            fpas_meter_options bpo={.total=256,.label="Storage",.unit="GB",.flags=gray?1:0,.width_kind=1};
            if (adapter) shown=fpas_cui_pattern_meter(s->ui,"storage",bp,4,&bpo,1,0,&node,&key);
            else shown=cui_pattern_meter_ex(s->ui,"storage",parts,4,&po);
            break;
        }
        case 9:
            if (adapter) fpas_cui_line_chart(s->ui,"empty",NULL,0,&bo);
            else cui_line_chart_ex(s->ui,"empty",NULL,0,&o);
            break;
        }
    }
    /* Destroy caller storage before CUI lays out and paints the frame. */
    memset(a,0,sizeof a); memset(b,0,sizeof b); memset(shares,0,sizeof shares);
    cui_end_frame(s->ui);
    cui_sw_render(s->sw,cui_get_draw_data(s->ui),s->pixels,width,400,width*4,CUI_SW_RGBA);
    return shown;
}
/* CUI-019: a collapsed large range produces NaN geometry. Do not rasterize it. */
static void range_regression(void) {
    sample s = create(600, false);
    float values[] = {1e10f, 1e10f, 1e10f};
    cui_series series = {"Constant", values, 3, {0}};
    cui_begin_frame(s.ui, .04f);
    cui_line_chart_ex(s.ui, "constant", &series, 1, NULL);
    cui_end_frame(s.ui);
    const cui_draw_data *draw = cui_get_draw_data(s.ui);
    int invalid = 0;
    for (int i = 0; i < draw->vertex_count; ++i)
        if (!isfinite(draw->vertices[i].x) || !isfinite(draw->vertices[i].y)) ++invalid;
    CHECK(invalid > 0);
    destroy(s);
}

int main(void) {
    range_regression();
    for(int dark=0;dark<2;dark++) for(int compact=0;compact<2;compact++) for(int kind=0;kind<10;kind++) {
        int width=compact?300:700; sample reference=create(width,dark), actual=create(width,dark);
        for(int frame=0;frame<8;frame++) {
            if(frame==3) {
                cui_rect r = cui_get_rect(reference.ui, cui_make_id(reference.ui,"storage") ^ 0x9E7E5u);
                float x = kind==8 ? r.x+12 : kind==6 ? 190 : 130;
                float y = kind==8 ? r.y+r.h/2 : 130;
                cui_input_mouse_move(reference.ui,x,y);cui_input_mouse_move(actual.ui,x,y);
            }
            if(frame==5) {
                cui_input_mouse_leave(reference.ui);cui_input_mouse_leave(actual.ui);
            }
            if(kind==8 && frame>=5) {
                cui_key key=frame==5?CUI_KEY_TAB:frame==6?CUI_KEY_RIGHT:CUI_KEY_END;
                cui_input_key(reference.ui,key,true,0);cui_input_key(reference.ui,key,false,0);
                cui_input_key(actual.ui,key,true,0);cui_input_key(actual.ui,key,false,0);
            }
            int a=draw(&reference,false,kind,width,frame>=6), b=draw(&actual,true,kind,width,frame>=6);
            CHECK(a==b);
            if(kind==8 && frame==3) CHECK(a==0);
            if(kind==8 && frame==7) CHECK(a==3);
            if(memcmp(reference.pixels,actual.pixels,(size_t)width*400*4)) {
                fprintf(stderr,"parity dark=%d width=%d kind=%d frame=%d\n",dark,width,kind,frame); failures++;
            }
            comparisons++;
        }
        destroy(reference);destroy(actual);
    }
    printf("%d strict chart pixel comparisons, %d failures\n",comparisons,failures);
    return failures?1:0;
}
