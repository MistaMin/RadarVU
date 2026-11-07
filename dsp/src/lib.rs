//! RadarVU — a BS.1770-5 loudness meter with a polar radar display.
//!
//! The loudness engine (K-weighting, gating, true peak, LRA) is the verified implementation shared
//! with ArcVu and lives in [`loudness`]. This module adds the radar display: a plan-position-indicator
//! sweep that plots momentary loudness in polar coordinates (angle = time, radius = level) against the
//! delivery target, a small stereo goniometer inset, and a right-hand readout panel.
//!
//! The editor does not draw widgets. [`radarvu_paint`] returns a list of drawing commands that the
//! JUCE editor replays, exactly like ArcVu. The audio passes through unchanged; this crate only meters.

use std::os::raw::{c_float, c_int};

pub mod loudness;
mod extra;
mod skin;
use skin::Palette;

use goodlookindri::color::Color;
use goodlookindri::scene::{Cmd, Point, Rect, Scene};
use goodlookindri::spectrum::SampleRing;
use goodlookindri::spectrum_draw::{SpectrumSettings, SpectrumStyle, SpectrumView};

/// Fixed design size. The editor scales the whole face to the window and keeps the aspect ratio.
pub const VIEW_W: f32 = 1100.0;
pub const VIEW_H: f32 = 560.0;

const HEADER_H: f32 = 44.0;
const SCOPE_CX: f32 = 370.0;
const SCOPE_CY: f32 = 268.0;
const SCOPE_R: f32 = 216.0;
const GONIO_CX: f32 = 115.0;
const GONIO_CY: f32 = 415.0;
const GONIO_R: f32 = 64.0;
/// One selected-meter sample per `window / history_len` seconds; the sweep makes one revolution per `window`.
/// Default half-degree spacing; users can select 180, 360, 720 or 1440 points.
const HISTORY: usize = 720;
const MAX_HISTORY: usize = 1440;
pub const HISTORY_POINTS: [usize; 4] = [180, 360, 720, 1440];
// Adjacent fill cells share a colour; their outer edge still uses every history point. Grouping
// four cells avoids hundreds of separately antialiased radial seams and reduces paint overhead.
const FILL_GROUP: usize = 4;
/// Goniometer trace buffer (decimated in the audio path).
const TRACE: usize = 512;
const MAX_POINTS: usize = 32; // 64 floats = 32 xy pairs per paint command
const TEXT_CAP: usize = 47;

/// Accepted range of the user-set loudness target (LUFS) and true-peak ceiling (dBTP).
pub const TARGET_RANGE: (f32, f32) = (-40.0, -6.0);
pub const CEILING_RANGE: (f32, f32) = (-6.0, 0.0);

fn sane(v: f32, fallback: f32, (lo, hi): (f32, f32)) -> f32 {
    if v.is_finite() { v.clamp(lo, hi) } else { fallback }
}

/// The values the earlier fixed lists offered, kept for migrating old projects.
pub const TARGETS: [f32; 4] = [-23.0, -18.0, -16.0, -14.0];
pub const CEILINGS: [f32; 3] = [-1.0, -2.0, -3.0];
pub const WINDOWS: [f32; 5] = [5.0, 10.0, 15.0, 20.0, 30.0];

// ------------------------------------------------------------------------------------------------
// Ballistic detector (ported from ArcVu). VU galvanometer (IEC 60268-17), sample peak with hold,
// RMS, and the DIN/Nordic/BBC/EBU peak-programme meters (ITU-R BS.645-2 release).
// ------------------------------------------------------------------------------------------------

const ZETA: f32 = 0.82;
const WN: f32 = 14.0;
const SINE_TO_RMS: f32 = std::f32::consts::PI / (2.0 * std::f32::consts::SQRT_2);
const PEAK_RELEASE_SECONDS: f32 = 1.6;
const PEAK_HOLD_S: f32 = 3.0;
const VALUE_HOLD_S: f32 = 5.0;
const CLIP_HOLD_S: f32 = 2.0;
const CORRELATION_TAU_S: f32 = 2000.0 / 48_000.0; // = 0.9995 per sample at 48 kHz
const MAX_CHANNELS: usize = 2;

#[derive(Clone, Copy)]
struct Channel {
    x: f32,               // galvanometer deflection (averaged level, linear)
    v: f32,               // galvanometer velocity
    peak: f32,            // sample peak with hold (linear)
    numeric_peak: f32,
    numeric_peak_hold_t: f32,
    hold: f32,
    over_run: u32,
    rms: f32,             // squared mean
    ppm: f32,             // peak-programme magnitude (non-EBU)
    ppm_slow: f32,        // EBU stage 1
    ppm_fast: f32,        // EBU stage 2
    ppm_third: f32,       // EBU stage 3
    ppm_slow_out: f32,    // EBU slow mode
    meter_hold: f32,      // held reading, dB
    meter_hold_t: f32,
    clip_t: f32,
}

impl Channel {
    fn silent() -> Self {
        Self {
            x: 1.0e-9, v: 0.0, peak: 1.0e-9, numeric_peak: 1.0e-9, numeric_peak_hold_t: 0.0,
            hold: 0.0, over_run: 0, rms: 1.0e-16, ppm: 1.0e-9, ppm_slow: 1.0e-9, ppm_fast: 1.0e-9,
            ppm_third: 1.0e-9, ppm_slow_out: 1.0e-9, meter_hold: -80.0, meter_hold_t: 0.0, clip_t: 0.0,
        }
    }
}

/// Attack/release coefficients for the peak-programme meters (ITU-R BS.645-2 Annex 2 Table 1).
fn ppm_coeff(kind: i32, sr: f32) -> (f32, f32) {
    let (attack_ms, decay_db, decay_s) = match kind {
        5 | 6 => (5.0, 20.0, 1.7),   // DIN and Nordic, Type I
        8 => (5.6, 24.0, 2.8),       // EBU
        7 => (10.0, 26.0, 3.0),      // BBC Type IIa
        _ => (10.0, 20.0, 1.5),
    };
    let atk = 1.0 - (-1.0 / (attack_ms * 0.001 * sr)).exp();
    let rel = 1.0 - (-decay_db / (20.0 * decay_s * sr) * std::f32::consts::LN_10).exp();
    (atk, rel)
}

fn meter_colour(v: f32, p: Palette) -> Color {
    if v <= -100.0 { p.dim } else if v >= 0.0 { p.red } else if v >= -3.0 { p.amber } else { p.green }
}

/// Continuous six-stop heat ramp, chosen to suit the selected screen palette.
fn level_heat(pos: f32, p: Palette) -> Color {
    let t = pos.clamp(0.0, 1.0) * 5.0;
    let i = (t as usize).min(4);
    p.heat[i].mix(p.heat[i + 1], t - i as f32)
}

fn spectrum_settings() -> SpectrumSettings {
    let mut s = SpectrumSettings::default();
    s.style = SpectrumStyle::Multicolour;
    s.size_index = 3; // 4096-point FFT
    s
}

fn level_colour(v: f32, target: f32, p: Palette) -> Color {
    if v <= -100.0 { return p.dim; }
    let rel = v - target;
    if rel > 3.0 { p.red } else if rel > 1.0 { p.amber } else if rel >= -1.0 { p.green } else { p.blue }
}

fn peak_colour(v: f32, ceiling: f32, p: Palette) -> Color {
    if v <= -100.0 { p.dim } else if v > ceiling { p.red } else if v > ceiling - 1.0 { p.amber } else { p.green }
}

fn db_text(v: f32) -> String {
    if v <= -100.0 { "−−.−".to_string() }
    else {
        let s = format!("{:.1}", v.abs());
        if v < -0.049 { format!("−{s}") } else { s }
    }
}

/// The header chips, shared by the drawing code and the click hit-test so the two cannot drift.
fn chips() -> [Rect; 12] {
    [
        Rect::new(150.0, 10.0, 96.0, 26.0),  // 0 meter type
        Rect::new(251.0, 10.0, 86.0, 26.0),  // 1 reference
        Rect::new(342.0, 10.0, 118.0, 26.0), // 2 target
        Rect::new(465.0, 10.0, 118.0, 26.0), // 3 ceiling
        Rect::new(588.0, 10.0, 90.0, 26.0),  // 4 window
        Rect::new(683.0, 10.0, 64.0, 26.0),  // 5 fill toggle
        Rect::new(752.0, 10.0, 64.0, 26.0),  // 6 fft toggle
        Rect::new(821.0, 10.0, 72.0, 26.0),  // 7 reset
        Rect::new(898.0, 10.0, 66.0, 26.0),  // 8 radar mode
        Rect::new(969.0, 10.0, 66.0, 26.0),  // 9 spectrum mode
        Rect::new(632.0, 504.0, 172.0, 34.0), // 10 history resolution
        Rect::new(814.0, 504.0, 254.0, 34.0), // 11 skin
    ]
}

pub struct Meter {
    sample_rate: f32,
    loud: loudness::Loudness,
    target: f32,
    ceiling: f32,
    window: f32,
    mode: i32,
    kind: i32,
    reference_db: f32,
    slow_mode: bool,
    aes17: bool,
    history: [f32; MAX_HISTORY],
    history_len: usize,
    skin: i32,
    history_idx: usize,
    history_acc: f64,
    history_step: f64,
    elapsed_frames: u64,
    trace: [[f32; 2]; TRACE],
    trace_idx: usize,
    trace_dec: u32,
    trace_dec_ct: u32,
    channels: [Channel; MAX_CHANNELS],
    overs: u32,
    correlation: f32,
    corr_xy: f32,
    corr_xx: f32,
    corr_yy: f32,
    spectrum_ring: SampleRing,
    spectrum: SpectrumView,
    last_paint_frames: u64,
    scratch: Vec<f32>,
    fill_on: bool,
    fft_on: bool,
}

