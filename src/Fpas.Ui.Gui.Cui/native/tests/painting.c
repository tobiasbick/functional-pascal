// [Documentation](../../../../docs/ui/gui/canvas-and-bars/painting.md)
/* Native canvas painting must match direct public CUI calls, including DPI. */
#include "../fpas_cui.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct { bool adapter; cui_texture image; } scene;
static const uint8_t rgba[] = { 255,0,0,255, 0,255,0,255, 0,0,255,255, 255,255,0,255 };

static void paint(void *user, cui_ctx *ui, cui_rect r, cui_id images, int disabled) {
    const scene *s = user;
    (void)disabled;
    const cui_color ink = {24,32,48,255}, fill = {220,230,240,255};
    const uint32_t packed = 0xff302018u, other = 0xfff0e6dcu;
    const float rect[] = {10,10,100,60,8,2};
    const float circle[] = {145,40,18};
    const float line[] = {10,90,180,90,2};
    const float shadow[] = {210,20,50,50,5,8,-2,0,3};
    const float arc[] = {290,40,20,0,2.5f,2};
    const float text[] = {10,110,CUI_FONT_BODY,17};
    const float icon[] = {320,20,24,24,20};
    const float clip[] = {5,5,360,180};
    const cui_vec2 points[] = {{180,20},{200,65},{165,70}};
    if (s->adapter) {
        fpas_cui_canvas_primitive(ui,10,r.x,r.y,clip,0,0,NULL);
        fpas_cui_canvas_primitive(ui,3,r.x,r.y,rect,other,packed,NULL);
        fpas_cui_canvas_primitive(ui,1,r.x,r.y,circle,packed,0,NULL);
        fpas_cui_canvas_primitive(ui,2,r.x,r.y,line,packed,0,NULL);
        fpas_cui_canvas_primitive(ui,5,r.x,r.y,shadow,packed,0,NULL);
        fpas_cui_canvas_primitive(ui,0,r.x,r.y,rect,other,0,NULL);
        fpas_cui_canvas_primitive(ui,4,r.x,r.y,rect,other,packed,NULL);
        fpas_cui_canvas_primitive(ui,6,r.x,r.y,arc,packed,0,NULL);
        fpas_cui_canvas_primitive(ui,7,r.x,r.y,text,packed,0,"Gr\xc3\xbc\xc3\x9f" "e \xe6\x97\xa5\xe6\x9c\xac \xf0\x9f\x8e\xb5");
        fpas_cui_canvas_primitive(ui,8,r.x,r.y,icon,packed,0,CUI_ICON_CHECK);
        fpas_cui_canvas_points(ui,r.x,r.y,points,3,0,packed,2);
        fpas_cui_canvas_points(ui,r.x,r.y,points,3,2,other,1);
        const fpas_media_source source = {.pixels=rgba,.width=2,.height=2,.version=1};
        fpas_cui_canvas_image(ui,images,0,&source,r.x+240,r.y+120,60,50,4,0xffffffffu);
        fpas_cui_canvas_images_finish(ui,images,1);
    } else {
        cui_push_clip(ui,(cui_rect){r.x+5,r.y+5,360,180});
        cui_draw_rect_border(ui,(cui_rect){r.x+10,r.y+10,100,60},CUI_CORNERS(8),fill,ink,2);
        cui_draw_circle(ui,(cui_vec2){r.x+145,r.y+40},18,ink);
        cui_draw_line(ui,(cui_vec2){r.x+10,r.y+90},(cui_vec2){r.x+180,r.y+90},2,ink);
        cui_draw_shadow(ui,(cui_rect){r.x+210,r.y+20,50,50},CUI_CORNERS(5),8,-2,(cui_vec2){0,3},ink);
        cui_draw_rect(ui,(cui_rect){r.x+10,r.y+10,100,60},CUI_CORNERS(8),fill);
        cui_draw_rect_gradient(ui,(cui_rect){r.x+10,r.y+10,100,60},CUI_CORNERS(8),fill,ink);
        cui_draw_arc(ui,(cui_vec2){r.x+290,r.y+40},20,0,2.5f,2,ink);
        cui_draw_text(ui,CUI_FONT_BODY,17,(cui_vec2){r.x+10,r.y+110},ink,"Gr\xc3\xbc\xc3\x9f" "e \xe6\x97\xa5\xe6\x9c\xac \xf0\x9f\x8e\xb5",-1);
        cui_draw_icon(ui,CUI_ICON_CHECK,(cui_rect){r.x+320,r.y+20,24,24},20,ink);
        cui_vec2 shifted[3]; for (int i=0;i<3;i++) shifted[i]=(cui_vec2){r.x+points[i].x,r.y+points[i].y};
        cui_draw_polygon(ui,shifted,3,ink);
        cui_draw_polyline(ui,shifted,3,2,fill,true);
        cui_draw_image(ui,s->image,(cui_rect){r.x+240,r.y+120,60,50},(cui_rect){0,0,1,1},CUI_CORNERS(4),(cui_color){255,255,255,255});
    }
    cui_pop_clip(ui);
}

