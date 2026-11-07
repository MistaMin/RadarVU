#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Meter Meter;

typedef struct RadarVuSetup {
    int mode;           // 0 = radar, 1 = spectrum
    float target;       // LUFS
    float ceiling;      // dBTP
    float window;       // seconds, one radar sweep per window
    int kind;           // 0 VU, 1 RMS, 2 K-12, 3 K-14, 4 K-20, 5 DIN, 6 NORDIC, 7 BBC, 8 EBU, 9 LOUDNESS
    float reference_db; // -20 / -18 / -14 dBFS
    int fill_on;        // radar colour fill on/off
    int fft_on;         // spectrum analyser on/off
    int history_points; // 180 / 360 / 720 / 1440
    int skin;           // 0 Stock Dark, 1 Retro Console, 2 Ivory Terminal, 3 Matrix, 4 Modern Console, 5 Studio Amber
} RadarVuSetup;

typedef struct RadarVuReadings {
    float momentary, short_term, sliding, integrated, range, true_peak;
    float max_momentary, max_short_term;
    int range_stable;
    float elapsed, sample_peak;
} RadarVuReadings;

typedef struct RadarVuPaint {
    int kind;
    float x, y, w, h, x2, y2, radius, width;
    unsigned int c0, c1;
    int flags, npts;
    float pts[64];
    unsigned char text[48];
} RadarVuPaint;

typedef struct RadarVuSkinColours {
    uint32_t background, text, highlight, highlight_text;
} RadarVuSkinColours;

void radarvu_skin_colours(int skin, RadarVuSkinColours* out);

Meter* radarvu_create(double sample_rate);
void radarvu_destroy(Meter* meter);
void radarvu_set_sample_rate(Meter* meter, double sample_rate);
void radarvu_setup(Meter* meter, const RadarVuSetup* setup);
void radarvu_readings(const Meter* meter, RadarVuReadings* out);
void radarvu_reset(Meter* meter);
void radarvu_process(Meter* meter, const float* interleaved, int frames, int channels);
int radarvu_paint(const Meter* meter, RadarVuPaint* out, int cap);
void radarvu_view_size(float* w, float* h);
// Header chip hit-test: 0 none, 1 meter type, 2 reference, 3 target, 4 ceiling, 5 window,
// 6 fill toggle, 7 fft toggle, 8 reset, 9 radar, 10 spectrum, 11 resolution, 12 skin.
int radarvu_click(float x, float y);

#ifdef __cplusplus
}
#endif