impl Meter {
    pub fn new(sample_rate: f64) -> Self {
        let mut m = Self {
            sample_rate: 48_000.0,
            loud: loudness::Loudness::new(sample_rate),
            target: -14.0,
            ceiling: -1.0,
            window: 10.0,
            mode: 0,
            kind: 0,
            reference_db: -18.0,
            slow_mode: false,
            aes17: false,
            history: [loudness::FLOOR; MAX_HISTORY],
            history_len: HISTORY,
            skin: 0,
            history_idx: 0,
            history_acc: 0.0,
            history_step: 10.0 / HISTORY as f64,
            elapsed_frames: 0,
            trace: [[0.0; 2]; TRACE],
            trace_idx: 0,
            trace_dec: 8,
            trace_dec_ct: 0,
            channels: [Channel::silent(); MAX_CHANNELS],
            overs: 0,
            correlation: 0.0,
            corr_xy: 0.0,
            corr_xx: 1.0e-8,
            corr_yy: 1.0e-8,
            spectrum_ring: SampleRing::new(),
            spectrum: SpectrumView::new(spectrum_settings()),
            last_paint_frames: 0,
            scratch: Vec::with_capacity(16384),
            fill_on: true,
            fft_on: true,
        };
        m.set_sample_rate(sample_rate);
        m
    }

    fn set_sample_rate(&mut self, sr: f64) {
        self.sample_rate = sr.clamp(8_000.0, 768_000.0) as f32;
        self.loud.set_sample_rate(sr);
    }

    fn set_window(&mut self, seconds: f32) {
        let window = seconds.clamp(1.0, 30.0);
        if window != self.window {
            self.window = window;
            self.clear_history();
        }
        self.loud.set_window(self.window);
        self.history_step = self.window as f64 / self.history_len as f64;
    }

    fn set_history_points(&mut self, points: usize) {
        let points = if HISTORY_POINTS.contains(&points) { points } else { HISTORY };
        if self.history_len != points {
            self.history_len = points;
            self.history_step = self.window as f64 / points as f64;
            self.clear_history();
        }
    }

    /// The current reading of one channel for the selected meter type, in the meter's own dB.
    fn reading(&self, channel: usize) -> f32 {
        if self.kind == 9 { return self.loud.readings().momentary; }
        let state = &self.channels[channel.min(MAX_CHANNELS - 1)];
        let align_peak = 10f32.powf(self.reference_db / 20.0);
        let align_rms = align_peak * std::f32::consts::FRAC_1_SQRT_2;
        let rms_db = 10.0 * state.rms.log10() + if self.aes17 { 3.0103 } else { 0.0 };
        let rms_reading = match self.kind {
            1 => rms_db,
            2 => rms_db - (-12.0),
            3 => rms_db - (-14.0),
            4 => rms_db - (-20.0),
            _ => rms_db - self.reference_db,
        };
        match self.kind {
            8 if self.slow_mode => 20.0 * (state.ppm_slow_out.max(1.0e-9) / align_peak).log10(),
            5 | 6 | 7 | 8 => 20.0 * (state.ppm / align_peak).log10(),
            1..=4 => rms_reading,
            _ => 20.0 * (state.x / align_rms).log10(), // VU
        }
    }

    /// The value the radar plots: the louder channel of the selected meter.
    fn radar_reading(&self) -> f32 {
        self.reading(0).max(self.reading(1))
    }

    /// Numeric peak of one channel on the selected meter's scale (or true peak for LOUDNESS).
    fn peak_db(&self, channel: usize) -> f32 {
        let state = &self.channels[channel.min(MAX_CHANNELS - 1)];
        if self.kind == 9 { return self.loud.readings().true_peak; }
        let align_peak = 10f32.powf(self.reference_db / 20.0);
        match self.kind {
            2 => 20.0 * state.numeric_peak.max(1.0e-9).log10() - (-12.0),
            3 => 20.0 * state.numeric_peak.max(1.0e-9).log10() - (-14.0),
            4 => 20.0 * state.numeric_peak.max(1.0e-9).log10() - (-20.0),
            _ => 20.0 * (state.numeric_peak.max(1.0e-9) / (align_peak * std::f32::consts::FRAC_1_SQRT_2)).log10(),
        }
    }

    fn any_clip(&self) -> bool {
        self.channels[0].clip_t > 0.0 || self.channels[1].clip_t > 0.0
    }

    /// Ballistic detector + peak/hold/over/clip + correlation. No allocation on this path.
    fn meter_process(&mut self, samples: &[f32], channels: usize) {
        let ch = channels.clamp(1, MAX_CHANNELS);
        let frames = samples.len() / ch;
        let dt = 1.0 / self.sample_rate;
        let release = (-dt / PEAK_RELEASE_SECONDS).exp();
        let rms_a = 1.0 - (-1.0 / (300.0 * 0.001 * self.sample_rate)).exp();
        let (ppm_atk, ppm_rel) = ppm_coeff(self.kind, self.sample_rate);
        // EBU (8): three amplitude stages fitted to Tech 3205, each releasing 24 dB in 2.8 s.
        let ebu_c1 = 1.0 - (-1.0 / (2.1834e-3 * self.sample_rate)).exp();
        let ebu_c2 = 1.0 - (-1.0 / (0.4374e-3 * self.sample_rate)).exp();
        let ebu_c3 = 1.0 - (-1.0 / (0.1167e-3 * self.sample_rate)).exp();
        let ebu_release = 10f32.powf(-24.0 / 2.8 / 20.0 / self.sample_rate);
        let ebu_slow_c = 1.0 - (-1.0 / (1.15 * self.sample_rate)).exp();
        // Correlation window: a 41.7 ms time constant at every rate (0.9995 per sample at 48 kHz).
        let corr_keep = (-1.0 / (CORRELATION_TAU_S * self.sample_rate)).exp();
        for frame in 0..frames {
            for ci in 0..ch {
                let sample = samples[frame * ch + ci];
                let rectified = sample.abs() * SINE_TO_RMS;
                let state = &mut self.channels[ci];
                // VU galvanometer (underdamped second-order movement; rise and fall share it).
                let accel = WN * WN * (rectified - state.x) - (2.0 * ZETA * WN) * state.v;
                state.v += accel * dt;
                state.x = (state.x + state.v * dt).max(1.0e-9);
                // Sample peak with a hold, then release.
                let mag = sample.abs().max(1.0e-9);
                if mag >= state.peak {
                    state.peak = mag;
                    state.hold = PEAK_HOLD_S;
                } else if state.hold > 0.0 {
                    state.hold = (state.hold - dt).max(0.0);
                } else {
                    state.peak = (state.peak * release).max(mag);
                }
                // Numeric peak readout (value hold).
                if mag >= state.numeric_peak {
                    state.numeric_peak = mag;
                    state.numeric_peak_hold_t = VALUE_HOLD_S;
                } else if state.numeric_peak_hold_t > 0.0 {
                    state.numeric_peak_hold_t = (state.numeric_peak_hold_t - dt).max(0.0);
                } else {
                    state.numeric_peak = mag;
                }
                // Over (3 consecutive samples above 0 dBFS) and clip.
                if mag > 1.0 {
                    state.over_run += 1;
                    if state.over_run == 3 { self.overs = self.overs.saturating_add(1); }
                } else {
                    state.over_run = 0;
                }
                if mag >= 1.0 { state.clip_t = CLIP_HOLD_S; }
                else { state.clip_t = (state.clip_t - dt).max(0.0); }
                // RMS (mean square).
                state.rms += rms_a * (sample * sample - state.rms);
                state.rms = state.rms.max(1.0e-16);
                // Peak-programme meter.
                if self.kind == 8 {
                    let level = sample.abs();
                    if level > state.ppm_slow { state.ppm_slow += ebu_c1 * (level - state.ppm_slow); } else { state.ppm_slow *= ebu_release; }
                    if state.ppm_slow > state.ppm_fast { state.ppm_fast += ebu_c2 * (state.ppm_slow - state.ppm_fast); } else { state.ppm_fast *= ebu_release; }
                    if state.ppm_fast > state.ppm_third { state.ppm_third += ebu_c3 * (state.ppm_fast - state.ppm_third); } else { state.ppm_third *= ebu_release; }
                    state.ppm = state.ppm_third.max(1.0e-9);
                    state.ppm_slow_out += ebu_slow_c * (state.ppm - state.ppm_slow_out);
                } else {
                    let coeff = if mag > state.ppm { ppm_atk } else { ppm_rel };
                    state.ppm += coeff * (mag - state.ppm);
                    state.ppm = state.ppm.max(1.0e-9);
                }
            }
            // Phase correlation (L/R, on the raw pass-through signal).
            let l = samples[frame * ch];
            let r = if ch > 1 { samples[frame * ch + 1] } else { l };
            self.corr_xx = self.corr_xx * corr_keep + l * l;
            self.corr_yy = self.corr_yy * corr_keep + r * r;
            self.corr_xy = self.corr_xy * corr_keep + l * r;
            let denom = (self.corr_xx * self.corr_yy).sqrt().max(1.0e-12);
            self.correlation = extra::correlation_db(self.corr_xy / denom);
        }
        // Held reading (max with a value hold), per channel.
        let held = [self.reading(0), self.reading(1)];
        for ci in 0..ch {
            let db = held[ci];
            let state = &mut self.channels[ci];
            if db >= state.meter_hold {
                state.meter_hold = db;
                state.meter_hold_t = VALUE_HOLD_S;
            } else if state.meter_hold_t > 0.0 {
                state.meter_hold_t -= dt * frames as f32;
            } else {
                state.meter_hold = db;
            }
        }
    }