static void direct(cui_ctx *ui, cui_rect area, void *user) { paint(user,ui,area,0,0); }

static bool parity(bool dark, int scale) {
    const int width=400*scale, height=220*scale;
    const size_t bytes=(size_t)width*height*4;
    unsigned char *pixels[2]={calloc(1,bytes),calloc(1,bytes)};
    bool ok=pixels[0]&&pixels[1];
    for (int pass=0;ok&&pass<2;pass++) {
        cui_sw_renderer *renderer=cui_sw_create();
        const cui_ctx_desc desc={.renderer=cui_sw_iface(renderer)};
        cui_ctx *ui=cui_create(&desc);
        cui_set_viewport(ui,400,220,(float)scale);
        cui_set_reduced_motion(ui,true);
        cui_set_theme(ui,dark?CUI_THEME_DARK:CUI_THEME_LIGHT);
        scene s={pass!=0, pass ? 0 : cui_image_create(ui,2,2,rgba)};
        for (int frame=0;frame<3;frame++) {
            cui_begin_frame(ui,.016f);
            cui_column(ui,.pad=8) {
                if (pass) {
                    float x,y,w,h,dx,dy;
                    fpas_cui_drawing_canvas(ui,"paint",1,0,2,190,NULL,0,1,&x,&y,&w,&h,0,&dx,&dy,paint,&s,0, 0, 0, 0, -1);
                } else cui_canvas(ui,"paint",CUI_FILL,CUI_PX(190),direct,&s,sizeof s);
            }
            cui_end_frame(ui);
            cui_sw_render(renderer,cui_get_draw_data(ui),pixels[pass],width,height,width*4,CUI_SW_RGBA);
        }
        if (s.image) cui_image_destroy(ui,s.image);
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    ok=ok&&memcmp(pixels[0],pixels[1],bytes)==0;
    free(pixels[0]);free(pixels[1]);
    if (!ok) fprintf(stderr,"Painting parity failed: dark=%d scale=%d\n",dark,scale);
    return ok;
}

typedef struct { size_t blocks; } allocation_count;
static void *allocate(size_t size, void *user) { ((allocation_count *)user)->blocks++; return malloc(size); }
static void *resize(void *p,size_t size,void *user) { if (!p) ((allocation_count *)user)->blocks++; return realloc(p,size); }
static void release(void *p,void *user) { if (p) { ((allocation_count *)user)->blocks--; free(p); } }

typedef struct { int next, live, created; bool rgba[64]; } textures;
static cui_texture texture_create(void *user,int w,int h,cui_texture_format format,const void *pixels) {
    textures *t=user;
    (void)w;(void)h;(void)pixels;
    const int id=++t->next;
    if (format==CUI_TEXTURE_RGBA8) { t->rgba[id]=true;t->live++;t->created++; }
    return (cui_texture)id;
}
static void texture_update(void *user,cui_texture id,int x,int y,int w,int h,const void *pixels,int stride) {
    (void)user;(void)id;(void)x;(void)y;(void)w;(void)h;(void)pixels;(void)stride;
}
static void texture_destroy(void *user,cui_texture id) {
    textures *t=user;
    if (t->rgba[id]) { t->rgba[id]=false;t->live--; }
}
typedef struct { bool visible; int version; } image_scene;
static void image_paint(void *user,cui_ctx *ui,cui_rect r,cui_id images,int disabled) {
    const image_scene *s=user;
    (void)disabled;
    if (s->visible) {
        const fpas_media_source source={.pixels=rgba,.width=2,.height=2,.version=s->version};
        fpas_cui_canvas_image(ui,images,0,&source,r.x,r.y,30,30,0,0xffffffffu);
    }
    fpas_cui_canvas_images_finish(ui,images,s->visible?1:0);
}

static bool image_lifetime(void) {
    allocation_count count={0};textures tracking={0};
    const cui_ctx_desc desc={.allocator={allocate,resize,release,&count},
        .renderer={&tracking,texture_create,texture_update,texture_destroy}};
    cui_ctx *ui=cui_create(&desc);
    cui_set_viewport(ui,200,100,1);
    image_scene s={true,1};cui_id key=0;bool ok=true;
    for (int frame=0;frame<8;frame++) {
        if (frame==3) s.version++;
        if (frame==4) s.visible=false;
        if (frame==5) s.visible=true;
        cui_begin_frame(ui,.016f);
        key=cui_make_id(ui,"image-canvas");
        float x,y,w,h,dx,dy;
        fpas_cui_drawing_canvas(ui,"image-canvas",2,80,2,60,NULL,0,1,&x,&y,&w,&h,0,&dx,&dy,image_paint,&s,0, 0, 0, 0, -1);
        cui_end_frame(ui);
        ok=ok&&tracking.live==(s.visible?1:0);
        if (frame==2) ok=ok&&tracking.created==1;
        if (frame==3) ok=ok&&tracking.created==2;
        if (frame==6) { fpas_cui_forget(ui,key);ok=ok&&tracking.live==0; }
    }
    cui_destroy(ui);
    ok=ok&&tracking.live==0&&count.blocks==0;
    if (!ok) fprintf(stderr,"Canvas image lifetime failed: live=%d blocks=%zu\n",tracking.live,count.blocks);
    return ok;
}

static bool resources(void) {
    allocation_count count={0};
    const cui_ctx_desc desc={.allocator={allocate,resize,release,&count}};
    cui_ctx *ui=cui_create(&desc);
    void *captions=NULL;
    for (int iteration=0;iteration<20;iteration++) {
        char month[]="Januar", today[]="Heute \xe6\x97\xa5";
        const char *table[46]={0}; table[0]=month; table[45]=today;
        captions=fpas_cui_strings(ui,captions,table);
        memset((void *)table,0,sizeof table);
        memset(month,'X',sizeof month-1); memset(today,'X',sizeof today-1);
        if (!iteration) for (int frame=0;frame<640;frame++) { cui_begin_frame(ui,.016f);cui_end_frame(ui); }
        const cui_strings *copied=cui_get_strings(ui);
        if (!copied || strcmp(copied->month[0],"Januar") || copied->month[1]) return false;
        captions=fpas_cui_strings(ui,captions,NULL);
        float metrics[40]; fpas_cui_theme_metrics(ui,metrics,0);
        metrics[0]=3;metrics[14]=0;metrics[15]=1.5f;metrics[16]=2;metrics[17]=5;metrics[18]=-2;metrics[19]=.3f;
        fpas_cui_theme_metrics(ui,metrics,1);
        const cui_theme *t=cui_theme_get(ui);
        if (t->radius_control!=3 || t->font_scale!=1.5f || t->shadow_resting[0].spread!=-2) return false;
    }
    cui_destroy(ui);
    if (count.blocks) fprintf(stderr,"Resource allocations still live: %zu\n",count.blocks);
    return count.blocks==0;
}

int main(void) { return parity(false,1)&&parity(true,1)&&parity(false,2)&&parity(true,2)&&resources()&&image_lifetime()?0:1; }
