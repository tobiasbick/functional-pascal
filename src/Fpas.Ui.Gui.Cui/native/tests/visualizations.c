/* Pixel parity and pointer contracts against direct pinned CUI calls. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct { cui_ctx *ui; cui_sw_renderer *sw; unsigned char *pixels; int selected; } sample;
static int failures, comparisons;
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d: %s\n",__LINE__,#x); ++failures; } } while(0)
static sample create(int width, bool dark) {
    sample s={0}; s.selected=-1; s.sw=cui_sw_create();
    s.ui=cui_create(&(cui_ctx_desc){.renderer=cui_sw_iface(s.sw)});
    cui_set_viewport(s.ui,width,600,1);cui_set_reduced_motion(s.ui,true);
    cui_set_theme(s.ui,dark?CUI_THEME_DARK:CUI_THEME_LIGHT);cui_input_mouse_leave(s.ui);
    s.pixels=malloc(700*600*4);return s;
}
static void destroy(sample s) { free(s.pixels);cui_destroy(s.ui);cui_sw_destroy(s.sw); }

static int calendar_skip(sample *s) {
    int base=cui_date_to_days((cui_date){2026,1,5}), wd=((base+3)%7+7)%7;
    int weeks=(wd+364+6)/7;
    cui_rect r=cui_get_rect(s->ui,cui_make_id(s->ui,"widget"));
    int fit=(int)floorf((r.w-32+3)/14);if(fit<1)fit=1;
    return fit<weeks?(weeks-fit)*7-wd:0;
}

static int draw(sample *s, bool bridge, int kind, int width, bool enabled, bool extras) {
    float matrix[12]={1,2,3,4,5,6,7,8,9,10,11,12}, days[364];
    for(int i=0;i<364;i++) days[i]=(float)(i%13);
    float a[]={82,64,90,58,75,88},b[]={70,85,72,80,60,79};
    const char *rows[]={"Mon","Tue","Wed"},*columns[]={"0h",NULL,"6h","9h"};
    const char *axes[]={"Speed","Size","Polish","Docs","Platforms","Tests"};
    cui_series series[]={{"1.0",a,6,{0}},{"1.1",b,6,CUI_COLOR_SERIES(3)}};
    fpas_chart_series bs[]={{"1.0",a,6,0,0},{"1.1",b,6,4,0}};
    cui_viz_item items[]={{"Layout",38,{0}},{"Text",24,{0}},{"Paint",21,{0}},{"Input",9,{0}},{"Other",8,{0}}};
    fpas_viz_item bi[]={{"Layout",38,0,0},{"Text",24,0,0},{"Paint",21,0,0},{"Input",9,0,0},{"Other",8,0,0}};
    if(extras) { items[1].color=(cui_color){17,109,90,255}; bi[1].palette=-1;bi[1].rgba=0xff5a6d11u; }
    int result=-1,node;
    cui_begin_frame(s->ui,.04f);
    cui_column(s->ui,.pad=16,.gap=12) {
        if(!bridge)cui_push_disabled(s->ui,!enabled);
        switch(kind) {
        case 0:
            if(bridge)result=fpas_cui_heatmap(s->ui,"widget",matrix,3,4,rows,columns,extras?18:0,0,extras?20:0,0,0,"%.0f req",0,0,enabled,&node);
            else result=cui_heatmap(s->ui,"widget",matrix,3,4,.row_labels=rows,.col_labels=columns,.cell=extras?18:0,.min=0,.max=extras?20:0,.format="%.0f req");
            break;
        case 1: {
            if(bridge)result=fpas_cui_calendar_heatmap(s->ui,"widget",2026,1,5,days,364,0,0,0,"%.0f","contributions",enabled,&node);
            else {
                result=cui_calendar_heatmap(s->ui,"widget",((cui_date){2026,1,5}),days,364,.format="%.0f",.unit="contributions");
                if(result>=0)result+=calendar_skip(s);
            }
            break;
        }
        case 2:
            if(bridge)result=fpas_cui_treemap(s->ui,"widget",bi,5,1,0,240,"%.0f KB",1,&s->selected,enabled,&node);
            else result=cui_treemap(s->ui,"widget",items,5,.height=240,.format="%.0f KB",.selected=&s->selected);
            break;
        case 3:
            if(bridge)result=fpas_cui_radar_chart(s->ui,"widget",bs,2,axes,6,220,100,"%.0f",!extras,enabled,&node);
            else result=cui_radar_chart(s->ui,"widget",series,2,axes,6,.size=220,.max=100,.format="%.0f",.no_legend=extras);
            break;
        case 4:
            if(bridge)result=fpas_cui_funnel(s->ui,"widget",bi,5,1,0,0,0,"%.0f",enabled,&node);
            else result=cui_funnel(s->ui,"widget",items,5,.format="%.0f");
            break;
        case 5:
            if(bridge)result=fpas_cui_split_bar(s->ui,"widget",bi,5,1,0,12,"%.0f ms",!extras,enabled,&node);
            else result=cui_split_bar(s->ui,"widget",items,5,.height=12,.format="%.0f ms",.no_legend=extras);
            break;
        case 6:
            if(bridge)result=fpas_cui_waffle(s->ui,"widget",bi,5,14,10,10,"%.0f",enabled,&node);
            else result=cui_waffle(s->ui,"widget",items,5,.cell=14,.format="%.0f");
            break;
        }
        if(!bridge)cui_pop_disabled(s->ui);
    }
    memset(matrix,0,sizeof matrix);memset(days,0,sizeof days);memset(a,0,sizeof a);memset(b,0,sizeof b);
    cui_end_frame(s->ui);
    cui_sw_render(s->sw,cui_get_draw_data(s->ui),s->pixels,width,600,width*4,CUI_SW_RGBA);
    return result;
}
int main(void) {
    for(int dark=0;dark<2;dark++)for(int compact=0;compact<2;compact++)for(int kind=0;kind<7;kind++) {
        int initial=compact?300:700;
        sample a=create(initial,dark),b=create(initial,dark);
        for(int frame=0;frame<16;frame++) {
            int width=frame>=8&&frame<12?(compact?700:300):initial;
            cui_set_viewport(a.ui,width,600,1);cui_set_viewport(b.ui,width,600,1);
            if(frame==3||frame==4||frame==5||frame==6||frame==7) {
                cui_rect r=cui_get_rect(a.ui,cui_make_id(a.ui,"widget"));
                float x=r.x+10,y=r.y+10;
                if(kind==0){x+=50;y+=18;}
                if(kind==1){x+=30;y+=20;}
                if(kind==3){x=r.x+r.w/2;y=r.y+40;}
                cui_input_mouse_move(a.ui,x,y);cui_input_mouse_move(b.ui,x,y);
            } else {cui_input_mouse_leave(a.ui);cui_input_mouse_leave(b.ui);}
            if(frame==4||frame==5||frame==6||frame==7) {
                bool down=frame==4||frame==6;
                cui_input_mouse_button(a.ui,CUI_MOUSE_LEFT,down);
                cui_input_mouse_button(b.ui,CUI_MOUSE_LEFT,down);
            }
            int ar=draw(&a,false,kind,width,frame!=7,frame>=12),br=draw(&b,true,kind,width,frame!=7,frame>=12);
            CHECK(ar==br);CHECK(a.selected==b.selected);
            cui_rect ra=cui_get_rect(a.ui,cui_make_id(a.ui,"widget")), rb=cui_get_rect(b.ui,cui_make_id(b.ui,"widget"));
            CHECK(ra.x==rb.x && ra.y==rb.y && ra.w==rb.w && ra.h==rb.h);
            if(frame==5)CHECK(ar>=0);
            if(memcmp(a.pixels,b.pixels,(size_t)width*600*4)) {
                fprintf(stderr,"parity kind=%d width=%d dark=%d frame=%d\n",kind,width,dark,frame);failures++;
            }
            comparisons++;
        }
        destroy(a);destroy(b);
    }
    printf("%d strict visualization pixel comparisons, %d failures\n",comparisons,failures);
    return failures?1:0;
}