    /// Feeds the loudness engine, the ballistic detector, and advances the sweep and the trace.
    pub fn process_interleaved(&mut self, samples: &[f32], channels: usize) {
        if channels == 0 { return; }
        let ch = channels.clamp(1, MAX_CHANNELS);
        self.scratch.clear();
        let frames = samples.len() / ch;
        let mut offset = 0;
        while offset < frames {
            // Advance both detectors only as far as the next history timestamp. Sampling after
            // an entire host block repeated its final reading when the block spanned several bins.
            let until_capture = ((self.history_step - self.history_acc) * self.sample_rate as f64 - 1.0e-7)
                .ceil().max(1.0) as usize;
            let n = until_capture.min(frames - offset);
            let chunk = &samples[offset * ch..(offset + n) * ch];
            self.loud.process_interleaved(chunk, ch);
            self.meter_process(chunk, ch);
            for frame in chunk.chunks_exact(ch) {
                let l = frame[0];
                let r = if ch > 1 { frame[1] } else { l };
                self.scratch.push(0.5 * (l + r)); // mono mix for the spectrum ring
                self.trace_dec_ct += 1;
                if self.trace_dec_ct >= self.trace_dec {
                    self.trace_dec_ct = 0;
                    self.trace[self.trace_idx] = [l, r];
                    self.trace_idx = (self.trace_idx + 1) % TRACE;
                }
            }
            self.elapsed_frames += n as u64;
            self.history_acc += n as f64 / self.sample_rate as f64;
            if self.history_acc + 1.0e-12 >= self.history_step {
                self.history_acc = (self.history_acc - self.history_step).max(0.0);
                self.history[self.history_idx] = self.radar_reading();
                self.history_idx = (self.history_idx + 1) % self.history_len;
            }
            offset += n;
        }
        self.spectrum_ring.push(&[self.scratch.as_slice()]);
    }

    /// Clears the history and sweep when the meter type changes, so the trail never mixes two scales.
    fn clear_history(&mut self) {
        self.history = [loudness::FLOOR; MAX_HISTORY];
        self.history_idx = 0;
        self.history_acc = 0.0;
    }

    fn reset(&mut self) {
        self.loud.reset();
        self.clear_history();
        self.elapsed_frames = 0;
        self.trace = [[0.0; 2]; TRACE];
        self.trace_idx = 0;
        self.trace_dec_ct = 0;
        self.channels = [Channel::silent(); MAX_CHANNELS];
        self.overs = 0;
        self.correlation = 0.0;
        self.corr_xy = 0.0;
        self.corr_xx = 1.0e-8;
        self.corr_yy = 1.0e-8;
        self.spectrum = SpectrumView::new(spectrum_settings());
        self.last_paint_frames = 0;
    }

    fn sweep_angle(&self) -> f32 {
        let turns = self.elapsed_frames as f64 / self.sample_rate as f64 / self.window as f64;
        (turns.fract() * std::f64::consts::TAU) as f32
    }

    fn history_angle(&self, k: usize) -> f32 {
        // Anchor each recorded point to its capture time, not the moving sweep head.
        self.sweep_angle() - (self.history_acc / self.window as f64) as f32 * std::f32::consts::TAU
            - (self.history_len - 1 - k) as f32 * std::f32::consts::TAU / self.history_len as f32
    }
}

fn paint(m: &mut Meter) -> Scene {
    let p = skin::palette(m.skin);
    let mut s = Scene::new();
    s.fill_rect(Rect::new(0.0, 0.0, VIEW_W, VIEW_H), p.bg_top, p.bg_bottom, true);
    paint_header(&mut s, m);
    paint_readouts(&mut s, m);
    match m.mode { 1 => paint_spectrum(&mut s, m), _ => paint_radar(&mut s, m) }
    paint_preferences(&mut s, m);
    s
}

fn chip(s: &mut Scene, p: Palette, r: Rect, label: &str, on: bool) {
    if on {
        s.fill_round(r, p.button_radius, p.active, p.active_bottom, true);
        s.stroke_round(r, p.button_radius, p.phosphor, 1.0);
        s.text(Point::new(r.center().x, r.center().y), label, p.active_text, 11.0, true);
    } else {
        s.fill_round(r, p.button_radius, p.button, p.button_bottom, true);
        s.stroke_round(r, p.button_radius, p.panel_edge, 1.0);
        s.text(Point::new(r.center().x, r.center().y), label, p.label, 11.0, true);
    }
}

fn paint_header(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    s.fill_rect(Rect::new(0.0, 0.0, VIEW_W, HEADER_H), p.header, p.header, true);
    s.line(Point::new(0.0, HEADER_H - 0.5), Point::new(VIEW_W, HEADER_H - 0.5), p.panel_edge, 1.0);
    s.text(Point::new(24.0, 27.0), "RADARVU", p.bright, 20.0, false);
    s.text(Point::new(26.0, 39.0), "METER · LUFS", p.label, 10.0, false);
    let [meter, reference, target, ceiling, window, fill, fft, reset, radar, spec, _, _] = chips();
    chip(s, p, meter, &format!("METER {}", extra::kind_name(m.kind)), false);
    chip(s, p, reference, &format!("REF {}", db_text(m.reference_db)), false);
    chip(s, p, target, &format!("TARGET {} LUFS", db_text(m.target)), false);
    chip(s, p, ceiling, &format!("CEILING {} dBTP", db_text(m.ceiling)), false);
    chip(s, p, window, &format!("SWEEP {} s", m.window.round() as i32), false);
    chip(s, p, fill, "FILL", m.fill_on);
    chip(s, p, fft, "FFT", m.fft_on);
    chip(s, p, reset, "↺ RESET", false);
    chip(s, p, radar, "RADAR", m.mode == 0);
    chip(s, p, spec, "SPEC", m.mode == 1);
}

fn paint_preferences(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let controls = chips();
    s.text(Point::new(633.0, 498.0), "RADAR RESOLUTION", p.label, 9.0, false);
    s.text(Point::new(815.0, 498.0), "COLOUR SKIN", p.label, 9.0, false);
    chip(s, p, controls[10], &format!("POINTS {}  ▾", m.history_len), false);
    chip(s, p, controls[11], &format!("{}  ▾", skin::NAMES[m.skin as usize]), false);
}

fn paint_tp_scale(s: &mut Scene, p: Palette, cell: Rect, tp: f32, ceiling: f32) {
    // True-peak scale -6 .. 0 dBTP under the readout, with the ceiling as a red tick.
    let bar = Rect::new(cell.x + 16.0, cell.y + TP_BAR_Y, cell.w - 32.0, 8.0);
    // Ticks are 2 px wide and are kept inside the bar, so a ceiling of 0 dBTP is not cut off.
    let x_of = |db: f32| (bar.x + ((db + 6.0) / 6.0).clamp(0.0, 1.0) * bar.w).clamp(bar.x + 1.0, bar.x + bar.w - 1.0);
    s.fill_rect(bar, p.screen, p.screen, true);
    s.stroke_round(bar, 1.0, p.tick, 1.0);
    s.fill_rect(Rect::new(x_of(ceiling) - 1.0, bar.y - 3.0, 2.0, bar.h + 6.0), p.red, p.red, true);
    if tp > -100.0 {
        s.fill_rect(Rect::new(x_of(tp) - 1.0, bar.y - 1.0, 2.0, bar.h + 2.0), p.white, p.white, true);
    }
}

/// The readout panel and its five cards. The cards are spread evenly with the same margin above
/// the first and below the last, so nothing touches or crosses the panel edge in any skin.
const READOUT_PANEL: Rect = Rect { x: 616.0, y: 52.0, w: 468.0, h: 432.0 };
const READOUT_PAD: f32 = 12.0;
const READOUT_CELL_H: f32 = 75.0;
const TP_BAR_Y: f32 = 62.0; // true-peak bar offset inside its card; the 14 px tick fits in the card

fn readout_cells() -> [Rect; 5] {
    let gap = (READOUT_PANEL.h - 2.0 * READOUT_PAD - 5.0 * READOUT_CELL_H) / 4.0;
    std::array::from_fn(|i| Rect::new(READOUT_PANEL.x + READOUT_PAD + 4.0, READOUT_PANEL.y + READOUT_PAD + i as f32 * (READOUT_CELL_H + gap),
        READOUT_PANEL.w - 2.0 * READOUT_PAD - 8.0, READOUT_CELL_H))
}

fn paint_readouts(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let panel = READOUT_PANEL;
    s.fill_round(panel, p.panel_radius, p.panel, p.panel, true);
    s.stroke_round(panel, p.panel_radius, p.panel_edge, 1.5);
    if p.hardware {
        for x in [623.0, 1077.0] {
            for y in [59.0, 477.0] {
                s.fill_ellipse_solid(Rect::new(x - 2.5, y - 2.5, 5.0, 5.0), p.tick);
                s.line(Point::new(x - 1.5, y), Point::new(x + 1.5, y), p.screen, 0.8);
            }
        }
    }
    let r = m.loud.readings();
    // (label, value text, colour, true-peak cell?)
    let cells: [(&str, String, Color, bool); 5] = [
        ("MOMENTARY", format!("{} LUFS", db_text(r.momentary)), level_colour(r.momentary, m.target, p), false),
        ("SHORT-TERM 3 s", format!("{} LUFS", db_text(r.short_term)), level_colour(r.short_term, m.target, p), false),
        ("INTEGRATED", format!("{} LUFS", db_text(r.integrated)), level_colour(r.integrated, m.target, p), false),
        ("TRUE PEAK dBTP", format!("{} dBTP", db_text(r.true_peak)), peak_colour(r.true_peak, m.ceiling, p), true),
        ("LOUDNESS RANGE", if r.range_stable == 1 { format!("{:.1} LU", r.range) } else { "settling".to_string() },
         if r.range_stable == 1 { p.bright } else { p.dim }, false),
    ];
    for ((label, text, colour, is_tp), cell) in cells.into_iter().zip(readout_cells()) {
        let y = cell.y;
        s.fill_round(cell, p.panel_radius.min(9.0), p.cell, p.cell_bottom, true);
        s.stroke_round(cell, p.panel_radius.min(9.0), p.panel_edge, 1.0);
        s.text(Point::new(648.0, y + 21.0), label, p.label, 11.0, false);
        s.text(Point::new(646.0, y + 58.0), &text, colour, 30.0, false);
        if is_tp {
            paint_tp_scale(s, p, cell, r.true_peak, m.ceiling);
        } else {
            s.fill_rect(Rect::new(648.0, y + 65.0, 404.0, 3.0), colour, colour, true);
        }
    }
}

fn dashed_circle(s: &mut Scene, cx: f32, cy: f32, r: f32, color: Color, width: f32) {
    let steps = 72;
    let seg = std::f32::consts::TAU / steps as f32;
    for i in (0..steps).step_by(2) {
        s.stroke_arc(Point::new(cx, cy), r, i as f32 * seg, (i as f32 + 0.6) * seg, color, width);
    }
}

fn paint_radar(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let (cx, cy, r) = (SCOPE_CX, SCOPE_CY, SCOPE_R);
    // Bezel and face.
    s.fill_ellipse_solid(Rect::new(cx - r - 10.0, cy - r - 10.0, (r + 10.0) * 2.0, (r + 10.0) * 2.0), p.bezel);
    s.stroke_ellipse(Rect::new(cx - r - 10.0, cy - r - 10.0, (r + 10.0) * 2.0, (r + 10.0) * 2.0), p.panel_edge, 1.5);
    s.fill_ellipse_solid(Rect::new(cx - r, cy - r, r * 2.0, r * 2.0), p.screen);
    s.stroke_ellipse(Rect::new(cx - r, cy - r, r * 2.0, r * 2.0), p.phosphor.with_alpha(0.35), 1.0);
    if p.hardware {
        s.stroke_ellipse(Rect::new(cx - r - 7.0, cy - r - 7.0, (r + 7.0) * 2.0, (r + 7.0) * 2.0), p.tick, 1.0);
    }
    if p.scanlines {
        for y in (-212..213).step_by(4) {
            let dy = y as f32;
            let dx = (r * r - dy * dy).sqrt();
            s.line(Point::new(cx - dx, cy + dy), Point::new(cx + dx, cy + dy), p.phosphor.with_alpha(0.045), 0.6);
        }
    }

    // Grid rings + labels on the selected meter's scale.
    for frac in [0.25f32, 0.5, 0.75, 1.0] {
        let rr = r * frac;
        s.stroke_ellipse(Rect::new(cx - rr, cy - rr, rr * 2.0, rr * 2.0), p.grid, 1.0);
        s.text(Point::new(cx + 6.0, cy - rr + 6.0), extra::scale_label(m.kind, frac), p.label, 9.0, false);
    }
    // Crosshair.
    s.line(Point::new(cx - r, cy), Point::new(cx + r, cy), p.grid, 1.0);
    s.line(Point::new(cx, cy - r), Point::new(cx, cy + r), p.grid, 1.0);
    // Diagonal half-spokes.
    for k in [1usize, 3, 5, 7] {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        let (sx, sy) = (a.cos(), a.sin());
        s.line(Point::new(cx + sx * r * 0.5, cy - sy * r * 0.5), Point::new(cx + sx * r, cy - sy * r), p.grid, 1.0);
    }
    // Outer ticks every 15 degrees.
    for k in 0..24 {
        let a = k as f32 * std::f32::consts::TAU / 24.0;
        let (sx, sy) = (a.cos(), a.sin());
        s.line(Point::new(cx + sx * (r - 6.0), cy - sy * (r - 6.0)), Point::new(cx + sx * r, cy - sy * r), p.tick, 1.0);
    }

    // Reference ring: the delivery target for LOUDNESS, or the 0 mark for the amplitude meters.
    if m.kind == 9 {
        dashed_circle(s, cx, cy, r * extra::position(9, m.target), p.green.with_alpha(0.9), 1.5);
    } else {
        dashed_circle(s, cx, cy, r * extra::position(m.kind, 0.0), p.white.with_alpha(0.35), 1.0);
    }

    // The sweep's phosphor trail, then the sweep line and head.
    paint_trail(s, m);
    paint_sweep(s, m);

    // Stereo goniometer inset and the meter numeric readout.
    paint_gonio(s, m);
    paint_meter_readout(s, m);
}

/// Share of the screen colour mixed into the fill at the oldest and newest ends of the trail. The
/// newest part keeps most of its heat colour; the contour drawn over it supplies the edge definition.
const FILL_OLDEST_BG: f32 = 0.85;
const FILL_NEWEST_BG: f32 = 0.3;

fn paint_trail(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let hub = Point::new(SCOPE_CX, SCOPE_CY);
    let point = |k: usize| {
        let i = (m.history_idx + k) % m.history_len;
        let angle = m.history_angle(k);
        let radius = extra::position(m.kind, m.history[i]) * SCOPE_R;
        Point::new(SCOPE_CX + radius * angle.cos(), SCOPE_CY - radius * angle.sin())
    };
    if m.fill_on {
        // Filled multicolour history follows every sample, with the colour fading with age.
        let bg = p.screen;
        // The ring buffer wraps in memory, not in time: never join newest back to oldest.
        for start in (0..m.history_len - 1).step_by(FILL_GROUP) {
            let end = (start + FILL_GROUP).min(m.history_len - 1);
            let middle = (start + end) / 2;
            let i = (m.history_idx + middle) % m.history_len;
            let heat = level_heat(extra::position(m.kind, m.history[i]), p);
            let t = middle as f32 / (m.history_len - 1) as f32;
            let colour = heat.mix(bg, FILL_OLDEST_BG + (FILL_NEWEST_BG - FILL_OLDEST_BG) * t);
            let mut sector = Vec::with_capacity(end - start + 2);
            sector.push(hub);
            sector.extend((start..=end).map(point));
            s.fill_poly(sector, colour, colour);
        }
    }
    // Keep a crisp contour visible with either fill setting. Each segment fits the flat C ABI's
    // 32-point capacity, with one shared endpoint so there are no gaps between segments.
    for start in (0..m.history_len - 1).step_by(MAX_POINTS - 1) {
        let end = (start + MAX_POINTS - 1).min(m.history_len - 1);
        let seg = (start..=end).map(point).collect();
        let alpha = 0.15 + 0.8 * end as f32 / (m.history_len - 1) as f32;
        s.stroke_poly(seg, p.phosphor.with_alpha(alpha), 1.2, false);
    }
}

fn paint_sweep(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let value = m.radar_reading();
    let a = m.sweep_angle();
    let head_r = extra::position(m.kind, value) * SCOPE_R;
    let (dx, dy) = (a.cos(), a.sin());
    // Afterglow.
    for k in 1..=4 {
        let aa = a - k as f32 * 0.03;
        s.line(
            Point::new(SCOPE_CX, SCOPE_CY),
            Point::new(SCOPE_CX + aa.cos() * SCOPE_R, SCOPE_CY - aa.sin() * SCOPE_R),
            p.phosphor.with_alpha(0.14 / k as f32),
            1.0,
        );
    }
    // Main sweep to the level head.
    s.line(Point::new(SCOPE_CX, SCOPE_CY), Point::new(SCOPE_CX + dx * head_r, SCOPE_CY - dy * head_r), p.phosphor, 2.0);
    s.fill_ellipse_solid(Rect::new(SCOPE_CX + dx * head_r - 3.0, SCOPE_CY - dy * head_r - 3.0, 6.0, 6.0), p.phosphor);
    s.fill_ellipse_solid(Rect::new(SCOPE_CX - 3.0, SCOPE_CY - 3.0, 6.0, 6.0), p.phosphor.with_alpha(0.6));
}

fn paint_gonio(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let (cx, cy, r) = (GONIO_CX, GONIO_CY, GONIO_R);
    s.fill_ellipse_solid(Rect::new(cx - r, cy - r, r * 2.0, r * 2.0), p.screen);
    s.stroke_ellipse(Rect::new(cx - r, cy - r, r * 2.0, r * 2.0), p.panel_edge, 1.5);
    s.line(Point::new(cx - r, cy), Point::new(cx + r, cy), p.grid, 1.0);
    s.line(Point::new(cx, cy - r), Point::new(cx, cy + r), p.grid, 1.0);
    s.text(Point::new(cx, cy - r - 8.0), "STEREO", p.label, 9.0, true);

    const CHUNKS: usize = 32;
    let per = TRACE / CHUNKS;
    for c in 0..CHUNKS {
        let mut seg: Vec<Point> = Vec::with_capacity(per + 2);
        for k in 0..=per {
            let kk = (c * per + k).min(TRACE - 1);
            let i = (m.trace_idx + kk) % TRACE;
            let (l, rr) = (m.trace[i][0], m.trace[i][1]);
            let mid = (l + rr) * 0.7071;
            let side = (l - rr) * 0.7071;
            seg.push(Point::new(cx + mid * r * 0.82, cy - side * r * 0.82));
        }
        let alpha = 0.25 + 0.6 * (c as f32 / (CHUNKS - 1) as f32);
        s.stroke_poly(seg, p.phosphor.with_alpha(alpha), 1.2, false);
    }
}

fn phase_gauge(s: &mut Scene, p: Palette, r: Rect, correlation: f32) {
    s.fill_rect(r, p.screen, p.screen, true);
    s.stroke_round(r, 2.0, p.tick, 1.0);
    let t = ((correlation.clamp(-39.0, 0.0) + 39.0) / 39.0).clamp(0.0, 1.0);
    let x = r.x + t * r.w;
    let colour = if correlation < -20.0 { p.red } else { p.green };
    s.fill_ellipse_solid(Rect::new(x - 4.0, r.center().y - 4.0, 8.0, 8.0), colour);
    s.text(Point::new(r.right() + 8.0, r.center().y + 3.0), "PHASE", p.label, 8.0, false);
}

fn paint_meter_readout(s: &mut Scene, m: &Meter) {
    let p = skin::palette(m.skin);
    let area = Rect::new(154.0, 496.0, 432.0, 56.0);
    s.fill_round(area, p.panel_radius, p.panel, p.panel, true);
    s.stroke_round(area, p.panel_radius, p.panel_edge, 1.0);
    let cur = m.radar_reading();
    let peak = m.peak_db(0).max(m.peak_db(1));
    let head = format!("{} {}", extra::kind_name(m.kind), extra::signed_db(cur));
    s.text(Point::new(170.0, 524.0), &head, meter_colour(cur, p), 22.0, false);
    s.text(Point::new(414.0, 517.0), &format!("PK {}", extra::signed_db(peak)), p.bright, 16.0, false);
    phase_gauge(s, p, Rect::new(170.0, 536.0, 220.0, 10.0), m.correlation);
    if m.overs > 0 {
        s.text(Point::new(414.0, 535.0), &format!("OVER {}", m.overs), p.amber, 11.0, false);
    }
    if m.any_clip() {
        s.text(Point::new(510.0, 535.0), "CLIP", p.red, 11.0, false);
    }
}

fn paint_spectrum(s: &mut Scene, m: &mut Meter) {
    let p = skin::palette(m.skin);
    let area = Rect::new(40.0, 52.0, 520.0, 432.0);
    s.fill_round(area, p.panel_radius, p.panel, p.panel, true);
    s.stroke_round(area, p.panel_radius, p.panel_edge, 1.5);
    s.text(Point::new(72.0, 84.0), "SPECTRUM", p.label, 12.0, false);
    if m.fft_on {
        // Update the analyser with the audio time since the last paint (clamped so a long gap between
        // paints does not decay the whole display in one step).
        let dt = ((m.elapsed_frames.saturating_sub(m.last_paint_frames)) as f32 / m.sample_rate).min(0.05);
        m.last_paint_frames = m.elapsed_frames;
        m.spectrum.update(&m.spectrum_ring, dt, m.sample_rate as f64);
        let plot = Rect::new(64.0, 104.0, 472.0, 348.0);
        m.spectrum.settings.range = std::array::from_fn(|i| level_heat(i as f32 / 6.0, p).0);
        let mut spectrum = Scene::new();
        m.spectrum.paint(&mut spectrum, plot, 20.0, 20000.0);
        for cmd in spectrum.cmds {
            // The toolkit emits a full-width peak-hold polyline. Split it before the flat C ABI
            // so every frequency is drawn, rather than silently truncating after 32 points.
            match cmd {
                Cmd::StrokePoly { pts, width, closed, .. } if pts.len() > MAX_POINTS => {
                    for start in (0..pts.len() - 1).step_by(MAX_POINTS - 1) {
                        let end = (start + MAX_POINTS).min(pts.len());
                        s.stroke_poly(pts[start..end].to_vec(), p.phosphor.with_alpha(0.4), width, false);
                    }
                    if closed { s.line(*pts.last().unwrap(), pts[0], p.phosphor.with_alpha(0.4), width); }
                }
                _ => s.cmds.push(cmd),
            }
        }
        s.text(Point::new(plot.x, plot.bottom() + 18.0), "20 Hz → 20 kHz", p.label, 9.0, false);
    } else {
        s.text(Point::new(area.center().x, area.center().y), "FFT OFF", p.dim, 22.0, true);
    }
}

// ------------------------------------------------------------------------------------------------
// C ABI — mirrored by plugin/radarvu.h.
// ------------------------------------------------------------------------------------------------

#[repr(C)]
pub struct RadarVuSetup {
    pub mode: c_int,
    pub target: c_float,
    pub ceiling: c_float,
    pub window: c_float,
    pub kind: c_int,
    pub reference_db: c_float,
    pub fill_on: c_int,
    pub fft_on: c_int,
    pub history_points: c_int,
    pub skin: c_int,
}

#[repr(C)]
pub struct RadarVuReadings {
    pub momentary: f32,
    pub short_term: f32,
    pub sliding: f32,
    pub integrated: f32,
    pub range: f32,
    pub true_peak: f32,
    pub max_momentary: f32,
    pub max_short_term: f32,
    pub range_stable: i32,
    pub elapsed: f32,
    pub sample_peak: f32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RadarVuPaint {
    pub kind: c_int,
    pub x: c_float,
    pub y: c_float,
    pub w: c_float,
    pub h: c_float,
    pub x2: c_float,
    pub y2: c_float,
    pub radius: c_float,
    pub width: c_float,
    pub c0: u32,
    pub c1: u32,
    pub flags: c_int,
    pub npts: c_int,
    pub pts: [c_float; 64],
    pub text: [u8; 48],
}

#[repr(C)]
pub struct RadarVuSkinColours {
    pub background: u32,
    pub text: u32,
    pub highlight: u32,
    pub highlight_text: u32,
}

#[no_mangle]
pub extern "C" fn radarvu_skin_colours(index: c_int, out: *mut RadarVuSkinColours) {
    if let Some(out) = unsafe { out.as_mut() } {
        let p = skin::palette(index);
        *out = RadarVuSkinColours { background: p.cell.0, text: p.bright.0,
            highlight: p.active.0, highlight_text: p.active_text.0 };
    }
}

fn pack(cmd: &Cmd) -> RadarVuPaint {
    let mut out = RadarVuPaint {
        kind: 0, x: 0.0, y: 0.0, w: 0.0, h: 0.0, x2: 0.0, y2: 0.0, radius: 0.0, width: 0.0,
        c0: 0, c1: 0, flags: 0, npts: 0, pts: [0.0; 64], text: [0; 48],
    };
    let put_pts = |out: &mut RadarVuPaint, pts: &[Point]| {
        let n = pts.len().min(MAX_POINTS);
        out.npts = n as c_int;
        for (i, p) in pts.iter().take(n).enumerate() {
            out.pts[i * 2] = p.x;
            out.pts[i * 2 + 1] = p.y;
        }
    };
    let put_text = |out: &mut RadarVuPaint, t: &str| {
        let b = t.as_bytes();
        let n = b.len().min(TEXT_CAP);
        out.text[..n].copy_from_slice(&b[..n]);
    };
    match cmd {
        Cmd::FillEllipse { rect, top, bottom, diagonal } => {
            out.kind = 1; out.x = rect.x; out.y = rect.y; out.w = rect.w; out.h = rect.h;
            out.c0 = top.0; out.c1 = bottom.0; out.flags = *diagonal as c_int;
        }
        Cmd::StrokeEllipse { rect, color, width } => {
            out.kind = 2; out.x = rect.x; out.y = rect.y; out.w = rect.w; out.h = rect.h;
            out.c0 = color.0; out.width = *width;
        }
        Cmd::FillRoundRect { rect, radius, top, bottom, vertical } => {
            out.kind = 3; out.x = rect.x; out.y = rect.y; out.w = rect.w; out.h = rect.h;
            out.radius = *radius; out.c0 = top.0; out.c1 = bottom.0; out.flags = *vertical as c_int;
        }
        Cmd::StrokeRoundRect { rect, radius, color, width } => {
            out.kind = 4; out.x = rect.x; out.y = rect.y; out.w = rect.w; out.h = rect.h;
            out.radius = *radius; out.c0 = color.0; out.width = *width;
        }
        Cmd::FillRect { rect, top, bottom, vertical } => {
            out.kind = 5; out.x = rect.x; out.y = rect.y; out.w = rect.w; out.h = rect.h;
            out.c0 = top.0; out.c1 = bottom.0; out.flags = *vertical as c_int;
        }
        Cmd::Line { a, b, color, width } => {
            out.kind = 6; out.x = a.x; out.y = a.y; out.x2 = b.x; out.y2 = b.y;
            out.c0 = color.0; out.width = *width;
        }
        Cmd::FillPoly { pts, top, bottom, diagonal } => {
            out.kind = 7; out.c0 = top.0; out.c1 = bottom.0; out.flags = *diagonal as c_int;
            put_pts(&mut out, pts);
        }
        Cmd::StrokePoly { pts, color, width, closed } => {
            out.kind = 8; out.c0 = color.0; out.width = *width; out.flags = *closed as c_int;
            put_pts(&mut out, pts);
        }
        Cmd::StrokeArc { center, radius, from, to, color, width } => {
            out.kind = 9; out.x = center.x; out.y = center.y; out.radius = *radius;
            out.x2 = *from; out.y2 = *to; out.c0 = color.0; out.width = *width;
        }
        Cmd::Text { at, text, color, size, center } => {
            out.kind = 10; out.x = at.x; out.y = at.y; out.w = *size; out.c0 = color.0;
            out.flags = *center as c_int; put_text(&mut out, text);
        }
        Cmd::ClipRound { rect, radius } => {
            out.kind = 11; out.x = rect.x; out.y = rect.y; out.w = rect.w; out.h = rect.h; out.radius = *radius;
        }
        Cmd::PopClip => out.kind = 12,
    }
    out
}

#[no_mangle]
pub extern "C" fn radarvu_create(sample_rate: f64) -> *mut Meter {
    Box::into_raw(Box::new(Meter::new(sample_rate)))
}

#[no_mangle]
pub extern "C" fn radarvu_destroy(meter: *mut Meter) {
    if !meter.is_null() { unsafe { drop(Box::from_raw(meter)) } }
}

#[no_mangle]
pub extern "C" fn radarvu_set_sample_rate(meter: *mut Meter, sample_rate: f64) {
    if let Some(m) = unsafe { meter.as_mut() } { m.set_sample_rate(sample_rate); }
}

#[no_mangle]
pub extern "C" fn radarvu_setup(meter: *mut Meter, setup: *const RadarVuSetup) {
    if setup.is_null() { return; }
    if let Some(m) = unsafe { meter.as_mut() } {
        let s = unsafe { &*setup };
        m.mode = s.mode;
        m.target = sane(s.target, -14.0, TARGET_RANGE);
        m.ceiling = sane(s.ceiling, -1.0, CEILING_RANGE);
        m.set_window(s.window);
        let kind = s.kind.clamp(0, 9);
        if kind != m.kind { m.kind = kind; m.clear_history(); }
        m.reference_db = s.reference_db.clamp(-36.0, 0.0);
        m.fill_on = s.fill_on != 0;
        m.fft_on = s.fft_on != 0;
        m.set_history_points(s.history_points.max(0) as usize);
        m.skin = s.skin.clamp(0, 5);
    }
}

#[no_mangle]
pub extern "C" fn radarvu_readings(meter: *const Meter, out: *mut RadarVuReadings) {
    if out.is_null() { return; }
    if let Some(m) = unsafe { meter.as_ref() } {
        let r = m.loud.readings();
        unsafe {
            *out = RadarVuReadings {
                momentary: r.momentary, short_term: r.short_term, sliding: r.sliding,
                integrated: r.integrated, range: r.range, true_peak: r.true_peak,
                max_momentary: r.max_momentary, max_short_term: r.max_short_term,
                range_stable: r.range_stable, elapsed: r.elapsed, sample_peak: r.sample_peak,
            };
        }
    }
}

#[no_mangle]
pub extern "C" fn radarvu_reset(meter: *mut Meter) {
    if let Some(m) = unsafe { meter.as_mut() } { m.reset(); }
}

#[no_mangle]
pub extern "C" fn radarvu_process(meter: *mut Meter, samples: *const c_float, frames: c_int, channels: c_int) {
    if samples.is_null() || frames <= 0 { return; }
    if let Some(m) = unsafe { meter.as_mut() } {
        let n = (frames as usize) * (channels.max(0) as usize);
        let data = unsafe { std::slice::from_raw_parts(samples, n) };
        m.process_interleaved(data, channels.max(0) as usize);
    }
}

#[no_mangle]
pub extern "C" fn radarvu_view_size(w: *mut c_float, h: *mut c_float) {
    if !w.is_null() { unsafe { *w = VIEW_W } }
    if !h.is_null() { unsafe { *h = VIEW_H } }
}

/// Hit-test for the header chips. Returns 0 = nothing, 1 = meter type, 2 = reference, 3 = target,
/// 4 = ceiling, 5 = window, 6 = fill, 7 = fft, 8 = reset, 9 = radar, 10 = spectrum, 11 = points, 12 = skin.
#[no_mangle]
pub extern "C" fn radarvu_click(x: c_float, y: c_float) -> c_int {
    let p = Point::new(x, y);
    for (i, r) in chips().iter().enumerate() {
        if r.contains(p) { return (i + 1) as c_int; }
    }
    0
}

#[no_mangle]
pub extern "C" fn radarvu_paint(meter: *const Meter, out: *mut RadarVuPaint, cap: c_int) -> c_int {
    if out.is_null() || cap <= 0 { return 0; }
    // The spectrum analyser is UI-thread-only state; the audio thread only touches the atomic ring
    // and the detector. The editor calls this from its paint (UI) thread.
    let Some(m) = (unsafe { (meter as *mut Meter).as_mut() }) else { return 0 };
    let scene = paint(m);
    let n = scene.cmds.len().min(cap as usize);
    let dest = unsafe { std::slice::from_raw_parts_mut(out, n) };
    for (slot, cmd) in dest.iter_mut().zip(scene.cmds.iter()) {
        *slot = pack(cmd);
        if slot.kind == 10 && skin::palette(m.skin).mono { slot.flags |= 2; }
    }
    n as c_int
}

#[cfg(test)]
mod ballistics;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radar_history_is_independent_of_host_buffer_size() {
        for sr in [44_100.0, 96_000.0] {
            let samples: Vec<f32> = (0..(sr as usize * 2)).flat_map(|i| {
                let t = i as f32 / sr as f32;
                let amp = 0.05 + 0.2 * (t * 7.0).sin().abs();
                let v = amp * (t * 1000.0 * std::f32::consts::TAU).sin();
                [v, v * 0.7]
            }).collect();
            for points in HISTORY_POINTS {
                for kind in [0, 1, 9] {
                    let mut reference = Meter::new(sr);
                    reference.kind = kind;
                    reference.set_history_points(points);
                    reference.set_window(1.0);
                    for chunk in samples.chunks(64 * 2) { reference.process_interleaved(chunk, 2); }
                    for block in [512, 2048, samples.len() / 2] {
                        let mut m = Meter::new(sr);
                        m.kind = kind;
                        m.set_history_points(points);
                        m.set_window(1.0);
                        for chunk in samples.chunks(block * 2) { m.process_interleaved(chunk, 2); }
                        assert_eq!(m.history_idx, reference.history_idx, "{sr} Hz, kind {kind}, block {block}");
                        for (i, (a, b)) in m.history.iter().zip(reference.history.iter()).enumerate() {
                            assert!((a - b).abs() < 1.0e-4,
                                "{sr} Hz, kind {kind}, block {block}, history {i}: {a} vs {b}");
                        }
                        let (min, max) = m.history[..points].iter().fold((f32::INFINITY, f32::NEG_INFINITY),
                            |(lo, hi), &v| (lo.min(v), hi.max(v)));
                        assert!(max - min > 0.5, "must retain the changing programme, not repeat the block endpoint");
                    }
                }
            }
        }
    }

    #[test]
    fn radar_capture_cadence_tracks_all_sweep_windows_and_resolutions() {
        for points in HISTORY_POINTS {
            for window in WINDOWS {
                let mut m = Meter::new(48_000.0);
                m.set_history_points(points);
                m.set_window(window);
                m.process_interleaved(&vec![0.1; 48_000 * 2], 2);
                let expected = (points as f32 / window) as usize;
                assert_eq!(m.history_idx, expected, "{points} points, window {window}");
                assert_eq!(m.history.iter().filter(|v| **v > loudness::FLOOR).count(), expected);
            }
        }
    }

    #[test]
    fn changing_resolution_preserves_measurements_and_restarts_only_the_trail() {
        let mut m = Meter::new(48_000.0);
        tone(&mut m, 2.0, 0.1);
        let before = m.loud.readings();
        let elapsed = m.elapsed_frames;
        let vu = m.radar_reading();
        for points in HISTORY_POINTS {
            m.set_history_points(points);
            assert_eq!(m.history_len, points);
            assert_eq!(m.history_idx, 0);
            assert!(m.history.iter().all(|v| *v <= loudness::FLOOR));
            assert_eq!(m.elapsed_frames, elapsed);
            assert_eq!(m.loud.readings().momentary, before.momentary);
            assert_eq!(m.radar_reading(), vu);
        }
        m.set_history_points(999);
        assert_eq!(m.history_len, HISTORY);
    }

    #[test]
    fn skins_preserve_history_measurements_and_fit_the_drawing_budget() {
        let mut m = Meter::new(48_000.0);
        tone(&mut m, 1.0, 0.1);
        let before = m.history;
        let reading = m.loud.readings().momentary;
        for skin in 0..6 {
            let setup = RadarVuSetup {
                mode: 0, target: -14.0, ceiling: -1.0, window: 10.0, kind: 0,
                reference_db: -18.0, fill_on: 1, fft_on: 1,
                history_points: HISTORY as i32, skin,
            };
            radarvu_setup(&mut m, &setup);
            assert_eq!(m.history, before, "skin changes must retain the trail");
            assert_eq!(m.loud.readings().momentary, reading);
        }
        for skin in 0..6 {
            m.skin = skin;
            for points in HISTORY_POINTS {
                m.set_history_points(points);
                m.history.fill(-10.0);
                for mode in [0, 1] {
                    m.mode = mode;
                    let scene = paint(&mut m);
                    assert!(scene.cmds.len() < 8192);
                    for cmd in &scene.cmds {
                        if let Cmd::FillPoly { pts, .. } | Cmd::StrokePoly { pts, .. } = cmd {
                            assert!(pts.len() <= MAX_POINTS, "skin {skin}, points {points}, mode {mode}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn radar_history_does_not_rotate_between_captures() {
        let mut m = Meter::new(48_000.0);
        m.process_interleaved(&vec![0.1; 700 * 2], 2);
        let before = m.history_angle(HISTORY - 1);
        let index = m.history_idx;
        m.process_interleaved(&vec![0.1; 100 * 2], 2);
        assert_eq!(m.history_idx, index);
        assert!((m.history_angle(HISTORY - 1) - before).abs() < 1.0e-6);
        assert!(m.sweep_angle() > before);
        m.process_interleaved(&vec![0.1; 600 * 2], 2);
        assert_eq!(m.history_idx, index + 1);
        assert!((m.history_angle(HISTORY - 2) - before).abs() < 1.0e-6,
            "an existing point must stay in place when the next point arrives");
    }

    #[test]
    fn radar_contour_is_visible_with_fill_and_fits_the_paint_abi() {
        let mut m = Meter::new(48_000.0);
        for points in HISTORY_POINTS {
            m.set_history_points(points);
            m.history.fill(-10.0);
            for fill in [false, true] {
                m.fill_on = fill;
                let mut scene = Scene::new();
                paint_trail(&mut scene, &m);
                let mut fills = 0;
                let mut fill_edges = 0;
                let mut edges = 0;
                for cmd in &scene.cmds {
                    match cmd {
                        Cmd::FillPoly { pts, .. } => {
                            fills += 1;
                            assert!(pts.len() >= 3 && pts.len() <= MAX_POINTS);
                            fill_edges += pts.len() - 2;
                        }
                        Cmd::StrokePoly { pts, closed, .. } => {
                            assert!(pts.len() <= MAX_POINTS);
                            assert!(!closed, "newest must not connect to oldest");
                            edges += pts.len() - 1;
                        }
                        _ => panic!("unexpected trail command"),
                    }
                }
                assert_eq!(fills, if fill { (points - 1).div_ceil(FILL_GROUP) } else { 0 });
                assert_eq!(fill_edges, if fill { points - 1 } else { 0 });
                assert_eq!(edges, points - 1, "every consecutive pair must appear without truncation");
            }
        }
    }

    fn tone(m: &mut Meter, seconds: f32, amplitude: f32) {
        let frames = (seconds * m.sample_rate) as usize;
        let mut inter = vec![0.0f32; frames * 2];
        let mut phase = 0.0f32;
        let step = std::f32::consts::TAU * 1000.0 / m.sample_rate;
        for i in 0..frames {
            let v = amplitude * phase.sin();
            phase += step;
            inter[i * 2] = v;
            inter[i * 2 + 1] = v;
        }
        m.process_interleaved(&inter, 2);
    }

    #[test]
    fn smoke_process_readings_and_paint() {
        let mut m = Meter::new(48_000.0);
        assert!(m.sweep_angle() == 0.0);
        tone(&mut m, 4.0, 10f32.powf(-23.0 / 20.0));
        let r = m.loud.readings();
        assert!(r.momentary > -25.0 && r.momentary < -21.0, "momentary {}", r.momentary);
        assert!(r.true_peak > -24.0, "true peak {}", r.true_peak);
        assert!(m.elapsed_frames > 0);
        // The radar must emit a drawing list and the sweep must have advanced.
        let scene = paint(&mut m);
        assert!(scene.cmds.len() > 100, "scene has {} commands", scene.cmds.len());
        assert!(m.sweep_angle() > 0.0);
        // Serialization stays within the flat command capacity.
        let mut out = vec![
            RadarVuPaint { kind: 0, x: 0.0, y: 0.0, w: 0.0, h: 0.0, x2: 0.0, y2: 0.0, radius: 0.0, width: 0.0, c0: 0, c1: 0, flags: 0, npts: 0, pts: [0.0; 64], text: [0; 48] };
            8192
        ];
        let n = radarvu_paint(&m, out.as_mut_ptr(), 8192);
        assert!(n > 100 && n <= 8192);
    }

    #[test]
    fn reset_clears_history_and_sweep() {
        let mut m = Meter::new(48_000.0);
        tone(&mut m, 2.0, 0.1);
        assert!(m.sweep_angle() > 0.0);
        m.reset();
        assert!(m.sweep_angle() == 0.0);
        assert!(m.loud.readings().elapsed == 0.0);
    }

    #[test]
    fn click_hit_test_matches_the_chips() {
        assert_eq!(radarvu_click(198.0, 20.0), 1);   // meter type
        assert_eq!(radarvu_click(294.0, 20.0), 2);   // reference
        assert_eq!(radarvu_click(401.0, 20.0), 3);   // target
        assert_eq!(radarvu_click(524.0, 20.0), 4);   // ceiling
        assert_eq!(radarvu_click(633.0, 20.0), 5);   // window
        assert_eq!(radarvu_click(715.0, 20.0), 6);   // fill
        assert_eq!(radarvu_click(784.0, 20.0), 7);   // fft
        assert_eq!(radarvu_click(857.0, 20.0), 8);   // reset
        assert_eq!(radarvu_click(931.0, 20.0), 9);   // radar
        assert_eq!(radarvu_click(1002.0, 20.0), 10); // spectrum
        assert_eq!(radarvu_click(700.0, 521.0), 11); // resolution
        assert_eq!(radarvu_click(900.0, 521.0), 12); // skin
        assert_eq!(radarvu_click(300.0, 300.0), 0);  // scope, no chip
    }

    #[test]
    fn vu_reads_zero_for_a_reference_sine() {
        let mut m = Meter::new(48_000.0);
        m.kind = 0;
        m.reference_db = -18.0;
        tone(&mut m, 4.0, 10f32.powf(-18.0 / 20.0));
        assert!((m.reading(0) - 0.0).abs() < 0.6, "VU {}", m.reading(0));
    }

    #[test]
    fn rms_reads_absolute_dbfs() {
        let mut m = Meter::new(48_000.0);
        m.kind = 1;
        tone(&mut m, 4.0, 10f32.powf(-20.0 / 20.0));
        // RMS of a -20 dBFS peak sine is -23.01 dBFS (sine RMS = peak - 3.01 dB).
        assert!((m.reading(0) - (-23.01)).abs() < 0.2, "RMS {}", m.reading(0));
    }

    #[test]
    fn rms_aes17_reads_zero_for_a_full_scale_sine() {
        let mut m = Meter::new(48_000.0);
        m.kind = 1;
        m.aes17 = true;
        tone(&mut m, 4.0, 1.0);
        assert!((m.reading(0) - 0.0).abs() < 0.1, "AES-17 {}", m.reading(0));
    }

    #[test]
    fn k12_reads_zero_at_minus_twelve_dbfs_rms() {
        let mut m = Meter::new(48_000.0);
        m.kind = 2; // K-12: 0 is -12 dBFS RMS
        let amp = 10f32.powf(-12.0 / 20.0) * std::f32::consts::SQRT_2;
        tone(&mut m, 4.0, amp);
        assert!((m.reading(0)).abs() < 0.3, "K-12 {}", m.reading(0));
    }

    #[test]
    fn correlation_reads_mono_for_identical_channels() {
        let mut m = Meter::new(48_000.0);
        tone(&mut m, 1.0, 0.1);
        assert!(m.correlation > -2.0, "correlation {}", m.correlation);
    }

    #[test]
    fn ppm_reads_a_finite_level() {
        let mut m = Meter::new(48_000.0);
        m.kind = 7; // BBC
        tone(&mut m, 0.05, 0.9);
        let held = m.reading(0);
        // A hot 50 ms burst reads clearly above the -18 dBFS reference but stays finite.
        assert!(held > 0.0 && held < 30.0, "BBC {}", held);
    }

    #[test]
    fn meter_type_change_clears_the_radar_history() {
        let mut m = Meter::new(48_000.0);
        tone(&mut m, 2.0, 0.1);
        assert!(m.history.iter().any(|v| *v > loudness::FLOOR));
        m.kind = 1;
        m.clear_history();
        assert!(m.history.iter().all(|v| *v <= loudness::FLOOR));
    }

    // -----------------------------------------------------------------------------------------
    // Meter ballistics (added 2026-11-02 for the Windows release QA; uncommitted QA addition).
    // Standards: VU per IEC 60268-17 (300 ms rise to 99% of deflection, overshoot 1.0-1.5%,
    // symmetric fall); PPM decay per ITU-R BS.645-2 Annex 2 Table 1 (Type I: 20 dB in 1.7 s)
    // and EBU Tech 3205 (24 dB in 2.8 s); peak hold per the product's 5 s value hold.
    // -----------------------------------------------------------------------------------------

    /// Feeds a 1 kHz stereo tone block, continuing `phase` so blocks join seamlessly.
    fn tone_block(m: &mut Meter, frames: usize, amplitude: f32, phase: &mut f32) {
        let mut inter = vec![0.0f32; frames * 2];
        let step = std::f32::consts::TAU * 1000.0 / m.sample_rate;
        for i in 0..frames {
            let v = amplitude * phase.sin();
            *phase += step;
            inter[i * 2] = v;
            inter[i * 2 + 1] = v;
        }
        m.process_interleaved(&inter, 2);
    }

    fn silence_block(m: &mut Meter, frames: usize) {
        let inter = vec![0.0f32; frames * 2];
        m.process_interleaved(&inter, 2);
    }

    #[test]
    fn vu_step_rise_overshoot_and_fall_follow_iec_60268_17() {
        let sr = 48_000.0;
        let amp = 10f32.powf(-18.0 / 20.0); // reference sine: steady state must read 0 VU
        // Steady-state reference.
        let mut steady = Meter::new(sr);
        steady.kind = 0;
        steady.reference_db = -18.0;
        tone(&mut steady, 4.0, amp);
        let target_db = steady.reading(0);
        assert!((target_db).abs() < 0.6, "VU steady state {} dB", target_db);

        // Rise: 99% of the steady deflection is target_db + 20*log10(0.99).
        let rise_thr = target_db + 20.0 * 0.99f32.log10();
        let mut m = Meter::new(sr);
        m.kind = 0;
        m.reference_db = -18.0;
        let ms = (sr * 0.001) as usize;
        let mut phase = 0.0f32;
        let mut rise_t = None;
        let mut max_db = f32::NEG_INFINITY;
        for step in 0..2000 {
            tone_block(&mut m, ms, amp, &mut phase);
            let r = m.reading(0);
            if rise_t.is_none() && r >= rise_thr {
                rise_t = Some((step + 1) as f32 * 0.001);
            }
            max_db = max_db.max(r);
        }
        let rise_t = rise_t.expect("VU never reached 99% of the reference deflection");
        assert!(
            (0.27..=0.33).contains(&rise_t),
            "VU 99% rise {:.3} s (IEC 60268-17: 0.300 s +/-10%)",
            rise_t
        );
        // Overshoot in per cent of the deflection (linear domain).
        let overshoot = (10f32.powf((max_db - target_db) / 20.0) - 1.0) * 100.0;
        assert!(
            (0.5..=1.6).contains(&overshoot),
            "VU overshoot {:.2}% (IEC 60268-17: 1.0-1.5%)",
            overshoot
        );
        // Fall: symmetric movement; silence must bring the pointer to 1% of the reference
        // deflection (target - 40 dB) in about the same time.
        let fall_thr = target_db - 40.0;
        let mut fall_t = None;
        for step in 0..2000 {
            silence_block(&mut m, ms);
            if fall_t.is_none() && m.reading(0) <= fall_thr {
                fall_t = Some((step + 1) as f32 * 0.001);
            }
        }
        let fall_t = fall_t.expect("VU never fell to 1% of the reference deflection");
        assert!(
            (0.25..=0.35).contains(&fall_t),
            "VU fall to 1% {:.3} s (symmetric with the 0.300 s rise)",
            fall_t
        );
        eprintln!(
            "VU ballistics: rise {} s, overshoot {:.2}%, fall {} s",
            rise_t, overshoot, fall_t
        );
    }

    #[test]
    fn ppm_type_i_falls_20_db_in_1_7_s() {
        let sr = 48_000.0;
        let mut m = Meter::new(sr);
        m.kind = 5; // DIN Type I
        m.reference_db = -18.0;
        tone(&mut m, 3.0, 10f32.powf(-18.0 / 20.0));
        let start = m.reading(0);
        let ms = (sr * 0.001) as usize;
        let (mut t_hi, mut t_lo) = (None, None);
        for step in 0..5000 {
            silence_block(&mut m, ms);
            let r = m.reading(0);
            if t_hi.is_none() && r <= start - 2.0 {
                t_hi = Some((step + 1) as f32 * 0.001);
            }
            if r <= start - 22.0 {
                t_lo = Some((step + 1) as f32 * 0.001);
                break;
            }
        }
        let fall = t_lo.unwrap() - t_hi.unwrap(); // seconds for the middle 20 dB
        assert!(
            (1.53..=1.87).contains(&fall),
            "Type I PPM 20 dB fall {:.3} s (ITU-R BS.645-2: 1.7 s)",
            fall
        );
        eprintln!("PPM Type I: 20 dB fall {} s", fall);
    }

    #[test]
    fn ebu_ppm_falls_24_db_in_2_8_s() {
        let sr = 48_000.0;
        let mut m = Meter::new(sr);
        m.kind = 8; // EBU
        m.reference_db = -18.0;
        tone(&mut m, 3.0, 10f32.powf(-18.0 / 20.0));
        let start = m.reading(0);
        let ms = (sr * 0.001) as usize;
        let (mut t_hi, mut t_lo) = (None, None);
        for step in 0..7000 {
            silence_block(&mut m, ms);
            let r = m.reading(0);
            if t_hi.is_none() && r <= start - 2.0 {
                t_hi = Some((step + 1) as f32 * 0.001);
            }
            if r <= start - 26.0 {
                t_lo = Some((step + 1) as f32 * 0.001);
                break;
            }
        }
        let fall = t_lo.unwrap() - t_hi.unwrap(); // seconds for the middle 24 dB
        assert!(
            (2.52..=3.08).contains(&fall),
            "EBU PPM 24 dB fall {:.3} s (EBU Tech 3205: 2.8 s)",
            fall
        );
        eprintln!("PPM EBU: 24 dB fall {} s", fall);
    }

    #[test]
    fn ppm_type_i_5ms_burst_matches_an_independent_one_pole_model() {
        // Independent model: the Type I attack is a one-pole integrator with a 5 ms time
        // constant over the rectified 1 kHz sine. Simulate that recurrence here (not via the
        // Meter) and require the meter's continuous-vs-5 ms-burst droop to match it.
        let sr = 48_000.0;
        let amp = 10f32.powf(-18.0 / 20.0);
        let atk = 1.0 - (-1.0 / (0.005 * sr as f32)).exp();
        let mut model = |seconds: f32| -> f32 {
            let frames = (seconds * sr as f32) as usize;
            let mut y = 0.0f32;
            let mut phase = 0.0f32;
            let step = std::f32::consts::TAU * 1000.0 / sr as f32;
            for _ in 0..frames {
                let mag = (amp * phase.sin()).abs();
                phase += step;
                if mag > y {
                    y += atk * (mag - y);
                }
            }
            y
        };
        let model_droop = 20.0 * (model(0.005) / model(3.0)).log10(); // negative dB

        let mut settled = Meter::new(sr);
        settled.kind = 5;
        settled.reference_db = -18.0;
        tone(&mut settled, 3.0, amp);
        let full = settled.reading(0);

        let mut m = Meter::new(sr);
        m.kind = 5;
        m.reference_db = -18.0;
        let mut phase = 0.0f32;
        tone_block(&mut m, (sr * 0.005) as usize, amp, &mut phase);
        let droop = m.reading(0) - full; // negative dB
        assert!(
            (droop - model_droop).abs() < 0.3,
            "Type I 5 ms burst droop {:.2} dB vs independent 5 ms one-pole model {:.2} dB",
            droop,
            model_droop
        );
        eprintln!(
            "PPM Type I: 5 ms burst droop {:.2} dB (model {:.2} dB)",
            droop, model_droop
        );
    }

    #[test]
    fn numeric_peak_holds_5s_then_falls_and_reset_clears() {
        let sr = 48_000.0;
        let mut m = Meter::new(sr);
        tone(&mut m, 0.1, 0.5); // peak 0.5 -> +14.99 on the -18 dBFS VU alignment scale
        let held = m.peak_db(0);
        let expected = 20.0
            * (0.5f32 / (10f32.powf(-18.0 / 20.0) * std::f32::consts::FRAC_1_SQRT_2)).log10();
        assert!((held - expected).abs() < 0.1, "peak of the burst: {} vs {}", held, expected);
        silence_block(&mut m, (sr * 4.0) as usize);
        assert!(
            (m.peak_db(0) - held).abs() < 0.01,
            "numeric peak moved during the 5 s hold: {} vs {}",
            m.peak_db(0),
            held
        );
        silence_block(&mut m, (sr * 3.0) as usize); // 7.1 s after the burst
        assert!(
            m.peak_db(0) < held - 0.5,
            "numeric peak still held after the 5 s hold: {}",
            m.peak_db(0)
        );
        m.reset();
        assert!(m.peak_db(0) < -100.0, "reset did not clear the peak hold");
    }

    #[test]
    fn readings_do_not_depend_on_the_buffer_size() {
        let sr = 48_000.0;
        let amp = 10f32.powf(-20.0 / 20.0);
        let mut small = Meter::new(sr);
        let mut big = Meter::new(sr);
        small.kind = 1;
        big.kind = 1;
        let mut phase = 0.0f32;
        let total = (sr * 3.0) as usize;
        let mut fed = 0;
        while fed < total {
            let n = 64.min(total - fed);
            tone_block(&mut small, n, amp, &mut phase);
            fed += n;
        }
        let mut phase = 0.0f32;
        let mut fed = 0;
        while fed < total {
            let n = 4096.min(total - fed);
            tone_block(&mut big, n, amp, &mut phase);
            fed += n;
        }
        assert!(
            (small.reading(0) - big.reading(0)).abs() < 0.01,
            "RMS reading depends on the buffer size: {} vs {}",
            small.reading(0),
            big.reading(0)
        );
        assert!(
            (small.loud.readings().integrated - big.loud.readings().integrated).abs() < 0.01,
            "integrated loudness depends on the buffer size: {} vs {}",
            small.loud.readings().integrated,
            big.loud.readings().integrated
        );
    }

    #[test]
    fn digital_silence_reads_the_floor_and_over_input_counts_without_wrapping() {
        let sr = 48_000.0;
        let mut m = Meter::new(sr);
        silence_block(&mut m, (sr * 0.5) as usize);
        for kind in 0..=8 {
            m.kind = kind;
            let r = m.reading(0);
            assert!(r.is_finite(), "kind {} reads non-finite on silence", kind);
            assert!(r <= -100.0, "kind {} silence floor {} dB", kind, r);
            assert_eq!(db_text(r), "−−.−", "kind {} silence must show the floor", kind);
        }
        // Over full-scale: three consecutive samples above 0 dBFS count one over; no wrapping.
        let hot = vec![1.5f32; 2 * 480];
        m.process_interleaved(&hot, 2);
        assert!(m.overs >= 1, "over counter did not count the clipped run");
        assert!(m.reading(0).is_finite(), "reading wrapped on over-full-scale input");
        assert!(m.channels[0].clip_t > 0.0, "clip indicator did not latch");
    }

    #[test]
    fn radar_links_stereo_by_the_louder_channel() {
        let sr = 48_000.0;
        let mut m = Meter::new(sr);
        m.kind = 1; // RMS
        let frames = (sr * 2.0) as usize;
        let mut inter = vec![0.0f32; frames * 2];
        let step = std::f32::consts::TAU * 1000.0 / sr as f32;
        let mut phase = 0.0f32;
        for i in 0..frames {
            let v = phase.sin();
            phase += step;
            inter[i * 2] = 10f32.powf(-10.0 / 20.0) * v; // loud left
            inter[i * 2 + 1] = 10f32.powf(-30.0 / 20.0) * v; // quiet right
        }
        m.process_interleaved(&inter, 2);
        let loud = m.reading(0);
        assert!(
            (m.radar_reading() - loud).abs() < 0.01,
            "radar must follow the louder channel: radar {} vs left {}",
            m.radar_reading(),
            loud
        );
    }

    #[test]
    fn readout_cards_sit_inside_the_panel_with_equal_margins() {
        let cells = readout_cells();
        let panel = READOUT_PANEL;
        let top = cells[0].y - panel.y;
        let bottom = panel.y + panel.h - (cells[4].y + cells[4].h);
        assert!((top - READOUT_PAD).abs() < 0.01 && (bottom - READOUT_PAD).abs() < 0.01, "margins {top} / {bottom}");
        for pair in cells.windows(2) {
            assert!(pair[1].y - (pair[0].y + pair[0].h) > 4.0, "cards must not touch");
        }
        for c in cells {
            assert!(c.x > panel.x && c.x + c.w < panel.x + panel.w);
            // The true-peak tick (bar height 8 plus 3 px either side) must stay inside its card.
            assert!(TP_BAR_Y - 3.0 >= 0.0 && TP_BAR_Y + 8.0 + 3.0 <= c.h, "tick leaves the card");
        }
    }

    #[test]
    fn target_and_ceiling_accept_any_value_inside_their_range() {
        let mut m = Meter::new(48_000.0);
        let mut setup = RadarVuSetup {
            mode: 0, target: -16.5, ceiling: -2.3, window: 10.0, kind: 9,
            reference_db: -18.0, fill_on: 1, fft_on: 1, history_points: HISTORY as i32, skin: 0,
        };
        radarvu_setup(&mut m, &setup);
        assert_eq!((m.target, m.ceiling), (-16.5, -2.3));
        setup.target = -90.0; setup.ceiling = 5.0;
        radarvu_setup(&mut m, &setup);
        assert_eq!((m.target, m.ceiling), (TARGET_RANGE.0, CEILING_RANGE.1));
        setup.target = f32::NAN; setup.ceiling = f32::INFINITY;
        radarvu_setup(&mut m, &setup);
        assert_eq!((m.target, m.ceiling), (-14.0, -1.0), "non-finite input falls back to the defaults");
        // A custom target and ceiling still paint inside the drawing budget in every skin.
        setup.target = -9.5; setup.ceiling = -4.4;
        for skin in 0..6 {
            setup.skin = skin;
            radarvu_setup(&mut m, &setup);
            assert!(paint(&mut m).cmds.len() < 8192);
        }
    }
}
